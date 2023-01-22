extern crate core;

use std::str::FromStr;

use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::LAST_MODIFIED;
use reqwest::{Client, Url};
use select::document::Document;
use select::predicate::Name;
use time::format_description::well_known::Rfc2822;
use time::{Date, Month, OffsetDateTime};
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing::info;

use crate::timetable::igd21::IGD21;
use crate::timetable::{Lesson, Subject};

const DATE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("\\S+ (\\d{2})\\.(\\d{2})\\.(\\d{4})").unwrap());
const REPLACEMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\+(.*) \\((.+)\\)").unwrap());

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
  pub rows: Vec<Row>,
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

  pub async fn get_applied_timetable(&self, date: Date) -> Vec<Lesson> {
    let mut day = IGD21.get(&date.weekday()).unwrap().clone();

    if let Some(data) = self.data.read().await.as_ref() {
      for row in &data.rows {
        if row.date != date || !row.class.contains(&"IGD21".to_string()) {
          continue;
        }

        for mut lesson in &mut day {
          if lesson.lesson == row.lesson {
            match &row.change {
              Change::Cancel {
                subject,
                teacher,
                place,
              } => {
                if &lesson.subject != subject {
                  continue;
                };
                // sanity checks??
                lesson.subject = Subject::Cancel(Box::new(lesson.subject.clone()));
                lesson.notice = row.notice.clone();
              }
              Change::PlaceChange {
                subject,
                teacher,
                place,
              } => {
                if &lesson.subject != subject {
                  continue;
                };
                // sanity checks?? place.from
                lesson.place = place.to.clone();
                lesson.notice = row.notice.clone();
              }
              Change::Addition {
                subject,
                teacher,
                place,
              } => {
                day.push(Lesson {
                  lesson: row.lesson,
                  subject: subject.clone(),
                  iteration: None,
                  place: place.clone(),
                  notice: row.notice.clone(),
                });
              }
              Change::Replacement {
                subject,
                teacher,
                place,
              } => {
                if lesson.subject != subject.from {
                  continue;
                };
                // sanity checks?? place.from

                lesson.subject = subject.to.clone();
                lesson.place = place.to.clone();
                lesson.notice = row.notice.clone();
              }
              Change::Other {
                value,
                subject,
                teacher,
                place,
              } => {
                lesson.notice = Some(match &row.notice {
                  None => format!("Other: {}", value),
                  Some(notice) => format!("Other: {} - {}", value, notice),
                });
              }
            }
            break;
          }
        }

        break;
      }
    }

    day
  }

  pub async fn update(&self) -> anyhow::Result<bool> {
    let mut url = self.entrypoint.clone();
    let mut rows = Vec::new();

    loop {
      match self.fetch(url, &mut rows).await? {
        None => break,
        Some(next) => url = next,
      };
    }

    let now = OffsetDateTime::now_utc();

    let mut data = self.data.write().await;

    // check if there is a difference
    if let Some(data) = data.as_mut() {
      if !rows.iter().zip(&data.rows).any(|(a, b)| a != b) {
        data.last_checked = now;
        return Ok(false);
      }
    }

    *data = Some(Data {
      last_checked: now,
      last_modified: now,
      rows,
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
          let column = text.trim();
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
        Some(u8::from_str(&columns[1][..columns[1].len() - 1]).unwrap())
      };
      let change = Change::new(
        &columns[5],
        columns[2].as_str(),
        columns[3].to_string(),
        columns[4].to_string(),
      )?;
      let notice = if columns[6].is_empty() {
        None
      } else {
        Some(columns[6].to_string())
      };

      let row = if let Some(last) = rows.last() {
        Row {
          date,
          class: class.unwrap_or_else(|| last.class.clone()),
          lesson: lesson.unwrap_or(last.lesson),
          change,
          notice,
        }
      } else {
        Row {
          date,
          class: class.expect("First row, can not have missing fields."),
          lesson: lesson.expect("First row, can not have missing fields."),
          change,
          notice,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
  pub date: Date,
  pub class: Vec<String>,
  pub lesson: u8,
  pub change: Change,
  pub notice: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
  Cancel {
    subject: Subject,
    teacher: String,
    place: String,
  },
  PlaceChange {
    subject: Subject,
    teacher: String,
    place: Replacement<String>,
  },
  Addition {
    subject: Subject,
    teacher: String,
    place: String,
  },
  Replacement {
    subject: Replacement<Subject>,
    place: Replacement<String>,
    teacher: Replacement<String>,
  },
  Other {
    value: String,
    subject: Subject,
    teacher: String,
    place: String,
  },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replacement<T> {
  pub from: T,
  pub to: T,
}

impl Change {
  fn new(value: &str, subject: &str, place: String, teacher: String) -> anyhow::Result<Self> {
    Ok(match value {
      "Fällt aus" => Self::Cancel {
        subject: subject.into(),
        place,
        teacher: teacher[1..teacher.len() - 1].to_string(),
      },
      "Raumänderung" => Self::PlaceChange {
        subject: subject.into(),
        place: place.as_str().try_into()?,
        teacher,
      },
      "Zusatzunterricht" => Self::Addition {
        subject: subject.into(),
        place,
        teacher,
      },
      "Vertreten" => Self::Replacement {
        subject: subject.try_into()?,
        place: place.as_str().try_into()?,
        teacher: teacher.as_str().try_into()?,
      },
      value => Self::Other {
        value: value.to_string(),
        subject: subject.into(),
        place,
        teacher,
      },
    })
  }
}

impl TryFrom<&str> for Replacement<String> {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match REPLACEMENT_REGEX
      .captures(value)
      .map(|capture| (capture.get(1), capture.get(2)))
    {
      Some((Some(to), Some(from))) => Ok(Replacement {
        from: from.as_str().to_string(),
        to: to.as_str().to_string(),
      }),
      _ => Err(anyhow!("can not parse replacement {}", value)),
    }
  }
}

impl TryFrom<&str> for Replacement<Subject> {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let replacement: Replacement<String> = TryFrom::try_from(value)?;
    Ok(Replacement {
      from: Subject::from(replacement.from.as_str()),
      to: Subject::from(replacement.to.as_str()),
    })
  }
}
