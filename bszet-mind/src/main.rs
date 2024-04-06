use std::collections::HashSet;
use std::fmt::Write;
use std::future::IntoFuture;
use std::iter::once;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use axum::extract::Path;
use axum::http::header::AUTHORIZATION;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router};
use clap::{arg, Parser};
use http_body_util::{BodyExt, Empty, Full};
use include_dir::{include_dir, Dir};
use reqwest::Url;
use time::{Date, OffsetDateTime, Weekday};
use tokio::net::TcpListener;
use tokio::select;
use tokio::time::Instant;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::trace::TraceLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;
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

#[cfg(test)]
mod tests;

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
  #[arg(
    long,
    short,
    env = "BSZET_MIND_USERNAME",
    conflicts_with = "username_file",
    required_unless_present = "username_file"
  )]
  username: Option<String>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_USERNAME_FILE",
    conflicts_with = "username",
    required_unless_present = "username"
  )]
  username_file: Option<PathBuf>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_PASSWORD",
    conflicts_with = "password_file",
    required_unless_present = "password_file"
  )]
  password: Option<String>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_PASSWORD_FILE",
    conflicts_with = "password",
    required_unless_present = "password"
  )]
  password_file: Option<PathBuf>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_TELEGRAM_TOKEN",
    conflicts_with = "telegram_token_file",
    required_unless_present = "telegram_token_file"
  )]
  telegram_token: Option<String>,
  #[arg(
    long,
    short,
    env = "BSZET_MIND_TELEGRAM_TOKEN_FILE",
    conflicts_with = "telegram_token",
    required_unless_present = "telegram_token"
  )]
  telegram_token_file: Option<String>,
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
  #[arg(
    long,
    env = "BSZET_MIND_API_TOKEN",
    conflicts_with = "api_token_file",
    required_unless_present = "api_token_file"
  )]
  api_token: Option<String>,
  #[arg(
    long,
    env = "BSZET_MIND_API_TOKEN_FILE",
    conflicts_with = "api_token",
    required_unless_present = "api_token"
  )]
  api_token_file: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::fmt::Layer::new()
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .compact(),
    )
    .init();

  let args2 = args.clone();

  let password = match args.password {
    None => tokio::fs::read_to_string(args.password_file.unwrap()).await?,
    Some(password) => password,
  };

  let username = match args.username {
    None => tokio::fs::read_to_string(args.username_file.unwrap()).await?,
    Some(username) => username,
  };

  let api_token = match args.api_token {
    None => tokio::fs::read_to_string(args.api_token_file.unwrap()).await?,
    Some(api_token) => api_token,
  };

  let telegram_token = match args.telegram_token {
    None => tokio::fs::read_to_string(args.telegram_token_file.unwrap()).await?,
    Some(telegram_token) => telegram_token,
  };

  let davinci = Arc::new(Davinci::new(args.entrypoint.clone(), username, password));

  let davinci2 = davinci.clone();

  let router = Router::new()
    .route("/davinci/:date/:class", get(timetable))
    .layer(Extension(davinci2.clone()))
    .layer(ValidateRequestHeaderLayer::bearer(&api_token))
    .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
    .layer(TraceLayer::new_for_http());

  let internal_router = Router::new()
    .route("/davinci/:date", get(html_plan))
    .route("/static/*path", get(static_path))
    .layer(Extension(davinci2.clone()))
    .layer(TraceLayer::new_for_http());

  let telegram = Telegram::new(&telegram_token)?;

  tokio::spawn(async move {
    let davinci2 = davinci2;
    loop {
      if let Err(err) = iteration(&args2, &telegram, &davinci2).await {
        error!("Error while executing loop: {}", err);
      }
    }
  });

  info!("Listening on http://{}...", args.listen_addr);
  let listener = TcpListener::bind(args.listen_addr).await?;

  info!(
    "Listening on http://{}... (internal)",
    args.internal_listen_addr
  );
  let internal_listener = TcpListener::bind(args.internal_listen_addr).await?;

  select! {
    public = axum::serve(listener, router).into_future() => {
      public?;
    }
    internal = axum::serve(internal_listener, internal_router).into_future() => {
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
      .body(Empty::new().boxed())
      .unwrap(),
    Some(file) => Response::builder()
      .status(StatusCode::OK)
      .header(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime_type).unwrap(),
      )
      .body(Full::from(file.contents()).boxed())
      .unwrap(),
  }
}

async fn iteration(args: &Args, telegram: &Telegram, davinci: &Davinci) -> anyhow::Result<()> {
  let result = match davinci.update().await {
    Err(err) => Err(anyhow!(format!(
      "Error executing davinci update schedule: {}",
      err
    ))),
    Ok(false) => {
      let now = OffsetDateTime::now_utc();

      if now.hour() == 15 && now.minute() <= 14 {
        info!("Send 15 o'clock notification");
        send_notifications(args, telegram, davinci).await
      } else {
        info!("Nothing changed");
        Ok(())
      }
    }
    Ok(true) => {
      info!("Detected changes, sending notifications...");

      send_notifications(args, telegram, davinci).await
    }
  };

  if let Err(err) = result {
    error!("Unable to execute iteration: {:?}", err);
  }

  await_next_execution().await;

  Ok(())
}

async fn send_notifications(
  args: &Args,
  telegram: &Telegram,
  davinci: &Davinci,
) -> anyhow::Result<()> {
  let mut now = OffsetDateTime::now_utc();

  if now.hour() >= 15 {
    now += time::Duration::days(1);
  }

  match now.weekday() {
    Weekday::Saturday => now += time::Duration::days(2),
    Weekday::Sunday => now += time::Duration::days(1),
    _ => {}
  }

  let (last_modified, day, unknown_changes, iteration) =
    davinci.get_applied_timetable(now.date()).await?;

  let table = table(day);

  let image_result = render_images(&args.gecko_driver_url, &args.internal_url, davinci)
    .await
    .unwrap_or_else(|err| {
      error!("Error while rendering images: {}", err);
      None
    });

  for id in &args.chat_ids {
    let age = last_modified
      .map(|last_modified| (OffsetDateTime::now_utc() - last_modified).unsigned_abs())
      .unwrap_or_else(|| Duration::from_secs(0));

    let mut text = format!(
      "Vertretungsplan für {} den {}. {} {}, Turnus {}. Zuletzt vor {} aktualisiert.\n```\n{}```",
      now.weekday(),
      now.day(),
      now.month(),
      now.year(),
      iteration,
      format_duration(age),
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

fn format_duration(duration: Duration) -> String {
  let secs = duration.as_secs();

  let units = [
    ("einem Jahr", "Jahren", 31_557_600),
    ("einem Monat", "Monaten", 2_630_016),
    ("einem Tag", "Tagen", 86400),
    ("einer Stunde", "Stunden", 3600),
    ("einer Minute", "Minuten", 60),
    ("einer Sekunde", "Sekunden", 1),
  ];

  let mut last = None;
  let mut last_remaining = secs;

  for (one, many, seconds) in units {
    let value = last_remaining / seconds;
    let remaining = last_remaining % seconds;

    if value != 0 {
      if let Some(last) = last {
        return format!(
          "{} und {}",
          last,
          match value {
            1 => one.to_string(),
            value => format!("{value} {many}"),
          }
        );
      } else {
        last = Some(match value {
          1 => one.to_string(),
          value => format!("{value} {many}"),
        });
      }
    }

    last_remaining = remaining;
  }

  last.unwrap_or_else(|| "idk".to_string())
}
