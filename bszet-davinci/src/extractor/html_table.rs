use select::document::Document;
use select::predicate::Name;

use crate::extractor::clean;

pub(crate) fn extract_html_table(doc: &Document) -> Vec<Vec<String>> {
  let mut rows = Vec::new();

  for row in doc.find(Name("tr")) {
    let mut columns = Vec::new();

    for data in row.find(Name("td")) {
      columns.push(clean(&data.text()).to_string());
    }

    if !columns.is_empty() {
      rows.push(columns);
    }
  }

  rows
}
