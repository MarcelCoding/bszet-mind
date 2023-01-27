use crate::telegram::Telegram;

#[tokio::test]
async fn send() -> anyhow::Result<()> {
  let telegram = Telegram::new("")?;
  telegram
    .send_images(-734603836, "Hallo".to_string())
    .await?;

  Ok(())
}
