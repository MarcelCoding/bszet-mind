use std::collections::HashMap;

use once_cell::sync::Lazy;
use time::Weekday::{Friday, Monday, Thursday, Tuesday, Wednesday};

use crate::timetable::Subject::{
  Art, Chemistry, EnglishAdvanced, EnglishBasic, Ethics, French, GermanBasic, History, Lf10__1,
  Lf10__2, Lf11, Lf11__1, Lf11__2, Lf13__1, Lf13__2, Lf9_12, Literature, MathAdvanced, MathBasic,
  PhysicalEducation, Physics, Russian,
};
use crate::timetable::{Lesson, Timetable};

/// https://geschuetzt.bszet.de/s-lk-vw/Stundenplaene/DuBAS/IGD%2021.pdf
pub static IGD21: Lazy<Timetable> = Lazy::new(|| {
  HashMap::from([
    (
      Monday,
      vec![
        Lesson::new(1, None, GermanBasic, "B6"),
        Lesson::new(2, None, Chemistry, "B9"),
        Lesson::new(3, Some(1), EnglishAdvanced, "A102"),
        Lesson::new(3, Some(1), MathAdvanced, "B11"),
        Lesson::new(3, Some(2), Lf11, "B5"),
        Lesson::new(4, Some(1), History, "B4"),
        Lesson::new(4, Some(2), Art, "A06"),
        Lesson::new(4, Some(2), Literature, "B4"),
      ],
    ),
    (
      Tuesday,
      vec![
        Lesson::new(1, None, MathBasic, "B05"),
        Lesson::new(1, None, EnglishBasic, "B104"),
        Lesson::new(2, None, GermanBasic, "B6"),
        Lesson::new(3, None, Lf13__1, "A103"),
        Lesson::new(3, None, Lf11__2, "B3"),
        Lesson::new(4, None, EnglishAdvanced, "A102"),
        Lesson::new(4, None, MathAdvanced, "B11"),
      ],
    ),
    (
      Wednesday,
      vec![
        Lesson::new(1, None, EnglishAdvanced, "B6"),
        Lesson::new(1, None, MathAdvanced, "B11"),
        Lesson::new(2, None, EnglishBasic, "B105"),
        Lesson::new(2, None, MathBasic, "B106"),
        Lesson::new(3, None, Ethics, "B4"),
        Lesson::new(4, None, French, "B111"),
        Lesson::new(4, None, Russian, "B4"),
      ],
    ),
    (
      Thursday,
      vec![
        Lesson::new(1, None, History, "B4"),
        Lesson::new(2, None, Lf9_12, "B8"),
        Lesson::new(3, None, PhysicalEducation, "117.GS Neu"),
        Lesson::new(4, None, Lf10__1, "B405"),
        Lesson::new(4, None, Lf13__2, "A103"),
      ],
    ),
    (
      Friday,
      vec![
        Lesson::new(1, None, Physics, "B112"),
        Lesson::new(2, None, French, "A102"),
        Lesson::new(2, None, Russian, "B4"),
        Lesson::new(3, None, Lf13__1, "A103"),
        Lesson::new(3, None, Lf10__2, "B405"),
        Lesson::new(4, None, Lf11__1, "B5"),
        Lesson::new(4, None, Lf13__2, "A103"),
      ],
    ),
  ])
});
