extern crate calamine;
extern crate chrono;
extern crate subpar;

use calamine::DataType;
use chrono::NaiveDateTime;
use subpar::{ExcelIO, ExcelObject, MetaWorkbook, SubparError};

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, ExcelIO)]
pub struct Payment {
  guid: String,
  #[subpar(parser = "cell_json_to_vec")]
  strings: Vec<String>,
  // payer: String,
  // payee: String,
  // method: String,
  // amount: f64,
  // comment: Option<String>,
  // date_received: NaiveDateTime,
}

#[derive(Clone, Debug, ExcelIO)]
pub struct Submission {
  accession_number: String,
  submitting_org: String,
  service_type: String,               //TODO: Convert to service enumeration
  service_line_items: Option<String>, // This is JSON, but easier to parse in ReasonML, so that's what I'm doing
  species: String,
  pet_name: Option<String>,
  slides: Option<i32>, //10
  // tissues: Option<i32>,
  // stained_slides: Option<String>,
  // stain_performed: Option<String>,
  // culture_performed: Option<String>,
  diagnosis: Option<String>,
  expenses: Option<String>,
  price: Option<f64>,
  // formula: Option<String>,
  invoice_number: Option<i32>,
  received: NaiveDateTime, // TODO: Convert this to a chrono::datetime
  finalized: Option<NaiveDateTime>,
  billed_on: Option<NaiveDateTime>,
  paid_on: Option<NaiveDateTime>,
  // payment_notes: Option<String>,
}

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, ExcelIO)]
pub struct DB {
  pub submissions: Vec<Submission>,
  #[subpar(rename = "payments")]
  pub payment: Vec<Payment>,
}

pub fn cell_json_to_vec(wrapped: &ExcelObject) -> Result<Vec<String>, SubparError> {
  let row = wrapped
    .unwrap_row()
    .expect("There was an error unwrapping the object in cell_csv_to_vec");
  match row.get("payment") {
    None => panic!(format!(
      "Could not find a column named payment in cell_csv_to_vec"
    )),
    Some(DataType::String(value)) => {
      println!("The cell is: {:#?}", value);
      Ok(
        value
          .trim_matches(|c| c == '[' || c == ']')
          .replace(" ", "")
          .split(",")
          .filter(|&x| x != "")
          .map(|s| s.to_string())
          .collect::<Vec<_>>(),
      )
    }
    Some(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Vec<String> for Payment",
      x
    )),
  }
}

#[test]
fn test_ctx() {
  let wb = MetaWorkbook::new("../subpar_test/data/DataLog.xlsx".to_string());
  let db = DB::from_excel(&ExcelObject::Workbook(wb));
  println!("db:\n{:#?}", db);
}

// #[test]
// fn test_writer() {
//   subpar::writer::create_workbook()
// }
