use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::timetable::{Lesson, Subject};
use crate::REPLACEMENT_REGEX;

static MOVED_FROM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("Von .+ verschoben").unwrap());
static MOVED_TO_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("Auf .+ verschoben").unwrap());

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Change {
  Cancel {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: String,
    notice: String,
  },
  PlaceChange {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: Replacement<String>,
    notice: String,
  },
  Addition {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: Option<String>,
    notice: String,
  },
  Replacement {
    lesson: u8,
    subject: Replacement<Subject>,
    teachers: Replacement<Vec<String>>,
    place: Replacement<String>,
    notice: String,
  },
  Other {
    lesson: u8,
    value: String,
    subject: Subject,
    teachers: Vec<String>,
    place: String,
    notice: String,
  },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Replacement<T> {
  pub from: Option<T>,
  pub to: T,
}

impl Change {
  pub(crate) fn new(
    lesson: u8,
    value: &str,
    subject: &str,
    place: String,
    teacher: &str,
    notice: Option<String>,
  ) -> anyhow::Result<Self> {
    Ok(match value {
      toc
        if ["Fällt aus", "Klasse fehlt", "Kurs fehlt"].contains(&toc)
          || MOVED_TO_REGEX.is_match(toc) =>
      {
        Self::Cancel {
          lesson,
          subject: subject.into(),
          place,
          teachers: teacher
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>(),
          notice: notice.unwrap_or(value.to_string()),
        }
      }
      "Raumänderung" => Self::PlaceChange {
        lesson,
        subject: subject.into(),
        place: place.as_str().try_into()?,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice: notice.unwrap_or(value.to_string()),
      },
      "Zusatzunterricht" => Self::Addition {
        lesson,
        subject: subject.into(),
        place: if place.is_empty() { None } else { Some(place) },
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice: notice.unwrap_or(value.to_string()),
      },
      toc if toc == "Vertreten" || MOVED_FROM_REGEX.is_match(toc) => Self::Replacement {
        lesson,
        subject: subject.try_into()?,
        place: place.as_str().try_into()?,
        teachers: teacher.try_into()?,
        notice: notice.unwrap_or(value.to_string()),
      },
      toc => Self::Other {
        lesson,
        value: toc.to_string(),
        subject: subject.into(),
        place,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice: notice.unwrap_or(value.to_string()),
      },
    })
  }

  /// Apples the change for the provided day.
  pub(crate) fn apply(&self, lessons: &mut Vec<Lesson>) -> anyhow::Result<bool> {
    Ok(match self {
      Change::Cancel {
        lesson,
        subject,
        notice,
        ..
      } => {
        match find_lesson(lessons, lesson, Some(subject), false)? {
          None => false,
          Some(lesson) => {
            // TODO: place, teachers
            lesson.subject = Subject::Cancel(Box::new(subject.clone()));
            lesson.notice = Some(notice.to_string());
            true
          }
        }
      }
      Change::PlaceChange {
        lesson,
        subject,
        place,
        notice,
        ..
      } => {
        match find_lesson(lessons, lesson, Some(subject), false)? {
          None => false,
          Some(lesson) => {
            // TODO: teachers, place.from
            lesson.place = Some(place.to.to_string());
            lesson.notice = Some(notice.to_string());
            true
          }
        }
      }
      Change::Addition {
        lesson,
        subject,
        place,
        notice,
        ..
      } => {
        // TODO: teachers
        lessons.push(Lesson {
          lesson: *lesson,
          subject: subject.clone(),
          iteration: None,
          place: place.as_ref().map(|string| string.to_string()),
          notice: Some(notice.to_string()),
        });
        true
      }
      Change::Replacement {
        lesson,
        subject,
        place,
        notice,
        ..
      } => {
        match find_lesson(lessons, lesson, subject.from.as_ref(), true)? {
          None => false,
          Some(lesson) => {
            // TODO: teachers, place.from
            lesson.subject = subject.to.clone();
            lesson.place = Some(place.to.to_string());
            lesson.notice = Some(notice.to_string());
            true
          }
        }
      }
      Change::Other { .. } => false,
    })
  }

  pub(crate) fn lesson(&self) -> u8 {
    match self {
      Change::Cancel { lesson, .. } => *lesson,
      Change::PlaceChange { lesson, .. } => *lesson,
      Change::Addition { lesson, .. } => *lesson,
      Change::Replacement { lesson, .. } => *lesson,
      Change::Other { lesson, .. } => *lesson,
    }
  }
}

fn find_lesson<'a>(
  lessons: &'a mut [Lesson],
  lesson: &u8,
  subject: Option<&Subject>,
  allow_cancel: bool,
) -> anyhow::Result<Option<&'a mut Lesson>> {
  let lessons = lessons
    .iter_mut()
    .filter(|l| &l.lesson == lesson)
    .collect::<Vec<&mut Lesson>>();

  match subject {
    None => {
      if lessons.len() == 1 {
        Ok(lessons.into_iter().next())
      } else {
        Err(anyhow!("Found multiple subjects without original value to clearly identify lesson. less: {}, subjs: {:?}",lesson , lessons))
      }
    }
    Some(subject) => Ok(lessons.into_iter().find(|lesson| {
      if &lesson.subject == subject {
        true
      } else if allow_cancel {
        if let Subject::Cancel(inner) = &lesson.subject {
          inner.as_ref() == subject
        } else {
          false
        }
      } else {
        false
      }
    })),
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
        from: Some(from.as_str().to_string()),
        to: to.as_str().to_string(),
      }),
      _ => Ok(Replacement {
        from: None,
        to: value.to_string(),
      }),
    }
  }
}

impl TryFrom<&str> for Replacement<Subject> {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let replacement: Replacement<String> = TryFrom::try_from(value)?;
    Ok(Replacement {
      from: replacement.from.map(|value| Subject::from(value.as_str())),
      to: Subject::from(replacement.to.as_str()),
    })
  }
}

impl TryFrom<&str> for Replacement<Vec<String>> {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let replacement: Replacement<String> = TryFrom::try_from(value)?;
    Ok(Replacement {
      from: replacement.from.map(|value| {
        value
          .split(',')
          .map(|value| value.trim().to_string())
          .collect::<Vec<String>>()
      }),
      to: replacement
        .to
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<String>>(),
    })
  }
}
