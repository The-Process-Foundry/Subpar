//! Load tablular data from a Google Sheets worksheet

use super::{to_subpar_error, CellType, SubparError};
use anyhow::{Context, Result};
use log::debug;
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct SheetsConfig {
  pub workbook_id: Option<String>,
  pub auth: wrapi::AuthMethod,
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
  pub fn list_sheets(conf: &SheetsConfig) -> Result<Vec<String>> {
    let worksheet = sheets_db::SheetDB::open(conf.auth.clone(), conf.workbook_id.clone().unwrap())
      .expect("Error opening the worksheet");

    Ok(to_subpar_error!(worksheet.list_sheets()))
  }

  /// Create a key to row map from the worksheet metadata. This is how we figure out the location
  /// for data updates.
  fn get_metadata_keymap(
    worksheet: &sheets_db::SheetDB,
    sheet_id: i64,
  ) -> Result<std::collections::HashMap<String, super::SheetRowId>> {
    // Load the key map
    let range = sheets_db::DeveloperMetadataLookup {
      location_type: None,
      metadata_location: sheets_db::DeveloperMetadataLocation {
        location_type: Some(sheets_db::DeveloperMetadataLocationType::Sheet),
        value: sheets_db::DeveloperMetadataLocationValue::SheetId(sheet_id),
      },
      metadata_key: Some("row_key".to_string()),
      metadata_value: None,
      metadata_id: None,
      location_matching_strategy: sheets_db::DeveloperMetadataMatchingStrategy::Intersecting,
      visibility: None,
    };
    let filters = vec![sheets_db::DataFilter::Lookup(range)];
    let search_result = to_subpar_error!(worksheet.search_metadata(filters));
    let mut key_map = std::collections::HashMap::new();
    match search_result.matches {
      None => (),
      Some(metadata) => {
        for key in metadata {
          let value = key.developer_metadata.value;
          let row_number = match key.developer_metadata.location.value {
            sheets_db::DeveloperMetadataLocationValue::Range(range) => range.start_index,
            sheets_db::DeveloperMetadataLocationValue::SheetId(_) => {
              unimplemented!("SheetId is not currently used in storing developer sheet metadata")
            }
            sheets_db::DeveloperMetadataLocationValue::Spreadsheet(_) => unimplemented!(
              "Spreadsheet is not currently used in storing developer sheet metadata"
            ),
          };
          let row_id = key.developer_metadata.id.unwrap();
          match key_map.insert(value.clone(), super::SheetRowId { row_number, row_id }) {
            None => (),
            Some(_) => log::warn!(
              "Duplicate key '{}' metadata in sheet_id '{}'. Should be impossible",
              value,
              sheet_id
            ),
          }
        }
      }
    }
    Ok(key_map)
  }

  /// Take a list of actions and then send the batch update to Google
  pub fn update_workbook(
    conf: SheetsConfig,
    requests: Vec<sheets_db::BatchUpdateRequestItem>,
  ) -> Result<Box<sheets_db::BatchUpdateResponse>> {
    let worksheet = to_subpar_error!(sheets_db::SheetDB::open(
      conf.auth.clone(),
      conf.workbook_id.clone().unwrap()
    ));
    Ok(to_subpar_error!(worksheet.batch_update(requests)))
  }

  pub fn read_metadata(conf: &SheetsConfig) -> Result<super::WorkbookMetadata> {
    let mut sheets = std::collections::HashMap::new();
    let worksheet = to_subpar_error!(sheets_db::SheetDB::open(
      conf.auth.clone(),
      conf.workbook_id.clone().unwrap()
    ));
    for sheet_name in to_subpar_error!(worksheet.list_sheets()).iter() {
      let props = to_subpar_error!(worksheet.get_sheet_properties(sheet_name.clone()));
      let range = (
        props.grid_properties.row_count.clone() as usize,
        props.grid_properties.column_count.clone() as usize,
      );

      let key_map = SheetsWorkbook::get_metadata_keymap(&worksheet, props.sheet_id)?;

      sheets.insert(
        sheet_name.clone(),
        super::SheetMetadata {
          header_map: None,
          header_vec: None,
          sheet_id: props.sheet_id.clone(),
          range,
          key_map,
        },
      );
    }

    Ok(super::WorkbookMetadata {
      sheet_map: sheets,
      last_accessed: chrono::Utc::now(),
    })
  }

  pub fn read_sheet(conf: SheetsConfig, sheet_name: String) -> Result<super::Sheet> {
    let workbook_id = match conf.workbook_id {
      Some(id) => Ok(id),
      None => Err(SubparError::SheetsError)
        .context("Cannot read_sheet because no workbook ID was configured"),
    }?;
    debug!(
      "Reading the sheet named '{}' in workbook '{}'",
      sheet_name, workbook_id
    );

    let worksheet = to_subpar_error!(sheets_db::SheetDB::open(conf.auth.clone(), workbook_id));
    let sheet = to_subpar_error!(worksheet.get_sheet(sheet_name.clone()));
    let value_range: &sheets_db::ValueRange = sheet.borrow();
    // debug!("Received data for range {:#?}", value_range.range);
    let mut rows = value_range.values.clone().into_iter();
    // Build the headers
    let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let header_vec = match rows.next() {
      None => Err(SubparError::EmptyWorksheet)
        .context(format!("Could not read empty worksheet '{}'", sheet_name))?,
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

// pub fn append_item<T>(conf: SheetsConfig, )

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
fn _to_cell_type(data: &sheets_db::Value) -> CellType {
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
