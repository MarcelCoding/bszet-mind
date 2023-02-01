use crate::api::AppError;
use crate::api::AppError::PlanUnavailable;
use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse};
use axum::{Extension, Json};
use bszet_davinci::timetable::Subject;
use bszet_davinci::Davinci;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::serde::format_description;
use time::Date;

format_description!(iso_date, Date, "[year]-[month]-[day]");

#[derive(Deserialize)]
pub(crate) struct PlanPath {
  #[serde(with = "iso_date")]
  date: Date,
}

#[derive(Deserialize)]
pub(crate) struct PlanQuery {
  class: String,
}

pub(crate) async fn html_plan(
  Extension(davinci): Extension<Arc<Davinci>>,
  Path(PlanPath { date }): Path<PlanPath>,
  Query(PlanQuery { class }): Query<PlanQuery>,
) -> Result<impl IntoResponse, AppError> {
  let split = class.split(',').collect::<Vec<&str>>();
  Ok(Html(
    davinci
      .get_html(&date, split.as_slice())
      .await?
      .ok_or(PlanUnavailable)?,
  ))
}

#[derive(Deserialize)]
pub(crate) struct TimetablePath {
  #[serde(with = "iso_date")]
  date: Date,
  class: String,
}

#[derive(Clone, Debug, Serialize)]
struct Lesson {
  pub lesson: u8,
  pub subject: String,
  pub iteration: Option<u8>,
  pub place: Option<String>,
  pub notice: Option<String>,
  pub cancel: bool,
}

pub(crate) async fn timetable(
  Extension(davinci): Extension<Arc<Davinci>>,
  Path(TimetablePath { date, .. }): Path<TimetablePath>,
) -> Result<impl IntoResponse, AppError> {
  Ok(Json(
    davinci
      .get_applied_timetable(date)
      .await
      .0
      .into_iter()
      .map(|lesson| {
        let (subject, cancel) = match lesson.subject {
          Subject::Cancel(subject) => (*subject, true),
          subject => (subject, false),
        };

        Lesson {
          lesson: lesson.lesson,
          subject: format!("{subject}"),
          iteration: lesson.iteration,
          place: lesson.place,
          notice: lesson.notice,
          cancel,
        }
      })
      .collect::<Vec<Lesson>>(),
  ))
}
