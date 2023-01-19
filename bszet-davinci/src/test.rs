use crate::Davinci;

#[tokio::test]
async fn test_load() -> anyhow::Result<()> {
  let mut davinci = Davinci::new(
    "https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_001.html".parse().unwrap(),
    "".to_string(),
    "".to_string(),
  );

  println!("{:?}", davinci.update().await?);

  // assert_eq!(true, davinci.update().await?);
  // assert_eq!(false, davinci.update().await?);

  for row in &davinci.data.unwrap().rows {
      println!("- {:?}", row);
  }

  Ok(())
}
