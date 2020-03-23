#[cfg(test)]
extern crate subpar;

use log::debug;

use chrono::NaiveDateTime;
use subpar::{ExcelObject, MetaWorkbook, SubparError, SubparTable};

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
  #[subpar(rename = "Invoice number")]
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

#[derive(Clone, Debug, SubparTable)]
pub struct Invoice {
  #[subpar(rename = "Invoice ID")]
  guid: String,
  #[subpar(rename = "Organization")]
  organization: String,
  #[subpar(rename = "Submissions")]
  submissions: String,
  #[subpar(rename = "Date Created")]
  created_on: NaiveDateTime,
}

impl Invoice {
  fn get_key_hash(&self) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("guid".to_string(), self.guid.clone());
    map
  }
}

#[derive(Debug, Clone, SubparTable)]
pub struct DB {
  // pub sent_messages: Vec<SentMessage>,
  pub submissions: Vec<Submission>,
  pub invoices: Vec<Invoice>,
  // #[subpar(rename="payments")]
  // pub payment: Vec<Payment>,
}

impl DB {
  pub fn upsert_invoice(workbook: &subpar::Workbook, invoice: &Invoice) -> Result<(), SubparError> {
    let key = invoice.get_key_hash();
    debug!("Upsert Key: {:#?}", key);
    // Make DataFilter call

    debug!("The Workbook: {:#?}", workbook);

    // match exists {
    // Some(existing) => Update call
    // None => Append Call
    // }
    unimplemented!()
  }
}

impl std::fmt::Display for DB {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\tsubmissions: {}\n\tinvoices: {}",
      self.submissions.len(),
      self.invoices.len()
    )
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
  println!("db submissions:\n{:#?}", db.unwrap().submissions.len());
}

#[test]
fn test_sheets() {
  env_logger::init();
  // Read the submissions tab of the log using subpar
  let sheet_id = String::from("1kwQgjicMgKVV1aZ1oStIjpahQLDronaqzkTKdD-paI0");
  let db_conf = subpar::WorkbookConfig::new_sheets_config(
    Some(sheet_id.clone()),
    "/home/dfogelson/FishheadLabs/subpar/subpar_test/data/service_acct.json".to_string(),
    "fhl@landfillinc.com".to_string(),
  );

  debug!("db_conf:\n{:#?}", db_conf);

  let wb = subpar::Workbook::open(&db_conf).expect("Failed opening the google sheets workbook");
  // let db = DB::from_excel(&ExcelObject::Workbook(wb.clone())).unwrap();
  // println!("Done loading DB:\n{}", db);

  // append a new invoice
  let mut invoice = Invoice {
    guid: "Live Append Test".to_string(),
    organization: "FHL".to_string(),
    submissions: "[\"F20-4\"]".to_string(),
    created_on: chrono::Utc::now().naive_utc(),
  };

  let result = DB::upsert_invoice(&wb, &invoice);
  debug!("The result of the append: {:#?}", result);

  // update the invoice
  invoice.submissions = "[\"F20-6\"]".to_string();
  let result = DB::upsert_invoice(&wb, &invoice);
  debug!("The result of the update: {:#?}", result);

  // reread the database

  // let db = DB::load_db().expect("Failed to open the db");

  // debug!("db:\n{:#?}", db);
}
