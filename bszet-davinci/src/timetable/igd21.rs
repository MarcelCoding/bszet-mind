use std::collections::HashMap;

use once_cell::sync::Lazy;
use time::Weekday::{Friday, Monday, Thursday, Tuesday, Wednesday};

use crate::timetable::Subject::{
  Art, Chemistry, EnglishAdvanced, EnglishBasic, Ethics, French, GermanBasic, History, Lf10, Lf11,
  Lf6_7_9, Lf8, Literature, MathAdvanced, MathBasic, PhysicalEducation, Physics, Russian,
};
use crate::timetable::{Lesson, Timetable};

/// https://geschuetzt.bszet.de/s-lk-vw/Stundenplaene/DuBAS/IGD%2021.pdf
pub const IGD21: Lazy<Timetable> = Lazy::new(|| {
  HashMap::from([
    (
      Monday,
      vec![
        Lesson::new(1, None, MathBasic, "B10".to_string()),
        Lesson::new(2, None, Lf6_7_9, "B8".to_string()),
        Lesson::new(3, Some(1), History, "B4".to_string()),
        Lesson::new(3, Some(2), EnglishAdvanced, "B03".to_string()),
        Lesson::new(3, Some(2), MathAdvanced, "B11".to_string()),
        Lesson::new(4, None, Lf11, "B5".to_string()), // group 1
        Lesson::new(4, None, Lf8, "B405".to_string()), // group 2
        Lesson::new(5, Some(1), EnglishBasic, "B03".to_string()),
        Lesson::new(5, Some(2), Art, "A06".to_string()),
      ],
    ),
    (
      Tuesday,
      vec![
        Lesson::new(1, None, Lf6_7_9, "B8".to_string()),
        Lesson::new(2, None, GermanBasic, "B6".to_string()),
        Lesson::new(3, None, History, "B8".to_string()),
        Lesson::new(4, None, Ethics, "B112".to_string()),
      ],
    ),
    (
      Wednesday,
      vec![
        Lesson::new(1, None, French, "A102".to_string()),
        Lesson::new(1, None, Russian, "B4".to_string()),
        Lesson::new(2, None, Chemistry, "B9".to_string()),
        Lesson::new(3, None, GermanBasic, "B6".to_string()),
        Lesson::new(4, None, EnglishAdvanced, "B03".to_string()),
        Lesson::new(4, None, MathAdvanced, "B11".to_string()),
        Lesson::new(5, Some(2), Literature, "B4".to_string()),
      ],
    ),
    (
      Thursday,
      vec![
        Lesson::new(1, None, Physics, "B106".to_string()),
        Lesson::new(2, None, Lf8, "B5".to_string()), // group 1
        Lesson::new(2, None, Lf11, "B405".to_string()), // group 2
        Lesson::new(3, None, EnglishBasic, "B05".to_string()),
        Lesson::new(3, None, MathBasic, "B11".to_string()),
        Lesson::new(4, None, PhysicalEducation, "117.GS".to_string()),
      ],
    ),
    (
      Friday,
      vec![
        Lesson::new(1, None, Lf10, "B405".to_string()), // group: 1
        Lesson::new(2, None, French, "A102".to_string()),
        Lesson::new(2, None, Russian, "B4".to_string()),
        Lesson::new(3, None, EnglishAdvanced, "B6".to_string()),
        Lesson::new(3, None, MathAdvanced, "B11".to_string()),
        Lesson::new(4, None, Lf8, "B3".to_string()), // group: 1
        Lesson::new(4, None, Lf11, "B5".to_string()), // group: 2
      ],
    ),
  ])
});
