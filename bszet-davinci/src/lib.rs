extern crate core;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::LAST_MODIFIED;
use reqwest::{Client, Url};
use select::document::Document;
use select::predicate::Name;
use time::format_description::well_known::Rfc2822;
use time::Month::January;
use time::{Date, Month, OffsetDateTime};
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing::info;

use change::Change;

use crate::iteration::get_iteration;
use crate::timetable::igd21::IGD21;
use crate::timetable::Lesson;

const DATE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("\\S+ (\\d{2})\\.(\\d{2})\\.(\\d{4})").unwrap());
const REPLACEMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\+(.*) \\((.+)\\)").unwrap());

mod change;
mod iteration;
#[cfg(test)]
mod test;
pub mod timetable;

pub struct Davinci {
  client: Client,
  username: String,
  password: String,
  entrypoint: Url,
  data: RwLock<Option<Data>>,
}

pub struct Data {
  pub last_checked: OffsetDateTime,
  pub last_modified: OffsetDateTime,
  pub rows: HashSet<Row>,
  pub visited_urls: Vec<Url>,
}

impl Davinci {
  pub fn new(entrypoint: Url, username: String, password: String) -> Self {
    Self {
      client: Client::new(),
      username,
      password,
      entrypoint,
      data: RwLock::new(None),
    }
  }

