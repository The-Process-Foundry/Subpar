#[cfg(test)]
extern crate subpar;

use log::debug;

use chrono::NaiveDateTime;
use subpar::{to_row, ExcelObject, MetaWorkbook, SheetMetadata, SubparError, SubparTable};

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

  // Use the key hash to create a standardized key string
  fn get_key_string(&self) -> String {
    let mut result = "".to_string();

    let hash = self.get_key_hash();
    let mut keys: Vec<String> = hash.keys().into_iter().map(|x| x.clone()).collect();
    keys.sort();
    for key in keys {
      result.push_str("|");
      let value: String =
        url::form_urlencoded::byte_serialize(hash.get(&key).unwrap().as_bytes()).collect();
      result.push_str(&value)
    }
    result
  }
}

#[derive(Debug, Clone, SubparTable)]
pub struct DB {
  // pub sent_messages: Vec<SentMessage>,
  // pub submissions: Vec<Submission>,
  pub invoices: Vec<Invoice>,
  // #[subpar(rename="payments")]
  // pub payment: Vec<Payment>,
}

// These are in the process of becoming macros themselves. Step one is to make the code work in the concrete
// world before converting to an abstract.
impl DB {
  pub fn upsert_invoice(workbook: &subpar::Workbook, invoice: &Invoice) -> Result<(), SubparError> {
    /// Duplicate code.
    let metadata: &SheetMetadata = match workbook.metadata.sheet_map.get("invoices") {
      Some(metadata) => metadata,
      None => Err(SubparError::NotFound(format!(
        "Could not find metadata for {}",
        "invoices"
      )))?,
    };

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

  pub fn update_metadata(&self, workbook: &mut subpar::Workbook) -> Result<(), SubparError> {
    debug!("Reconciling metadata");
    let invoice_metadata: &SheetMetadata = match workbook.metadata.sheet_map.get("invoices") {
      Some(metadata) => metadata,
      None => Err(SubparError::NotFound(format!(
        "Could not find metadata for {}",
        "invoices"
      )))?,
    };

    log::debug!("invoice_metadata.key_map:\n{:#?}", invoice_metadata.key_map);
    let mut existing = std::collections::HashMap::<String, i64>::new();
    for (key, value) in invoice_metadata.key_map.iter() {
      existing.insert(key.clone(), value.row_id.clone());
    }

    let mut updates = std::collections::HashMap::<String, sheets_db::BatchUpdateRequestItem>::new();
    let invoices = workbook.read_sheet("invoices".to_string())?;
    let sheet_id = invoice_metadata.sheet_id.clone();

    // Insert any new records
    for row in invoices.data {
      let invoice = Invoice::from_excel(&to_row(row.clone(), &invoices.header_map))?;
      let key = invoice.get_key_string();
      // println!("Checking Metadata on {:#?}:\n{:#?}", row[0].location, row);
      match existing.remove(&key) {
        Some(value) => {
          println!("Found key {} with metadataId {}", key, value);
          // verify
        }
        None => {
          debug!("No key found for {}. inserting", key);
          let developer_metadata = sheets_db::DeveloperMetadata {
            id: None,
            key: "row_key".to_string(),
            value: key.clone(),
            visibility: sheets_db::DeveloperMetadataVisibility::Project,
            location: sheets_db::DeveloperMetadataLocation {
              location_type: None,
              value: sheets_db::DeveloperMetadataLocationValue::Range(sheets_db::DimensionRange {
                sheet_id,
                dimension: sheets_db::Dimension::Rows,
                start_index: 1,
                end_index: 2,
              }),
            },
          };

          updates.insert(
            key.clone(),
            sheets_db::BatchUpdateRequestItem::CreateDeveloperMetadata(
              sheets_db::CreateDeveloperMetadataRequest { developer_metadata },
            ),
          );
        }
      }
    }

    // Now delete anything we didn't find
    for (key, row_id) in existing.iter() {
      log::debug!(
        "Did not find matching key '{}': Removing from the metadata",
        key
      );
      let range = sheets_db::DeveloperMetadataLookup {
        location_type: None,
        metadata_location: sheets_db::DeveloperMetadataLocation {
          location_type: Some(sheets_db::DeveloperMetadataLocationType::Sheet),
          value: sheets_db::DeveloperMetadataLocationValue::SheetId(sheet_id),
        },
        metadata_key: Some("row_key".to_string()),
        metadata_value: None,
        metadata_id: Some(row_id.clone()),
        location_matching_strategy: sheets_db::DeveloperMetadataMatchingStrategy::Intersecting,
        visibility: None,
      };
      updates.insert(
        key.clone(),
        sheets_db::BatchUpdateRequestItem::DeleteDeveloperMetadata(
          sheets_db::DeleteDeveloperMetadataRequest {
            filter: sheets_db::DataFilter::Lookup(range),
          },
        ),
      );
    }

    let requests: Vec<sheets_db::BatchUpdateRequestItem> =
      updates.values().into_iter().map(|x| x.clone()).collect();
    if requests.len() > 0 {
      let result = workbook.update_workbook(requests)?;
      log::debug!("\n\n-->\tUpdate Result:\n{:#?}", result);
    } else {
      log::debug!("All metadata already matches. Don't have to run an update")
    }
    Ok(())
  }
}

impl std::fmt::Display for DB {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\tsubmissions: \n\tinvoices: {}",
      // self.submissions.len(),
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
  println!("db submissions:\n{:#?}", db);
}

#[test]
fn test_sheets() {
  env_logger::init();
  // Read the submissions tab of the log using subpar
  // let sheet_id = String::from("1kwQgjicMgKVV1aZ1oStIjpahQLDronaqzkTKdD-paI0");

  // Just Invoices sample sheet
  let sheet_id = String::from("1YQWQE2rcM3O9mj4P_oT9GS8iWzDhkSMw0l_XH9f9Vzs");
  // let sheet_id = String::from("1mQgyqcCYBqyfFmHZB6q5J_L2FQSQ5ppO5hxo_81utZ0");
  let db_conf = subpar::WorkbookConfig::new_sheets_config(
    Some(sheet_id.clone()),
    "/home/dfogelson/fhl_service_acct.json".to_string(),
    "fhl@landfillinc.com".to_string(),
  );

  debug!("db_conf:\n{:#?}", db_conf);

  let mut wb = subpar::Workbook::open(&db_conf).expect("Failed opening the google sheets workbook");
  let db = DB::from_excel(&ExcelObject::Workbook(wb.clone())).unwrap();
  println!("Done loading DB:\n{}", db);

  // Reconcile the metadata on the worksheet. This should likely be done as part of the "open" command.
  let _ = db
    .update_metadata(&mut wb)
    .expect("Error updating the metadata");

  // append a new invoice
  let mut invoice = Invoice {
    guid: "Live Append Test".to_string(),
    organization: "FHL".to_string(),
    submissions: "[\"F20-4\"]".to_string(),
    created_on: chrono::Utc::now().naive_utc(),
  };

  let result = DB::upsert_invoice(&wb, &invoice);
  debug!("The result of the upsert: {:#?}", result);

  // update the invoice
  // invoice.submissions = "[\"F20-6\"]".to_string();
  // let result = DB::upsert_invoice(&wb, &invoice);
  // debug!("The result of the update: {:#?}", result);

  // reread the database

  // let db = DB::load_db().expect("Failed to open the db");

  // debug!("db:\n{:#?}", db);
}
