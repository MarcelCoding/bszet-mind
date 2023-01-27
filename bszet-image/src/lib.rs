use anyhow;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use thirtyfour::prelude::*;

pub struct WebToImageConverter {
  driver: WebDriver,
}

impl WebToImageConverter {
  pub async fn new(gecko_driver_url: &str) -> anyhow::Result<Self> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;

    Ok(Self {
      driver: WebDriver::new(gecko_driver_url, caps).await?,
    })
  }

  pub async fn create_image(&self, url: &str) -> anyhow::Result<Vec<u8>> {
    self.driver.set_window_rect(0, 0, 2900, 5000).await?;
    self.driver.goto(url).await?;
    self
      .driver
      .execute("document.body.style.MozTransform = \"scale(400%)\"", vec![])
      .await?;

    Ok(
      self
        .driver
        .find(By::ClassName("schedule-container"))
        .await?
        .screenshot_as_png()
        .await?,
    )
  }

  pub async fn quit(self) -> anyhow::Result<()> {
    self.driver.quit().await?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::WebToImageConverter;
  use std::fs::File;
  use std::io::Write;

  fn write_to_file(file_name: &str, data: &Vec<u8>) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(data)?;
    Ok(())
  }

  #[tokio::test]
  async fn open_selenium() -> anyhow::Result<()> {
    let web_to_image_convert = WebToImageConverter::new("http://localhost:4444").await?;

    let image = match web_to_image_convert.create_image("https://google.com").await {
      Ok(image) => image,
      Err(err) => {
        web_to_image_convert.quit().await?;
        return Err(err);
      },
    };

    if let Err(err) = write_to_file("cool_img.png", &image) {
      web_to_image_convert.quit().await?;
      return Err(err.into());
    }

    web_to_image_convert.quit().await?;

    Ok(())
  }
}
