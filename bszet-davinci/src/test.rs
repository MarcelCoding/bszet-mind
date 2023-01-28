use crate::{convert_lesson, Davinci};

#[tokio::test]
async fn test_load() -> anyhow::Result<()> {
  let mut davinci = Davinci::new(
    "https://geschuetzt.bszet.de/s-lk-vw/Vertretungsplaene/V_PlanBGy/V_DC_001.html"
      .parse()
      .unwrap(),
    "".to_string(),
    "".to_string(),
  );

  println!("{:?}", davinci.update().await?);

  // assert_eq!(true, davinci.update().await?);
  // assert_eq!(false, davinci.update().await?);

  for row in &davinci.data().await.as_ref().unwrap().rows {
    println!("- {:?}", row);
  }

  Ok(())
}

#[test]
fn test_convert_lesson() {
  assert_eq!(1, convert_lesson(1));
  assert_eq!(1, convert_lesson(2));
  assert_eq!(2, convert_lesson(3));
  assert_eq!(2, convert_lesson(4));
  assert_eq!(3, convert_lesson(5));
  assert_eq!(3, convert_lesson(6));
  assert_eq!(4, convert_lesson(7));
  assert_eq!(4, convert_lesson(8));
  assert_eq!(5, convert_lesson(9));
  assert_eq!(5, convert_lesson(10));
}
