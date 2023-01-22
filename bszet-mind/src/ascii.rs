use std::fmt::Write;

use bszet_davinci::timetable::Lesson;

pub fn table(day: Vec<Lesson>) -> String {
  let mut lesson_w = 0;
  let mut subject_w = 0;
  let mut place_w = 0;

  for lesson in &day {
    let l = format!("{}", lesson.lesson);
    let s = format!("{}", lesson.subject);
    let p = &lesson.place;

    lesson_w = lesson_w.max(l.len());
    subject_w = subject_w.max(s.len());
    place_w = place_w.max(p.len());
  }

  let mut out = String::with_capacity(day.len() * (lesson_w + subject_w + place_w + 2));

  let mut first = true;

  for lesson in &day {
    let l = format!("{}", lesson.lesson);
    let s = format!("{}", lesson.subject);
    let p = &lesson.place;

    if first {
      first = false;
    } else {
      writeln!(out).unwrap();
    }

    if let Some(notice) = &lesson.notice {
      write!(
        out,
        "{}{} {}{} {}{} {}",
        l,
        " ".repeat(lesson_w - l.len()),
        s,
        " ".repeat(subject_w - s.len()),
        p,
        " ".repeat(place_w - p.len()),
        notice
      )
      .unwrap();
    } else {
      write!(
        out,
        "{}{} {}{} {}",
        l,
        " ".repeat(lesson_w - l.len()),
        s,
        " ".repeat(subject_w - s.len()),
        p
      )
      .unwrap();
    }
  }

  out
}
