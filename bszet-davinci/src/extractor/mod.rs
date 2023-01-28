use std::str::FromStr;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use select::document::Document;
use select::predicate::Name;
use time::{Date, Month};

pub(crate) use html_table::*;
pub(crate) use parser::*;

mod html_table;
mod parser;

static DATE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("\\S+ (\\d{2})\\.(\\d{2})\\.(\\d{4})").unwrap());

pub(crate) fn extract_date(doc: &Document) -> anyhow::Result<Date> {
  for node in doc.find(Name("h1")) {
    if let Some(captures) = DATE_REGEX.captures(&node.text()) {
      let day = u8::from_str(captures.get(1).unwrap().as_str()).unwrap();
      let month = u8::from_str(captures.get(2).unwrap().as_str()).unwrap();
      let year = i32::from_str(captures.get(3).unwrap().as_str()).unwrap();

      return Ok(Date::from_calendar_date(
        year,
        Month::try_from(month)?,
        day,
      )?);
    }
  }

  Err(anyhow!("Missing date in document"))
}

pub(crate) fn extract_next_page(doc: &Document) -> Option<&str> {
  doc
    .find(Name("input"))
    .filter_map(|input| input.attr("onclick"))
    .last()
    .map(|js| &js[22..js.len() - 1])
}

/// Removes starting `(` and ending `)` characters.
fn clean(value: &str) -> &str {
  let value = value.trim();

  if value.starts_with('(') && value.ends_with(')') {
    return &value[1..value.len() - 1];
  }

  value
}
