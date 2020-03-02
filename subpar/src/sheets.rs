//! Load tablular data from a Google Sheets worksheet

use super::{CellType, SubparError};
use log::debug;

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
