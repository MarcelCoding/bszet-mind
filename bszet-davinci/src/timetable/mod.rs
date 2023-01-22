use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};

use time::Weekday;

pub mod igd21;

#[derive(Clone, Debug)]
pub struct Lesson {
    pub lesson: u8,
    pub subject: Subject,
    pub iteration: Option<u8>,
    pub place: String,
    pub notice: Option<String>,
}

type Day = Vec<Lesson>;

type Timetable = HashMap<Weekday, Day>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Subject {
    GermanBasic,
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

    Cancel(Box<Subject>),
    Other(String),
}

impl From<&str> for Subject {
    fn from(value: &str) -> Self {
        match value {
            "GK-DEU" => Self::GermanBasic,
            "GK-MA" => Self::MathBasic,
            "LK-Ma" => Self::MathAdvanced,
            "GK-ENG" => Self::EnglishBasic,
            "LK-ENG" => Self::EnglishAdvanced,
            "BK" => Self::Art,
            "GE/GGK" => Self::History,
            "FR" => Self::French,
            "DEU" => Self::GermanBasic,
            "ETH" => Self::Ethics,
            "RU" => Self::Russian,
            "CH" => Self::Chemistry,
            "PH" => Self::Physics,
            "SP" => Self::PhysicalEducation,
            "LIT" => Self::Literature,

            "LF 6+7+9" => Self::Lf6_7_9,
            "LF 8" => Self::Lf8,
            "LF 10" => Self::Lf10,
            "LF 11" => Self::Lf11,

            other => Self::Other(other.to_string()),
        }
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::GermanBasic => f.write_str("D"),
            Subject::MathBasic => f.write_str("GK-Ma"),
            Subject::MathAdvanced => f.write_str("LK-Ma"),
            Subject::EnglishBasic => f.write_str("GK-En"),
            Subject::EnglishAdvanced => f.write_str("LK-En"),
            Subject::Art => f.write_str("BK"),
            Subject::History => f.write_str("Ge"),
            Subject::French => f.write_str("Frz"),
            Subject::Ethics => f.write_str("Eth"),
            Subject::Russian => f.write_str("Ru"),
            Subject::Chemistry => f.write_str("Ch"),
            Subject::Physics => f.write_str("Ph"),
            Subject::PhysicalEducation => f.write_str("Sp"),
            Subject::Literature => f.write_str("Lit"),
            Subject::Lf6_7_9 => f.write_str("LF 6+7+9"),
            Subject::Lf8 => f.write_str("LF 8"),
            Subject::Lf10 => f.write_str("LF 10"),
            Subject::Lf11 => f.write_str("LF 11"),
            Subject::Cancel(inner) => {
                f.write_char('(')?;
                Display::fmt(inner, f)?;
                f.write_char(')')
            } // TODO: don't create a new formatter
            Subject::Other(other) => f.write_str(other),
        }
    }
}

impl Lesson {
    pub fn new(lesson: u8, iteration: Option<u8>, subject: Subject, place: String) -> Self {
        Self {
            lesson,
            iteration,
            subject,
            place,
            notice: None,
        }
    }
}
