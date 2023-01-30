use std::collections::HashSet;
use std::fmt::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Empty, Full};
use axum::extract::{Path, Query};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{body, Extension, Router, Server};
use clap::{arg, Parser};
use include_dir::{include_dir, Dir};
use reqwest::Url;
use serde::Deserialize;
use time::serde::format_description;
use time::{Date, OffsetDateTime, Weekday};
use tokio::time::Instant;
use tracing::{error, info};

use bszet_davinci::Davinci;
use bszet_image::WebToImageConverter;
use bszet_notify::telegram::Telegram;

use crate::ascii::table;
use crate::AppError::PlanUnavailable;

mod ascii;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[derive(Parser, Clone)]
#[command(author, version, about, long_about)]
struct Args {
  #[arg(
    long,
    short,
    env = "BSZET_MIND_ENTRYPOINT",
    default_value = "https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_001.html"
  )]
  entrypoint: Url,
  #[arg(long, short, env = "BSZET_MIND_USERNAME")]
  username: String,
  #[arg(long, short, env = "BSZET_MIND_PASSWORD")]
  password: String,
  #[arg(long, short, env = "BSZET_MIND_TELEGRAM_TOKEN")]
  telegram_token: String,
  #[arg(long, short, env = "BSZET_MIND_CHAT_IDS", value_delimiter = ',')]
  chat_ids: Vec<i64>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_GECKO_DRIVER_URL",
    default_value = "http://localhost:4444"
  )]
  gecko_driver_url: Url,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_LISTEN_ADDR",
    default_value = "127.0.0.1:8080"
  )]
  listen_addr: SocketAddr,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_INTERNAL_URL",
    default_value = "http://127.0.0.1:8080"
  )]
  internal_url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  tracing_subscriber::fmt::init();

  let davinci = Arc::new(Davinci::new(
    args.entrypoint.clone(),
    args.username.clone(),
    args.password.clone(),
  ));

  let args2 = args.clone();
  let davinci2 = davinci.clone();
  let davinci3 = davinci.clone();

  let router = Router::new()
    .route("/davinci/:date", get(plan))
    .route("/static/*path", get(static_path))
    .layer(Extension(davinci3));

  tokio::spawn(async move {
    let args2 = args2;
    let davinci2 = davinci2;
    loop {
      if let Err(err) = iteration(&args2, &davinci2).await {
        error!("Error while executing loop: {}", err);
      }
    }
  });

  info!("Listening on http://{}...", args.listen_addr);

  Server::bind(&args.listen_addr)
    .serve(router.into_make_service())
    .await?;

  Ok(())
}

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
  let path = path.trim_start_matches('/');
  let mime_type = match path.split('.').last() {
    Some("css") => "text/css",
    Some("woff2") => "font/woff2",
    _ => "application/octet-stream",
  };

  match STATIC_DIR.get_file(path) {
    None => Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(body::boxed(Empty::new()))
      .unwrap(),
    Some(file) => Response::builder()
      .status(StatusCode::OK)
      .header(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime_type).unwrap(),
      )
      .body(body::boxed(Full::from(file.contents())))
      .unwrap(),
  }
}
format_description!(iso_date, Date, "[year]-[month]-[day]");
#[derive(Deserialize)]
struct PlanPath {
  #[serde(with = "iso_date")]
  date: Date,
}

#[derive(Deserialize)]
struct PlanQuery {
  class: String,
}

async fn plan(
  Extension(davinci): Extension<Arc<Davinci>>,
  Path(PlanPath { date }): Path<PlanPath>,
  Query(PlanQuery { class }): Query<PlanQuery>,
) -> Result<impl IntoResponse, AppError> {
  let split = class.split(',').collect::<Vec<&str>>();
  Ok(Html(
    davinci
      .get_html(&date, split.as_slice())
      .await?
      .ok_or(PlanUnavailable)?,
  ))
}

async fn iteration(args: &Args, davinci: &Davinci) -> anyhow::Result<()> {
  match davinci.update().await {
    Err(err) => error!("Error executing davinci update schedule: {}", err),
    Ok(false) => {
      let now = OffsetDateTime::now_utc();

      if now.hour() == 15 && now.minute() <= 14 {
        send_notifications(args, davinci).await?;
        info!("Send 15 o'clock notification");
      } else {
        info!("Nothing changed");
      }
    }
    Ok(true) => {
      info!("Detected changes, sending notifications...");

      send_notifications(args, davinci).await?;
    }
  }

  await_next_execution().await;

  Ok(())
}

