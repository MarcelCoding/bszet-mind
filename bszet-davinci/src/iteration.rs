use std::collections::HashMap;

use time::{Date, Duration, Month};

pub(crate) fn get_iteration(date: Date) -> Option<u8> {
  let iterations = HashMap::from([
    (
      Date::from_calendar_date(2021, Month::September, 06).unwrap(),
      1u8,
    ),
    (
      Date::from_calendar_date(2021, Month::September, 13).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::September, 20).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::September, 27).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::October, 04).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::October, 11).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::November, 01).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::November, 08).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::November, 15).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::November, 22).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::November, 29).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::December, 06).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2021, Month::December, 13).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2021, Month::December, 20).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::January, 03).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::January, 10).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::January, 17).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::January, 24).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::January, 31).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::February, 07).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::February, 28).unwrap(),
      1,
    ),
    (Date::from_calendar_date(2022, Month::March, 07).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::March, 14).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::March, 21).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::March, 28).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::April, 04).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::April, 11).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::April, 25).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::May, 02).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::May, 09).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::May, 16).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::May, 23).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::May, 30).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::June, 07).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::June, 13).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::June, 20).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::June, 27).unwrap(), 1),
    (Date::from_calendar_date(2022, Month::July, 04).unwrap(), 2),
    (Date::from_calendar_date(2022, Month::July, 11).unwrap(), 1),
    // SUMMER HOLIDAYS 2022 🌴
    (
      Date::from_calendar_date(2022, Month::August, 29).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::September, 05).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::September, 12).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::September, 19).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::September, 26).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::October, 03).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::October, 10).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::October, 31).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::November, 07).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::November, 14).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::November, 21).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::November, 28).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::December, 05).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2022, Month::December, 12).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2022, Month::December, 19).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2023, Month::January, 02).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2023, Month::January, 09).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2023, Month::January, 16).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2023, Month::January, 23).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2023, Month::January, 30).unwrap(),
      2,
    ),
    (
      Date::from_calendar_date(2023, Month::February, 06).unwrap(),
      1,
    ),
    (
      Date::from_calendar_date(2023, Month::February, 20).unwrap(),
      2,
    ),
    (Date::from_calendar_date(2023, Month::March, 06).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::March, 13).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::March, 20).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::March, 27).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::April, 03).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::April, 17).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::April, 24).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::May, 01).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::May, 08).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::May, 15).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::May, 22).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::May, 29).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::June, 05).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::June, 12).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::June, 19).unwrap(), 1),
    (Date::from_calendar_date(2023, Month::June, 26).unwrap(), 2),
    (Date::from_calendar_date(2023, Month::July, 03).unwrap(), 1),
  ]);

  for i in 0..7i64 {
    let date = date - Duration::days(i);
    if let Some(date) = iterations.get(&date) {
      return Some(*date);
    }
  }

  None
}

#[cfg(test)]
mod test {
  use time::{Date, Month};

  use crate::iteration::get_iteration;

  #[test]
  fn test_get_iteration() {
    let date1 = Date::from_calendar_date(2021, Month::September, 13).unwrap();
    let date2 = Date::from_calendar_date(2021, Month::September, 14).unwrap();
    let date3 = Date::from_calendar_date(2021, Month::September, 15).unwrap();
    let date4 = Date::from_calendar_date(2021, Month::September, 16).unwrap();
    let date5 = Date::from_calendar_date(2021, Month::September, 17).unwrap();
    let date6 = Date::from_calendar_date(2021, Month::September, 18).unwrap();
    let date7 = Date::from_calendar_date(2021, Month::September, 19).unwrap();
    let date8 = Date::from_calendar_date(2021, Month::September, 20).unwrap();
    let date9 = Date::from_calendar_date(2021, Month::September, 21).unwrap();

    assert_eq!(Some(2), get_iteration(date1));
    assert_eq!(Some(2), get_iteration(date2));
    assert_eq!(Some(2), get_iteration(date3));
    assert_eq!(Some(2), get_iteration(date4));
    assert_eq!(Some(2), get_iteration(date5));
    assert_eq!(Some(2), get_iteration(date6));
    assert_eq!(Some(2), get_iteration(date7));
    assert_eq!(Some(1), get_iteration(date8));
    assert_eq!(Some(1), get_iteration(date9));
  }
}
