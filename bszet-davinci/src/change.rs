use anyhow::anyhow;

use crate::timetable::{Lesson, Subject};
use crate::REPLACEMENT_REGEX;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Change {
  Cancel {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: String,
    notice: Option<String>,
  },
  PlaceChange {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: Replacement<String>,
    notice: Option<String>,
  },
  Addition {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: Option<String>,
    notice: Option<String>,
  },
  Replacement {
    lesson: u8,
    subject: Replacement<Subject>,
    teachers: Replacement<Vec<String>>,
    place: Replacement<String>,
    notice: Option<String>,
  },
  ClassIsMissing {
    lesson: u8,
    subject: Subject,
    teachers: Vec<String>,
    place: String,
    notice: Option<String>,
  },
  Other {
    lesson: u8,
    value: String,
    subject: Subject,
    teachers: Vec<String>,
    place: String,
    notice: Option<String>,
  },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Replacement<T> {
  pub from: T,
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
      "Fällt aus" => Self::Cancel {
        lesson,
        subject: subject.into(),
        place,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice,
      },
      "Raumänderung" => Self::PlaceChange {
        lesson,
        subject: subject.into(),
        place: place.as_str().try_into()?,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice,
      },
      "Zusatzunterricht" => Self::Addition {
        lesson,
        subject: subject.into(),
        place: if place.is_empty() { None } else { Some(place) },
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice,
      },
      "Vertreten" => Self::Replacement {
        lesson,
        subject: subject.try_into()?,
        place: place.as_str().try_into()?,
        teachers: teacher.try_into()?,
        notice,
      },
      "Klasse fehlt" => Self::ClassIsMissing {
        lesson,
        subject: subject.into(),
        place,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice,
      },
      value => Self::Other {
        lesson,
        value: value.to_string(),
        subject: subject.into(),
        place,
        teachers: teacher
          .split(',')
          .map(|s| s.trim().to_string())
          .collect::<Vec<String>>(),
        notice,
      },
    })
  }

  /// Apples the change for the provided day.
  pub(crate) fn apply(&self, lessons: &mut Vec<Lesson>) -> bool {
    match self {
      Change::Cancel {
        lesson,
        subject,
        notice,
        ..
      }
      | Change::ClassIsMissing {
        lesson,
        subject,
        notice,
        ..
      } => {
        match find_lesson(lessons, lesson, subject) {
          None => false,
          Some(lesson) => {
            // TODO: place, teachers
            lesson.subject = Subject::Cancel(Box::new(subject.clone()));
            lesson.notice = notice.as_ref().map(|string| string.to_string());
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
        match find_lesson(lessons, lesson, subject) {
          None => false,
          Some(lesson) => {
            // TODO: teachers, place.from
            lesson.place = Some(place.to.to_string());
            lesson.notice = notice.as_ref().map(|string| string.to_string());
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
          notice: notice.as_ref().map(|string| string.to_string()),
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
        match find_lesson(lessons, lesson, &subject.from) {
          None => false,
          Some(lesson) => {
            // TODO: teachers, place.from
            lesson.subject = subject.to.clone();
            lesson.place = Some(place.to.to_string());
            lesson.notice = notice.as_ref().map(|string| string.to_string());
            true
          }
        }
      }
      Change::Other { .. } => false,
    }
  }

  pub(crate) fn lesson(&self) -> u8 {
    match self {
      Change::Cancel { lesson, .. } => *lesson,
      Change::PlaceChange { lesson, .. } => *lesson,
      Change::Addition { lesson, .. } => *lesson,
      Change::Replacement { lesson, .. } => *lesson,
      Change::ClassIsMissing { lesson, .. } => *lesson,
      Change::Other { lesson, .. } => *lesson,
    }
  }
}

fn find_lesson<'a>(
  lessons: &'a mut [Lesson],
  lesson: &u8,
  subject: &Subject,
) -> Option<&'a mut Lesson> {
  lessons
    .iter_mut()
    .find(|l| &l.lesson == lesson && &l.subject == subject)
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

impl TryFrom<&str> for Replacement<Vec<String>> {
  type Error = anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let replacement: Replacement<String> = TryFrom::try_from(value)?;
    Ok(Replacement {
      from: replacement
        .from
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<String>>(),
      to: replacement
        .to
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<String>>(),
    })
  }
}
