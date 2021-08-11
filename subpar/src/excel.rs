//! Excel Specific module for handling tabular data
//!
//! This is read-only because calamine only does read. The primary driver for this
//! is CSV and Google Sheets, and Excel was only for importing data from an end user.
//!

use anyhow::{Context, Result};
use calamine::Reader;

use super::{CellType, SubparError};

type ExcelReader = calamine::Xlsx<std::io::BufReader<std::fs::File>>;

#[derive(Debug, Clone)]
pub struct ExcelConfig {
  pub path: String,
}

pub struct ExcelWorkbook {
  reader: calamine::Xlsx<std::io::BufReader<std::fs::File>>,
}

impl std::fmt::Debug for ExcelWorkbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Workbook with sheets: {:#?}",
      self.reader.defined_names()
    )
  }
}

// impl Clone for ExcelWorkbook {
//   fn clone(&self) -> Self {
//     ExcelWorkbook {
//       reader: self.reader,
//     }
//   }
// }

impl ExcelWorkbook {
  fn get_reader(path: &String) -> Result<ExcelReader> {
    match calamine::open_workbook(path.clone()) {
      Ok(wb) => Ok(wb),
      Err(err) => Err(SubparError::InvalidPath)
        .context(format!("There was a problem opening the excel workbook: {:#?}", err).to_string()),
    }
  }

  fn get_range(
    conf: &ExcelConfig,
    sheet_name: String,
  ) -> Result<calamine::Range<calamine::DataType>> {
    let mut reader: ExcelReader = ExcelWorkbook::get_reader(&conf.path)?;
    match reader.worksheet_range(&sheet_name[..]) {
      Some(Ok(range)) => Ok(range),
      _ => unimplemented!(),
    }
  }

  fn list_sheets(conf: &ExcelConfig) -> Result<Vec<String>> {
    let reader: ExcelReader = ExcelWorkbook::get_reader(&conf.path)?;
    Ok(
      reader
        .sheet_names()
        .into_iter()
        .map(|e| e.clone())
        .collect(),
    )
  }

  pub fn read_metadata(conf: &ExcelConfig) -> Result<super::WorkbookMetadata> {
    let mut sheets = std::collections::HashMap::new();
    let sheet_names = ExcelWorkbook::list_sheets(&conf)?;
    for sheet in sheet_names {
      let range = ExcelWorkbook::get_range(conf, sheet.clone())?;
      let (height, width) = range.get_size();
      sheets.insert(
        sheet.clone(),
        super::SheetMetadata {
          header_map: None,
          header_vec: None,
          sheet_id: 0,
          range: (height.clone(), width.clone()),
          key_map: std::collections::HashMap::new(),
        },
      );
    }

    Ok(super::WorkbookMetadata {
      name: "Name not implemented for Excel".to_string(),
      sheet_map: sheets,
      last_accessed: chrono::Utc::now(),
    })
  }

  pub fn open(path: String) -> Result<ExcelWorkbook> {
    match calamine::open_workbook(path.clone()) {
      Ok(wb) => Ok(ExcelWorkbook { reader: wb }),
      Err(err) => Err(SubparError::InvalidPath)
        .context(format!("There was a problem opening the workbook: {:#?}", err).to_string()),
    }
  }

  pub fn read_sheet(conf: ExcelConfig, sheet_name: String) -> Result<super::Sheet> {
    let mut reader: ExcelReader = ExcelWorkbook::get_reader(&conf.path)?;
    match reader.worksheet_range(&sheet_name[..]) {
      Some(Ok(range)) => {
        let (height, width) = range.get_size();
        println!("Rows found in Excel: {} x {} ", height, width);

        // Convert to a generic Range
        let mut rows = range.rows();

        // Build the header hash
        let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let header_vec = match rows.next() {
          None => Err(SubparError::EmptyWorksheet)
            .context(format!("Could not read empty worksheet '{}'", sheet_name))?,
          Some(row) => row
            .into_iter()
            .enumerate()
            .map(|(i, cell)| {
              let header = match cell {
                calamine::DataType::String(value) => value.to_lowercase().trim().to_string(),
                calamine::DataType::Empty => "".to_string(),
                calamine::DataType::Int(int) => format!("{:#?}", int),
                calamine::DataType::Float(float) => format!("{:#?}", float),
                calamine::DataType::Bool(boolean) => match boolean {
                  true => "True".to_string(),
                  false => "False".to_string(),
                },
                calamine::DataType::Error(error) => panic!(format!(
                  "Don't know how to use an error to create a header: {:#?}",
                  error
                )),
              };
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
                data: to_cell_type(cell),
              })
              .collect()
          })
          .collect();

        Ok(super::Sheet {
          header_map: lookup,
          header_vec: header_vec,
          data: data,
        })
      }
      Some(Err(err)) => panic!(
        "Got an unknown error retrieving the sheet {}:\n{:#?}",
        sheet_name, err
      ),
      None => panic!(
        "Get sheet returned None when trying to get sheet '{}'. Valid members are {:#?}",
        sheet_name,
        reader.defined_names()
      ),
    }
  }
}

// fn cell_to_(excel_object: &ExcelObject) -> Result<, SubparError> {

// }

// Convert this to "impl From"
fn to_cell_type(data: &calamine::DataType) -> CellType {
  match data {
    calamine::DataType::String(string) => CellType::String(string.clone()),
    calamine::DataType::Empty => CellType::Null,
    calamine::DataType::Int(int) => CellType::Number(int.clone() as f64),
    calamine::DataType::Float(float) => CellType::Number(float.clone() as f64),
    calamine::DataType::Bool(boolean) => match boolean {
      true => CellType::Bool(true),
      false => CellType::Bool(false),
    },
    // Not sure what to do with this one, as I've never seen it
    calamine::DataType::Error(_error) => unimplemented!(),
  }
}
