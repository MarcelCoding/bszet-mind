use crate::Davinci;

#[tokio::test]
async fn test_load() -> anyhow::Result<()> {
  let davinci = Davinci::new(
    "https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_001.html"
      .parse()
      .unwrap(),
    "bsz-et-2122".to_string(),
    "impfung#21".to_string(),
  );

  println!("{:?}", davinci.update().await?);

  // assert_eq!(true, davinci.update().await?);
  // assert_eq!(false, davinci.update().await?);

  for row in &davinci.data().await.as_ref().unwrap().rows {
    println!("- {:?}", row);
  }

  Ok(())
}
