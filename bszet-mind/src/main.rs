use std::string::ToString;
use std::time::Duration;

use clap::Parser;
use reqwest::Url;
use time::Month::January;
use time::{Date, OffsetDateTime};
use tokio::time::Instant;
use tracing::{error, info};

use bszet_davinci::Davinci;
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
    #[arg(long, short, env = "BSZET_MIND_CHAT_IDS")]
    chat_ids: Vec<i32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let davinci = Davinci::new(args.entrypoint, args.username, args.password);

    loop {
        match davinci.update().await {
            Err(err) => error!("Error executing davinci update schedule: {}", err),
            Ok(false) => info!("Nothing changed"),
            Ok(true) => {
                info!("Detected changes, sending notifications...");

                let table = table(
                    davinci
                        .get_applied_timetable(Date::from_calendar_date(2023, January, 20)?)
                        .await,
                );

                let telegram = Telegram::new(&args.telegram_token)?;

                for id in &args.chat_ids {
                    match davinci.data().await.as_ref() {
                        None => {
                            telegram
                                .send(
                                    *id,
                                    "Es konnte kein Vertretungsplan geladen werden.".to_string(),
                                )
                                .await?
                        }
                        Some(data) => {
                            let age = OffsetDateTime::now_utc() - data.last_checked;
                            let text = format!("Der Vertretungsplan wurde zuletzt vor {} aktualisiert.\n```\n{}```", age, table);
                            telegram.send(*id, text).await?;
                        }
                    }
                }
            }
        }

        await_next_execution().await;
    }
}

async fn await_next_execution() {
    let now = OffsetDateTime::now_utc();
    let sleep_until =
        Instant::now() + Duration::from_secs((60 * (15 - now.minute() % 15) - now.second()) as u64);
    info!(
        "Next run in {} seconds ({} minutes)",
        60 * (15 - now.minute() % 15) - now.second(),
        (60 * (15 - now.minute() % 15) - now.second()) as f32 / 60.0
    );
    tokio::time::sleep_until(sleep_until).await;
}
