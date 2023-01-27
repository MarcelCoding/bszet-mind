use std::fmt::Write;
use std::time::Duration;

use clap::Parser;
use reqwest::Url;
use time::{OffsetDateTime, Weekday};
use tokio::time::Instant;
use tracing::{error, info};

use bszet_davinci::Davinci;
use bszet_image::WebToImageConverter;
use bszet_notify::telegram::Telegram;

use crate::ascii::table;

mod ascii;

#[derive(Parser)]
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
  gecko_driver_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  tracing_subscriber::fmt::init();

  let davinci = Davinci::new(
    args.entrypoint.clone(),
    args.username.clone(),
    args.password.clone(),
  );

  let web_to_image_converter = WebToImageConverter::new(args.gecko_driver_url.as_str()).await?;

  loop {
    match davinci.update().await {
      Err(err) => error!("Error executing davinci update schedule: {}", err),
      Ok(false) => {
        let now = OffsetDateTime::now_utc();

        if now.hour() == 15 && now.minute() <= 14 {
          send_notifications(&args, &davinci, &web_to_image_converter).await?;
          info!("Send 15 o'clock notification");
        } else {
          info!("Nothing changed");
        }
      }
      Ok(true) => {
        info!("Detected changes, sending notifications...");

        send_notifications(&args, &davinci, &web_to_image_converter).await?;
      }
    }

    await_next_execution().await;
  }
}

async fn send_notifications(
  args: &Args,
  davinci: &Davinci,
  web_to_image_converter: &WebToImageConverter,
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

  let (day, unknown_changes) = davinci.get_applied_timetable(now.date()).await;
  let table = table(day);

  let telegram = Telegram::new(&args.telegram_token)?;

  let image_result = match davinci.data().await.as_ref() {
    Some(data) => {
      let mut images = Vec::new();

      for url in &data.visited_urls {
        let mut authenticated_url = url.clone();
        authenticated_url
          .set_username(args.username.as_str())
          .unwrap();
        authenticated_url
          .set_password(Some(args.password.as_str()))
          .unwrap();

        images.push(
          web_to_image_converter
            .create_image(authenticated_url.as_str())
            .await?,
        )
      }

      Some(images)
    }
    None => None,
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
        writeln!(text, "- {:?}", row).unwrap();
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
