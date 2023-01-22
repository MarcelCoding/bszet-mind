use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::{Client, Url};
use serde::Serialize;

pub struct Telegram {
  client: Client,
  base: Url,
}

#[derive(Serialize)]
struct SendMessage {
  chat_id: i32,
  text: String,
  parse_mode: String,
}

impl Telegram {
  pub fn new(token: &str) -> anyhow::Result<Self> {
    let raw = format!("https://api.telegram.org/bot{}/", token);
    let base = Url::parse(&raw)?;

    Ok(Self {
      client: Client::new(),
      base,
    })
  }

  pub async fn send(&self, chat_id: i32, text: String) -> anyhow::Result<()> {
    self
      .client
      .post(self.base.join("sendMessage")?)
      .header(CONTENT_TYPE, HeaderValue::from_str("application/json")?)
      .body(serde_json::to_string(&SendMessage {
        chat_id,
        text,
        parse_mode: "markdown".to_string(),
      })?)
      .send()
      .await?
      .error_for_status()?;

    Ok(())
  }
}
