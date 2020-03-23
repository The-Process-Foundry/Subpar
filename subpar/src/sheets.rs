//! Load tablular data from a Google Sheets worksheet

use super::{CellType, SubparError};
use log::debug;
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct SheetsConfig {
  workbook_id: Option<String>,
  auth: wrapi::AuthMethod,
}

impl SheetsConfig {
  pub fn new(workbook_id: Option<String>, path: String, user_name: String) -> SheetsConfig {
    let auth = wrapi::build_service_account(path, Some(user_name));
    SheetsConfig { auth, workbook_id }
  }
}

pub struct SheetsWorkbook {
  config: SheetsConfig,
  // workbook: sheets_db::SheetDB,
}

impl SheetsWorkbook {
  pub fn list_sheets(conf: &SheetsConfig) -> Result<Vec<String>, SubparError> {
    let worksheet = sheets_db::SheetDB::open(conf.auth.clone(), conf.workbook_id.clone().unwrap())
      .expect("Error opening the worksheet");

    Ok(worksheet.list_sheets()?)
  }

  pub fn read_metadata(conf: &SheetsConfig) -> Result<super::WorkbookMetadata, SubparError> {
    let mut sheets = std::collections::HashMap::<String, (usize, usize)>::new();
    let worksheet = sheets_db::SheetDB::open(conf.auth.clone(), conf.workbook_id.clone().unwrap())?;
    for sheet_name in worksheet.list_sheets()?.iter() {
      let props = worksheet.get_sheet_properties(sheet_name.clone())?;

      debug!("worksheet.get_sheet_properties:\n{:#?}", props);
      sheets.insert(
        sheet_name.clone(),
        (
          props.grid_properties.row_count.clone() as usize,
          props.grid_properties.column_count.clone() as usize,
        ),
      );
    }

    Ok(super::WorkbookMetadata {
      sheet_map: sheets,
      last_accessed: chrono::Utc::now(),
    })
  }

  pub fn read_sheet(conf: SheetsConfig, sheet_name: String) -> Result<super::Sheet, SubparError> {
    debug!("Reading the sheet named '{}'", sheet_name.clone());
    let worksheet = sheets_db::SheetDB::open(conf.auth.clone(), conf.workbook_id.clone().unwrap())
      .expect("Error opening the worksheet");

    let sheet = worksheet.get_sheet(sheet_name.clone())?;
    let value_range: &sheets_db::ValueRange = sheet.borrow();
    // debug!("Received data for range {:#?}", value_range.range);
    let mut rows = value_range.values.clone().into_iter();
    // Build the headers
    let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let header_vec = match rows.next() {
      None => Err(SubparError::EmptyWorksheet(format!(
        "Could not read empty worksheet '{}'",
        sheet_name
      )))?,
      Some(row) => row
        .into_iter()
        .enumerate()
        .map(|(i, cell)| {
          let header = cell.trim().to_string();
          // match cell {
          //   sheets_db::Value::StringValue(value) => value.to_lowercase().trim().to_string(),
          //   sheets_db::Value::Null => "".to_string(),
          //   sheets_db::Value::Number(num) => format!("{:#?}", num),
          //   sheets_db::Value::Bool(boolean) => match boolean {
          //     true => "True".to_string(),
          //     false => "False".to_string(),
          //   },
          //   _ => panic!("Received an unexpected List/Struct in a header column from google sheets"),
          // };
          lookup.insert(header.clone(), i);
          header
        })
        .collect(),
    };

    let data = rows
      .into_iter()
      .enumerate()
      .map(|(i, row)| {
        row
          .into_iter()
          .enumerate()
          .map(|(j, cell)| super::Cell {
            location: (i, j),
            data: CellType::String(cell.clone()),
          })
          .collect()
      })
      .collect();

    // Now gather the rest of the data
    Ok(super::Sheet {
      header_map: lookup,
      header_vec: header_vec,
      data: data,
    })
  }
}

impl std::fmt::Debug for SheetsWorkbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Workbook with sheets: {:#?}",
      SheetsWorkbook::list_sheets(&self.config)
    )
  }
}

// Convert this to "impl From"
fn to_cell_type(data: &sheets_db::Value) -> CellType {
  match data {
    sheets_db::Value::StringValue(string) => CellType::String(string.clone()),
    sheets_db::Value::Null => CellType::Null,
    sheets_db::Value::Number(num) => CellType::Number(num.clone() as f64),
    sheets_db::Value::Bool(boolean) => match boolean {
      true => CellType::Bool(true),
      false => CellType::Bool(false),
    },
    _ => panic!("Received an unexpected List/Struct in a cell type from google sheets"),
  }
}
