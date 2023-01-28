use std::str::FromStr;

use anyhow::anyhow;
use time::Date;

use crate::change::Change;
use crate::Row;

pub(crate) fn parse(
  table: Vec<Vec<String>>,
  date: &Date,
  rows: &mut Vec<Row>,
) -> anyhow::Result<()> {
  for (index, columns) in table.into_iter().enumerate() {
    if columns.len() != 7 {
      return Err(anyhow!(
        "Invalid amount of columns; expected 7, got {}",
        columns.len()
      ));
    }

    let class = parse_optional(&columns[0]).map(parse_list);

    let lesson = match parse_optional(&columns[1]) {
      Some(lesson) => Some(parse_lesson(lesson)?),
      None => None,
    };

    let notice = parse_optional(&columns[6]).map(|notice| notice.to_string());

    let type_of_change = &columns[5];
    let subject = &columns[2];
    let place = columns[3].to_string();
    let teachers = &columns[4];

    let row = if let Some(last) = rows.last() {
      Row {
        index: index as u8,
        date: *date,
        class: class.unwrap_or_else(|| last.class.clone()),
        change: Change::new(
          lesson.unwrap_or(last.change.lesson()),
          type_of_change,
          subject,
          place,
          teachers,
          notice,
        )?,
        raw: columns,
      }
    } else {
      Row {
        index: index as u8,
        date: *date,
        class: class.ok_or_else(|| anyhow!("First row, can not have missing fields."))?,
        change: Change::new(
          lesson.ok_or_else(|| anyhow!("First row, can not have missing fields."))?,
          type_of_change,
          subject,
          place,
          teachers,
          notice,
        )?,
        raw: columns,
      }
    };

    rows.push(row);
  }

  Ok(())
}

fn parse_optional(value: &str) -> Option<&str> {
  if value.is_empty() {
    None
  } else {
    Some(value)
  }
}

fn parse_list(value: &str) -> Vec<String> {
  value
    .split(',')
    .map(|string| string.trim().to_string())
    .collect()
}

fn parse_lesson(value: &str) -> anyhow::Result<u8> {
  // split ending point from number
  let raw = &value[..value.len() - 1];
  let num = u8::from_str(raw)?;

  Ok(convert_lesson(num))
}

/// Convert raw lesson to block lesson
fn convert_lesson(lesson: u8) -> u8 {
  (lesson + lesson % 2) / 2
}

#[cfg(test)]
mod test {
  use crate::extractor::parser::convert_lesson;

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
}
