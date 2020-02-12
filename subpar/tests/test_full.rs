extern crate subpar;

use chrono::NaiveDateTime;
use subpar::{ExcelObject, MetaWorkbook, SubparError, SubparTable};

/** Submission
 *
 *  An individual set of tissues submitted. This belongs to a case
 */
#[derive(Clone, Debug, SubparTable)]
pub struct Submission {
  #[subpar(rename = "Accession Number")]
  accession_number: String,
  #[subpar(rename = "Submitting Organization")]
  submitting_org: String,
  #[subpar(rename = "Service Requested")]
  service_type: String, //TODO: Convert to service enumeration
  #[subpar(rename = "Service Line Items")]
  service_line_items: Option<String>, // This is JSON, but easier to parse in ReasonML, so that's what I'm doing
  #[subpar(rename = "Species Submitted")]
  species: String,
  #[subpar(rename = "Pet name")]
  pet_name: Option<String>,
  #[subpar(rename = "Slides made")]
  slides: Option<i32>, //10
  // tissues: Option<i32>,
  // stained_slides: Option<String>,
  // stain_performed: Option<String>,
  // culture_performed: Option<String>,
  diagnosis: Option<String>,
  #[subpar(rename = "Additional Expenses")]
  expenses: Option<String>,
  #[subpar(rename = "Client cost")]
  price: Option<f64>,
  // formula: Option<String>,
  #[subpar(rename = "Invoice Number")]
  invoice_number: Option<i32>,
  #[subpar(rename = "Date Received")]
  received: NaiveDateTime, // TODO: Convert this to a chrono::datetime
  #[subpar(rename = "Date Finalized")]
  finalized: Option<NaiveDateTime>,
  #[subpar(rename = "Billed")]
  billed_on: Option<NaiveDateTime>,
  #[subpar(rename = "Paid")]
  paid_on: Option<NaiveDateTime>,
  // payment_notes: Option<String>,
}

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, SubparTable)]
pub struct DB {
  // pub sent_messages: Vec<SentMessage>,
  pub submissions: Vec<Submission>,
  // #[subpar(rename="payments")]
  // pub payment: Vec<Payment>,
}

// Convert a cell to a json string
pub fn cell_csv_to_vec(wrapped: &ExcelObject) -> Result<Vec<String>, SubparError> {
  let row = wrapped
    .unwrap_row()
    .expect("There was an error unwrapping the object in cell_csv_to_vec");
  match row.get("payment") {
    None => panic!("No cell named payment in the row"),
    Some(cell) => match &cell.data {
      subpar::CellType::String(value) => {
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
      x => panic!("The cell value must be a string. Received {:#?}", x),
    },
  }
}

#[test]
fn test_ctx() {
  // Test Excel
  let excel_config = subpar::WorkbookConfig::new_excel_config(
    "/home/dfogelson/Downloads/Submissions_Log_2019-07.xlsx".to_string(),
  );
  // subpar::WorkbookConfig::new_excel_config("../subpar_test/data/test_db.xlsx".to_string());
  let wb = subpar::Workbook::open(&excel_config).expect("Failed opening the excel workbook");

  let db = DB::from_excel(&ExcelObject::Workbook(wb));
  println!("db:\n{:#?}", db);

  // Test Sheets
  // let sheets_config = subpar::WorkbookConfig::new_sheets_config(
  //   "1kwQgjicMgKVV1aZ1oStIjpahQLDronaqzkTKdD-paI0",
  //   "/home/dfogelson/FishheadLabs/TheProcessFoundry/service_acct.json".to_string(),
  // );
  // let wb = SubparWorkbook::open(sheets_config);

  // let db = DB::load_workbook(wb);
  // println!("db:\n{:#?}", db);
}
