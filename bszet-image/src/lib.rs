use fantoccini::{Client, ClientBuilder, Locator};

pub struct WebToImageConverter {
  client: Client,
}

impl WebToImageConverter {
  pub async fn new(gecko_driver_url: &str) -> anyhow::Result<Self> {
    let client = ClientBuilder::rustls().connect(gecko_driver_url).await?;

    Ok(Self { client })
  }

  pub async fn create_image(&self, url: &str) -> anyhow::Result<Vec<u8>> {
    self.client.set_window_rect(0, 0, 1500, 10_000).await?;
    self.client.goto(url).await?;

    let image = self
      .client
      .find(Locator::Css("body"))
      .await?
      .screenshot()
      .await?;

    Ok(image)
  }

  pub async fn close(&self) -> anyhow::Result<()> {
    self.client.close_window().await?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::fs::File;
  use std::io::Write;

  use crate::WebToImageConverter;

  fn write_to_file(file_name: &str, data: &Vec<u8>) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
  }

  #[tokio::test]
  async fn open_selenium() -> anyhow::Result<()> {
    let web_to_image_convert = WebToImageConverter::new("http://127.0.0.1:4444").await?;

    let image = web_to_image_convert
      .create_image("https://www.google.com")
      .await;
    web_to_image_convert.close().await?;
    let image = image?;

    write_to_file("cool_img.png", &image)?;

    Ok(())
  }
}
