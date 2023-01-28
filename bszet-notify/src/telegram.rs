use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Url};
use serde::Serialize;

pub struct Telegram {
  client: Client,
  base: Url,
}

#[derive(Debug, Serialize)]
enum ParseMode {
  Markdown,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "photo")]
struct InputMediaPhoto {
  media: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  caption: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  parse_mode: Option<ParseMode>,
}

#[derive(Debug, Serialize)]
struct SendMediaGroupData {
  chat_id: i64,
  message_thread_id: Option<i64>,
  media: Vec<InputMediaPhoto>,
  disable_notification: Option<bool>,
  protect_content: Option<bool>,
  reply_to_message_id: Option<i64>,
  allow_sending_without_reply: Option<bool>,
}

#[derive(Debug, Serialize)]
struct SendMessageData {
  chat_id: i64,
  text: String,
  parse_mode: ParseMode,
}

impl Telegram {
  pub fn new(token: &str) -> anyhow::Result<Self> {
    let raw = format!("https://api.telegram.org/bot{token}/");
    let base = Url::parse(&raw)?;

    Ok(Self {
      client: Client::new(),
      base,
    })
  }

  pub async fn send_text(&self, chat_id: i64, text: &str) -> anyhow::Result<()> {
    let data = SendMessageData {
      chat_id,
      text: text.to_string(),
      parse_mode: ParseMode::Markdown,
    };

    self
      .client
      .post(self.base.join("sendMessage")?)
      .json(&data)
      .send()
      .await?
      .error_for_status()?;

    Ok(())
  }

  pub async fn send_images(
    &self,
    chat_id: i64,
    text: &str,
    images: &[Vec<u8>],
  ) -> anyhow::Result<()> {
    let mut form = Form::new();
    let mut media = Vec::new();

    for (index, image) in images.iter().enumerate() {
      let file_name = format!("{index}.png");
      let field_name = format!("file{}", index + 1);

      form = form.part(
        field_name.clone(),
        Part::bytes(image.clone())
          .file_name(file_name.clone())
          .mime_str("image/png")?,
      );

      media.push(InputMediaPhoto {
        media: format!("attach://{}", field_name.clone()),
        caption: if index == 0 {
          Some(text.to_string())
        } else {
          None
        },
        parse_mode: Some(ParseMode::Markdown),
      })
    }

    form = form.part("chat_id", Part::text(chat_id.to_string()));

    let media_str = serde_json::to_string(&media)?;
    form = form.part("media", Part::text(media_str).mime_str("application/json")?);

    self
      .client
      .post(self.base.join("sendMediaGroup")?)
      .header(CONTENT_TYPE, HeaderValue::from_str("application/json")?)
      .multipart(form)
      .send()
      .await?
      .error_for_status()?;

    Ok(())
  }
}