  pub async fn data(&self) -> RwLockReadGuard<'_, Option<Data>> {
    self.data.read().await
  }

  pub async fn get_applied_timetable(&self, date: Date) -> (Vec<Lesson>, Vec<Row>) {
    let iteration = match get_iteration(date) {
      None => panic!("Unable to find iteration for date {date}"),
      Some(iteration) => iteration,
    };

    let mut day = IGD21
      .get(&date.weekday())
      .unwrap()
      .iter()
      .filter_map(|lesson| {
        if let Some(l_iteration) = lesson.iteration {
          if l_iteration != iteration {
            return None;
          }
        }
        Some(lesson.clone())
      })
      .collect::<Vec<Lesson>>();

    let mut relevant_rows = Vec::new();

    if let Some(data) = self.data.read().await.as_ref() {
      for row in &data.rows {
        if row.date != date
          || !(row.class.contains(&"IGD21".to_string())
            || row.class.contains(&"IGD 21".to_string()))
        {
          continue;
        }

        if !row.change.apply(&mut day) {
          relevant_rows.push(row.clone());
        }
      }
    }

    (day, relevant_rows)
  }

  pub async fn update(&self) -> anyhow::Result<bool> {
    let mut start_url = self.entrypoint.clone();
    let mut rows = Vec::new();
    let mut html_rows = Vec::new();

    let mut visited_urls = Vec::new();

    loop {
      visited_urls.push(start_url.clone());

      match self.fetch(start_url, &mut rows, &mut html_rows).await? {
        None => break,
        Some(next) => start_url = next,
      };
    }

    let mut html_rows2: HashMap<Date, Vec<String>> = HashMap::new();

    for (date, row) in html_rows {
      let mut buf = String::new();
      buf.push_str("<tr>");

      for x in row {
        buf.push_str("<td>");
        buf.push_str(&x);
        buf.push_str("</td>");
      }
      buf.push_str("</tr>");

      match html_rows2.entry(date) {
        Entry::Occupied(mut entry) => entry.get_mut().push(buf),
        Entry::Vacant(entry) => {
          entry.insert(Vec::from([buf]));
        }
      }
    }

    println!("{html_rows2:?}");

    let base = include_str!("index.html");
    let (date, html_rows) = (
      Date::from_calendar_date(2023, January, 27)?,
      html_rows2
        .get(&Date::from_calendar_date(2023, January, 27)?)
        .unwrap(),
    );
    println!(
      "{}",
      base
        .replace(
          "{{DATE}}",
          &format!(
            "{} den {:0<2}. {} {}",
            date.weekday(),
            date.day(),
            date.month(),
            date.year()
          ),
        )
        .replace("{{TABLE}}", &html_rows.join(""))
    );

    let now = OffsetDateTime::now_utc();

    let mut data = self.data.write().await;

    let mut hash = HashSet::with_capacity(rows.len());
    for row in rows {
      hash.insert(row);
    }

    // check if there is a difference
    if let Some(data) = data.as_mut() {
      // if !hash.iter().zip(&data.rows).any(|(a, b)| a != b) {
      if hash == data.rows {
        data.last_checked = now;
        return Ok(false);
      }
    }

    *data = Some(Data {
      last_checked: now,
      last_modified: now,
      rows: hash,
      visited_urls,
    });

    Ok(true)
  }

  async fn fetch(
    &self,
    url: Url,
    rows: &mut Vec<Row>,
    html_rows: &mut Vec<(Date, Vec<String>)>,
  ) -> anyhow::Result<Option<Url>> {
    let response = self
      .client
      .get(url.clone())
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await?
      .error_for_status()?;

    let last_modified = match response.headers().get(LAST_MODIFIED) {
      None => return Err(anyhow!("last-modified http header is required")),
      Some(value) => OffsetDateTime::parse(value.to_str()?, &Rfc2822)?,
    };

    info!("Crawled {}, last modified {}", url, last_modified);

    let text = response.text().await?;
    let document = Document::from(text.as_str());

    let mut date = None;
    for node in document.find(Name("h1")) {
      if let Some(captures) = DATE_REGEX.captures(&node.text()) {
        let day = u8::from_str(captures.get(1).unwrap().as_str()).unwrap();
        let month = u8::from_str(captures.get(2).unwrap().as_str()).unwrap();
        let year = i32::from_str(captures.get(3).unwrap().as_str()).unwrap();

        date = Some(Date::from_calendar_date(
          year,
          Month::try_from(month)?,
          day,
        )?);
      }
    }

    let date = if let Some(date) = date {
      date
    } else {
      return Err(anyhow!("Missing date in document"));
    };

    for row in document.find(Name("tr")) {
      let mut columns = Vec::with_capacity(7);

      for data in row.find(Name("td")) {
        columns.push(data.text().trim().to_string());
      }

      html_rows.push((date, columns));
    }

    let table = if let Some(table) = document.find(Name("tbody")).next() {
      table
    } else {
      return Err(anyhow!("Missing time table in document"));
    };

    for row in table.find(Name("tr")) {
      let columns = row
        .find(Name("td"))
        .map(|data| {
          let text = data.text();
          let column = clean(&text);
          column.to_string()
        })
        .collect::<Vec<String>>();

      if columns.len() != 7 {
        panic!("Invalid count of columns");
      }

      let class = if columns[0].is_empty() {
        None
      } else {
        Some(
          columns[0]
            .split(',')
            .map(|value| value.trim().to_string())
            .collect::<Vec<String>>(),
        )
      };

      let lesson = if columns[1].is_empty() {
        None
      } else {
        Some(convert_lesson(
          u8::from_str(&columns[1][..columns[1].len() - 1]).unwrap(),
        ))
      };

      let notice = if columns[6].is_empty() {
        None
      } else {
        Some(columns[6].to_string())
      };

      let row = if let Some(last) = rows.last() {
        Row {
          date,
          class: class.unwrap_or_else(|| last.class.clone()),
          change: Change::new(
            lesson.unwrap_or(last.change.lesson()),
            &columns[5],
            columns[2].as_str(),
            columns[3].to_string(),
            &columns[4],
            notice,
          )?,
        }
      } else {
        Row {
          date,
          class: class.expect("First row, can not have missing fields."),
          change: Change::new(
            lesson.expect("First row, can not have missing fields."),
            &columns[5],
            columns[2].as_str(),
            columns[3].to_string(),
            &columns[4],
            notice,
          )?,
        }
      };

      rows.push(row);
    }

    if let Some(js) = document
      .find(Name("input"))
      .filter_map(|input| input.attr("onclick"))
      .last()
    {
      let next = self.entrypoint.join(&js[22..js.len() - 1])?;
      if next != url {
        return Ok(Some(next));
      }
    }

    Ok(None)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Row {
  pub date: Date,
  pub class: Vec<String>,
  pub change: Change,
}

/// Removes starting `(` and ending `)` characters.
fn clean(value: &str) -> &str {
  let value = value.trim();

  if value.starts_with('(') && value.ends_with(')') {
    return &value[1..value.len() - 1];
  }

  value
}

/// Convert raw lesson to block lesson
fn convert_lesson(lesson: u8) -> u8 {
  (lesson + lesson % 2) / 2
}
