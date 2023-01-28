use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};

use time::Weekday;
use tracing::warn;

pub mod igd21;

#[derive(Clone, Debug)]
pub struct Lesson {
  pub lesson: u8,
  pub subject: Subject,
  pub iteration: Option<u8>,
  pub place: Option<String>,
  pub notice: Option<String>,
}

type Day = Vec<Lesson>;

type Timetable = HashMap<Weekday, Day>;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Subject {
  GermanBasic,
  GermanAdvanced,
  MathBasic,
  MathAdvanced,
  EnglishBasic,
  EnglishAdvanced,
  Art,
  History,
  French,
  Ethics,
  Russian,
  Chemistry,
  Physics,
  /// PE
  PhysicalEducation,
  Literature,

  Lf6_7_9,
  Lf8,
  Lf10,
  Lf11,

  FaeVerb,
  None,

  Cancel(Box<Subject>),
  Other(String),
}

impl From<&str> for Subject {
  fn from(value: &str) -> Self {
    match value {
      "DEU" => Self::GermanBasic,
      "LK-DEU" => Self::GermanAdvanced,
      "MA" => Self::MathBasic,
      "LK-MA" => Self::MathAdvanced,
      "ENG" => Self::EnglishBasic,
      "LK-ENG" => Self::EnglishAdvanced,
      "BK" => Self::Art,
      "GGK" => Self::History,
      "F-B" => Self::French,
      "ETH" => Self::Ethics,
      "R-B" => Self::Russian,
      "CH" => Self::Chemistry,
      "PH" => Self::Physics,
      "SP" => Self::PhysicalEducation,
      "LIT" => Self::Literature,

      "LF 6+7+9" => Self::Lf6_7_9,
      "LF 8" => Self::Lf8,
      "LF 10" => Self::Lf10,
      "LF 11" => Self::Lf11,

      "_fä.verb." => Self::FaeVerb,
      "" => Self::None,
      other => Self::Other(other.to_string()),
    }
  }
}

impl Display for Subject {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::GermanBasic => f.write_str("D"),
      Self::GermanAdvanced => f.write_str("LK-D"),
      Self::MathBasic => f.write_str("Ma"),
      Self::MathAdvanced => f.write_str("LK-Ma"),
      Self::EnglishBasic => f.write_str("En"),
      Self::EnglishAdvanced => f.write_str("LK-En"),
      Self::Art => f.write_str("BK"),
      Self::History => f.write_str("Ge"),
      Self::French => f.write_str("Frz"),
      Self::Ethics => f.write_str("Eth"),
      Self::Russian => f.write_str("Ru"),
      Self::Chemistry => f.write_str("Ch"),
      Self::Physics => f.write_str("Ph"),
      Self::PhysicalEducation => f.write_str("Sp"),
      Self::Literature => f.write_str("Lit"),
      Self::Lf6_7_9 => f.write_str("LF 6+7+9"),
      Self::Lf8 => f.write_str("LF 8"),
      Self::Lf10 => f.write_str("LF 10"),
      Self::Lf11 => f.write_str("LF 11"),
      Self::FaeVerb => f.write_str("Fä-Verb"),
      Self::None => f.write_str("None"),
      Self::Cancel(inner) => {
        f.write_char('(')?;
        Display::fmt(inner, f)?;
        f.write_char(')')
      }
      Self::Other(other) => {
        warn!("Unknown subject: {}", other);
        f.write_str(other)
      }
    }
  }
}

impl Lesson {
  pub fn new(lesson: u8, iteration: Option<u8>, subject: Subject, place: &str) -> Self {
    Self {
      lesson,
      iteration,
      subject,
      place: Some(place.to_string()),
      notice: None,
    }
  }
}
