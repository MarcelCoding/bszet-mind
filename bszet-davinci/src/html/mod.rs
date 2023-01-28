use sailfish::TemplateOnce;
use time::Date;

#[derive(TemplateOnce)]
#[template(path = "plan.stpl", rm_whitespace = true)]
pub(crate) struct SubstitutionPlanTemplate<'a> {
  pub(crate) date: Date,
  pub(crate) table: Vec<&'a [String]>,
  pub(crate) classes: &'a [&'a str],
}

#[cfg(test)]
mod test {
  use sailfish::TemplateOnce;
  use time::Date;
  use time::Month::January;

  use crate::html::SubstitutionPlanTemplate;

  #[test]
  fn test_template() -> anyhow::Result<()> {
    let a = vec![
      "1".to_string(),
      "1".to_string(),
      "1".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
    ];
    let b = vec![
      "IGD 21".to_string(),
      "1".to_string(),
      "1".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
    ];
    let c = vec![
      "IGD21".to_string(),
      "1".to_string(),
      "1".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
    ];

    let table = vec![a.as_slice(), b.as_slice(), c.as_slice(), a.as_slice()];

    let classes = vec!["IGD 21".to_string(), "IGD21".to_string()];

    let template = SubstitutionPlanTemplate {
      date: Date::from_calendar_date(2023, January, 28)?,
      table,
      classes: classes.as_slice(),
    };

    println!("{}", template.render_once()?);

    Ok(())
  }
}
