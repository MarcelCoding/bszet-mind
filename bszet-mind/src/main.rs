use std::collections::HashSet;
use std::fmt::Write;
use std::iter::once;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Empty, Full};
use axum::extract::Path;
use axum::http::header::AUTHORIZATION;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body, Extension, Router, Server};
use clap::{arg, Parser};
use include_dir::{include_dir, Dir};
use reqwest::Url;
use time::{Date, OffsetDateTime, Weekday};
use tokio::select;
use tokio::time::Instant;
use tower_http::auth::RequireAuthorizationLayer;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use bszet_davinci::Davinci;
use bszet_image::WebToImageConverter;
use bszet_notify::telegram::Telegram;

use crate::api::davinci::{html_plan, timetable};
use crate::ascii::table;

mod api;
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
    env = "BSZET_MIND_INTERNAL_LISTEN_ADDR",
    default_value = "127.0.0.1:8081"
  )]
  internal_listen_addr: SocketAddr,
  #[arg(
    long,
    env = "BSZET_MIND_INTERNAL_URL",
    default_value = "http://127.0.0.1:8081"
  )]
  internal_url: Url,
  #[arg(long, short, env = "BSZET_MIND_SENTRY_DSN")]
  sentry_dsn: Url,
  #[arg(long, env = "BSZET_MIND_API_TOKEN")]
  api_token: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let _guard = sentry::init((
    args.sentry_dsn.as_str(),
    sentry::ClientOptions {
      release: sentry::release_name!(),
      traces_sample_rate: 1.0,
      ..Default::default()
    },
  ));

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::fmt::Layer::new()
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .compact(),
    )
    .with(sentry_tracing::layer())
    .init();

  let result = real_main(args).await;

  if let Some(client) = sentry::Hub::current().client() {
    client.close(Some(Duration::from_secs(2)));
  }

  result
}

async fn real_main(args: Args) -> anyhow::Result<()> {
  let davinci = Arc::new(Davinci::new(
    args.entrypoint.clone(),
    args.username.clone(),
    args.password.clone(),
  ));

  let args2 = args.clone();
  let davinci2 = davinci.clone();

  let router = Router::new()
    .route("/davinci/:date/:class", get(timetable))
    .layer(Extension(davinci2.clone()))
    .layer(RequireAuthorizationLayer::bearer(&args.api_token))
    .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
    .layer(TraceLayer::new_for_http());

  let internal_router = Router::new()
    .route("/davinci/:date", get(html_plan))
    .route("/static/*path", get(static_path))
    .layer(Extension(davinci2.clone()))
    .layer(TraceLayer::new_for_http());

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
  info!(
    "Listening on http://{}... (internal)",
    args.internal_listen_addr
  );

  select! {
    public = Server::bind(&args.listen_addr).serve(router.into_make_service()) => {
      public?;
    }
    internal = Server::bind(&args.internal_listen_addr).serve(internal_router.into_make_service()) => {
      internal?;
    }
  }

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
    Ok(result) => result,
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
    now_sec_to_next_15_prec % 60,
  );
  tokio::time::sleep_until(sleep_until).await;
}