async fn send_notifications(args: &Args, davinci: &Davinci) -> anyhow::Result<()> {
  let mut now = OffsetDateTime::now_utc();

  if now.hour() >= 15 {
    now += time::Duration::days(1);
  }

  match now.weekday() {
    Weekday::Saturday => now += time::Duration::days(2),
    Weekday::Sunday => now += time::Duration::days(1),
    _ => {}
  }

  let (day, unknown_changes) = davinci.get_applied_timetable(now.date()).await;
  let table = table(day);

  let telegram = Telegram::new(&args.telegram_token)?;
  let image_result = match render_images(&args.gecko_driver_url, &args.internal_url, davinci).await
  {
    Ok(resukt) => resukt,
    Err(err) => {
      error!("Error while rendering images: {}", err);
      None
    }
  };

  for id in &args.chat_ids {
    // let age = OffsetDateTime::now_utc() - data.last_checked;
    let mut text = format!(
      "Vertretungsplan für {} den {}. {} {}.\n```\n{}```",
      now.weekday(),
      now.day(),
      now.month(),
      now.year(),
      table,
    );

    if !unknown_changes.is_empty() {
      writeln!(text, "\n\nÄnderungen, die nicht angewendet werden konnten:").unwrap();
      for row in &unknown_changes {
        writeln!(text, "- {row:?}").unwrap();
      }
    }

    match &image_result {
      Some(images) => {
        telegram.send_images(*id, text.as_str(), images).await?;
      }
      None => {
        telegram.send_text(*id, text.as_str()).await?;
      }
    }
  }

  Ok(())
}

async fn render_images(
  gecko_driver_url: &Url,
  base_url: &Url,
  davinci: &Davinci,
) -> anyhow::Result<Option<Vec<Vec<u8>>>> {
  let web_img_conv = WebToImageConverter::new(gecko_driver_url.as_str()).await?;

  match davinci.data().await.as_ref() {
    Some(data) => {
      let mut images = Vec::new();

      let dates = data
        .rows
        .iter()
        .map(|row| row.date)
        .collect::<HashSet<Date>>();
      let mut dates = dates.into_iter().collect::<Vec<Date>>();
      dates.sort();

      for date in dates {
        images.push(
          web_img_conv
            .create_image(
              base_url
                .join(&format!(
                  "davinci/{}-{:0>2}-{:0>2}?class=IGD21,IGD 21",
                  date.year(),
                  date.month() as u8,
                  date.day()
                ))?
                .as_str(),
            )
            .await?,
        )
      }

      Ok(Some(images))
    }

    None => Ok(None),
  }
}

async fn await_next_execution() {
  let now = OffsetDateTime::now_utc();

  let now_min = now.minute() as u64;
  let now_min_to_last_15 = now_min % 15;
  let now_min_to_next_15 = 15 - now_min_to_last_15;
  let now_sec_to_next_15 = now_min_to_next_15 * 60;
  let now_sec_to_next_15_prec = now_sec_to_next_15 - now.second() as u64;
  let duration = Duration::from_secs(now_sec_to_next_15_prec);

  let sleep_until = Instant::now() + duration;
  info!(
    "Next execution in {:0>2}:{:0>2} minutes",
    now_sec_to_next_15_prec / 60,
    now_sec_to_next_15_prec % 60
  );
  tokio::time::sleep_until(sleep_until).await;
}

enum AppError {
  InternalServerError(anyhow::Error),
  PlanUnavailable,
}

impl From<anyhow::Error> for AppError {
  fn from(inner: anyhow::Error) -> Self {
    AppError::InternalServerError(inner)
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, error_message) = match self {
      AppError::InternalServerError(inner) => {
        error!("stacktrace: {}", inner);
        (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
      }
      AppError::PlanUnavailable => (
        StatusCode::UNPROCESSABLE_ENTITY,
        "substitution plan is currently unavailable",
      ),
    };

    (status, error_message).into_response()
  }
}
