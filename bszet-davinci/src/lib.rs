extern crate core;

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::LAST_MODIFIED;
use reqwest::{Client, Url};
use sailfish::TemplateOnce;
use select::document::Document;
use time::format_description::well_known::Rfc2822;
use time::{Date, OffsetDateTime};
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing::info;

use change::Change;

use crate::extractor::{extract_date, extract_html_table, extract_next_page, parse};
use crate::html::SubstitutionPlanTemplate;
use crate::iteration::get_iteration;
use crate::timetable::igd21::IGD21;
use crate::timetable::Lesson;

static REPLACEMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\+(.*) \\((.+)\\)").unwrap());

mod change;
mod extractor;
mod html;
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

  pub async fn get_html(&self, date: &Date, classes: &[&str]) -> anyhow::Result<Option<String>> {
    Ok(match self.data.read().await.as_ref() {
      None => None,
      Some(data) => {
        let mut table = data
          .rows
          .iter()
          .filter(|row| &row.date == date)
          .collect::<Vec<&Row>>();

        table.sort_by(|a, b| a.index.cmp(&b.index));

        let table = table
          .iter()
          .map(|row| row.raw.as_slice())
          .collect::<Vec<&[String]>>();

        Some(
          SubstitutionPlanTemplate {
            date: *date,
            table,
            classes,
          }
          .render_once()?,
        )
      }
    })
  }

  pub async fn update(&self) -> anyhow::Result<bool> {
    let mut start_url = self.entrypoint.clone();
    let mut rows = Vec::new();

    loop {
      match self.fetch(start_url, &mut rows).await? {
        None => break,
        Some(next) => start_url = next,
      };
    }

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
    });

    Ok(true)
  }

  async fn fetch(&self, url: Url, rows: &mut Vec<Row>) -> anyhow::Result<Option<Url>> {
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
    let doc = Document::from(text.as_str());

    let date = extract_date(&doc)?;

    let table = extract_html_table(&doc);
    parse(table, &date, rows)?;

    Ok(match extract_next_page(&doc) {
      None => None,
      Some(next) => {
        let next = url.join(next)?;
        if next == url {
          None
        } else {
          Some(next)
        }
      }
    })
  }
}

#[derive(Clone, Debug)]
pub struct Row {
  /// IF YOU ADD PROPERTIES, UPDATE IMPLEMENTATIONS BELOW
  // ignored for Eq, PartialEq and Hash
  pub index: u8,
  pub date: Date,
  pub class: Vec<String>,
  pub change: Change,
  // ignored for Eq, PartialEq and Hash
  pub raw: Vec<String>,
}

impl Hash for Row {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.date.hash(state);
    self.class.hash(state);
    self.change.hash(state);
  }
}

impl PartialEq<Self> for Row {
  fn eq(&self, other: &Self) -> bool {
    self.date == other.date && self.class == other.class && self.change == other.change
  }
}

impl Eq for Row {}
