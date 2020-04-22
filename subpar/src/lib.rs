/*! Subpar - A Tabular Data manager

Some macros to make using excel, google sheets and CSV easier.
!*/

use chrono::{NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;

#[doc(hidden)]
pub use subpar_derive::SubparTable;

// pub mod common;
pub mod errors;
pub mod excel;
pub mod sheets;

pub use errors::SubparError;

/// A simple trait defining the functions needing to be implemented for each type of configuration
// pub trait WorkbookConfig<SubClass = Self> {
//   fn empty() -> WorkbookType<SubClass>;
// }

/// A trait to define the generic tabular workbook API
pub trait MetaWorkbook {
  fn new(config: &WorkbookConfig) -> Result<Workbook, SubparError>;
  fn open(config: &WorkbookConfig) -> Result<Workbook, SubparError>;
  fn read_metadata(config: &WorkbookConfig) -> Result<WorkbookMetadata, SubparError>;
  fn read_sheet(&self, sheet_name: String) -> Result<Sheet, SubparError>;

  // write_sheet(&self)
  // fn insert_row(&self, sheet_name: String, row_number: i32)
  // fn update_row(&self, sheet_name: String, row_data)
  // fn update_cell(&self, sheet_name: String, column_name: String, value)
}

#[derive(Debug, Clone)]
pub struct CsvConfig {
  path: String,
}

#[derive(Debug, Clone)]
pub enum WorkbookConfig {
  Excel(excel::ExcelConfig),
  GoogleSheets(sheets::SheetsConfig),
  CSV(CsvConfig),
}

impl WorkbookConfig {
  /// Get a configuration item for a local Excel workbook
  ///
  /// # Arguments
  ///
  /// * `path` - The location for the excel workbook
  ///
  pub fn new_excel_config(path: String) -> WorkbookConfig {
    WorkbookConfig::Excel(excel::ExcelConfig { path: path.clone() })
  }

  /// Get a configuration item for a google sheets workbook
  ///
  /// # Arguments
  ///
  /// * `workbook_id` - A string holding Google's identifier of the workbook. If empty, we assume it will
  ///   be a new workbook created
  /// * `path` - The path to the service account credentials. This can be looked up and downloaded via
  ///
  pub fn new_sheets_config(
    workbook_id: Option<String>,
    path: String,
    user_name: String,
  ) -> WorkbookConfig {
    WorkbookConfig::GoogleSheets(sheets::SheetsConfig::new(workbook_id, path, user_name))
  }
}

// #[derive(Debug)]
// pub enum WorkbookWrapper {
//   Unopened,
//   Error(SubparError),
//   Csv,
//   Excel,
//   Sheets, //(api: WrapiApi),
// }

/// The data type of the cell. Everything is kept in strings since I expect it to be parsed later
/// using the from_cell/to_cell macros
///
/// This is borrowed from Google's Protocol buffer, since I'm doing the most work with that
/// https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Value
#[derive(Clone, Debug)]
pub enum CellType {
  Null,
  Number(f64),
  String(String),
  Bool(bool),
  Struct(String),
  List(Vec<CellType>),
}

#[derive(Clone, Debug)]
pub struct Cell {
  pub location: (usize, usize),
  pub data: CellType,
}

#[derive(Clone, Debug)]
pub struct Sheet {
  pub header_map: std::collections::HashMap<String, usize>,
  pub header_vec: Vec<String>,
  pub data: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug)]
pub struct SheetRowId {
  row_number: i32,
  row_id: String,
}

#[derive(Clone, Debug)]
pub struct SheetMetadata {
  pub sheet_id: i64,
  pub range: (usize, usize),
  pub key_map: std::collections::HashMap<String, SheetRowId>,
}

/// A place to store generic information about the workbook.
/// This is needed for items like CSV where row/column information disappears after it is read
#[derive(Clone, Debug)]
pub struct WorkbookMetadata {
  pub sheet_map: std::collections::HashMap<String, SheetMetadata>,
  pub last_accessed: chrono::DateTime<Utc>,
}

//
#[derive(Clone, Debug)]
pub struct Workbook {
  pub metadata: WorkbookMetadata,
  config: WorkbookConfig,
  // workbook: WorkbookWrapper,
}

// Just for Copy/Paste
// match self {
//   Excel => unimplemented!(),
//   GoogleSheets => unimplemented!(),
//   CSV => unimplemented!(),
// }
// impl Workbook {
//   pub fn search_metadata(
//     &self,
//     filters: Vec<sheets_db::DataFilter>,
//   ) -> Result<SheetMetadata, SubparError> {
//     match &self.config {
//       WorkbookConfig::Excel(_) => unimplemented!("Search Metadata is not implemented for Excel"),
//       WorkbookConfig::CSV(_) => unimplemented!("Search Metadata is not implemented for CSV"),
//       WorkbookConfig::GoogleSheets(conf) => sheets::SheetsWorkbook::search_metadata(conf, filters),
//     }
//   }
// }

// Implemetation made to route the workbook to the proper code based on the workbook type
impl MetaWorkbook for Workbook {
  /// Attempt to validate the config passed in and create workbook object ready to run
  fn new(config: &WorkbookConfig) -> Result<Workbook, SubparError> {
    match config {
      WorkbookConfig::Excel(_) => Err(SubparError::ReadOnly(
        "Excel workbooks cannot be created, as they are currently read-only".to_string(),
      )),
      _ => unimplemented!(),
    }
  }

  fn read_metadata(config: &WorkbookConfig) -> Result<WorkbookMetadata, SubparError> {
    match config {
      WorkbookConfig::Excel(conf) => excel::ExcelWorkbook::read_metadata(&conf),
      WorkbookConfig::GoogleSheets(conf) => sheets::SheetsWorkbook::read_metadata(&conf),
      WorkbookConfig::CSV(_conf) => unimplemented!(),
    }
  }

  /// Open an existing workbook
  fn open(config: &WorkbookConfig) -> Result<Workbook, SubparError> {
    let metadata = Workbook::read_metadata(&config)?;
    Ok(Workbook {
      metadata: metadata,
      config: config.clone(),
    })
  }

  // Read the worksheet
  fn read_sheet(&self, sheet_name: String) -> Result<Sheet, SubparError> {
    // sheet in metadata?
    // refresh metadata if notread_sheet
    match self.metadata.sheet_map.get(&sheet_name) {
      None => Err(SubparError::UnknownSheet(format!(
        "The workbook does not contain a sheet named '{}'. Possible options: {:#?}",
        sheet_name, self.metadata.sheet_map
      ))),
      Some(_) => match &self.config {
        WorkbookConfig::Excel(conf) => {
          excel::ExcelWorkbook::read_sheet(conf.clone(), sheet_name.clone())
        }
        WorkbookConfig::GoogleSheets(conf) => {
          sheets::SheetsWorkbook::read_sheet(conf.clone(), sheet_name.clone())
        }
        WorkbookConfig::CSV(_conf) => unimplemented!(),
      },
    }
  }
}
/// Wrappers for the various types of Excel Resources so we can pass them around more easily
///
/// This allows us to generically iterate through the conversions
pub enum ExcelObject {
  Cell(Cell),
  Sheet(Sheet),
  Row(std::collections::HashMap<String, Cell>),
  Workbook(Workbook),
}

/// Convert a row from a given table into the given struct
pub trait SubparTable<SubClass = Self>: std::fmt::Debug + std::clone::Clone {
  fn from_excel(from_obj: &ExcelObject) -> Result<SubClass, SubparError>;
  fn get_object_name() -> String;
  // fn get_key_hash() -> HashMap<String, String>;
}

/// Special case - This is usually used for converting a Sheet into a range
impl<U> SubparTable for Vec<U>
where
  U: SubparTable,
{
  fn from_excel(excel_object: &ExcelObject) -> Result<Vec<U>, SubparError> {
    // let sheet_name = U::get_object_name();
    // println!("In vec::<{}>::from_excel", sheet_name);

    match excel_object {
      ExcelObject::Sheet(sheet) => {
        let mut result: Vec<U> = Vec::new();
        for (i, row) in sheet.data.clone().iter().enumerate() {
          let value = U::from_excel(&to_row(row.clone(), &sheet.header_map));
          match value.clone() {
            Ok(x) => result.push(x),
            err => {
              // TODO: Return the key, not just the row number
              let msg = format!("Error parsing row number {}: \n{:#?}", i, err);
              log::warn!("{}", msg);
              value?;
            }
          }
        }
        Ok(result)
      }
      _ => panic!("We expected either an excel sheet or cell here"),
    }
  }

  fn get_object_name() -> String {
    U::get_object_name()
  }
}

impl<U> SubparTable for Option<U>
where
  U: SubparTable,
{
  fn from_excel(excel_object: &ExcelObject) -> Result<Option<U>, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell.data {
        CellType::Null => Ok(None),
        _ => match U::from_excel(&excel_object.clone()) {
          Ok(value) => Ok(Some(value)),
          Err(err) => match err {
            SubparError::NullValue(_) => Ok(None),
            _ => Err(err),
          },
        },
      },
      _ => panic!("Tried to parse an optional object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an Option cell, which makes no sense")
  }
}

impl SubparTable for String {
  fn from_excel(excel_object: &ExcelObject) -> Result<String, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell.data.clone() {
        CellType::String(value) => Ok(value.clone()),
        CellType::Number(value) => Ok(value.to_string()),
        CellType::Null => Err(SubparError::NullValue(
          "Empty strings are invalid for String type".to_string(),
        )),
        x => Err(SubparError::InvalidCellType(format!(
          "Cannot turn {:?} into a String",
          x
        ))),
      },
      _ => panic!("Tried to parse a string from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_sheet_name for a String, which makes no sense")
  }
}

impl SubparTable for NaiveDateTime {
  fn from_excel(excel_object: &ExcelObject) -> Result<NaiveDateTime, SubparError> {
    // Excel sends back float and Google sheets is a string, so
    match f64::from_excel(excel_object) {
      Ok(excel_date) => {
        // https://github.com/SheetJS/js-xlsx/blob/3438923e5138f10de0aa70b35a8f56eedcfc320d/bits/20_jsutils.js#L34-L45
        let basedate = Utc.ymd(1899, 11, 30).and_hms(0, 0, 0).naive_utc();
        // println!("The ExcelDate is {:?} and the BaseDate is: {:?}", excel_date, basedate);
        // println!("The parsed date is {:?}", basedate + chrono::Duration::days(excel_date as i64 + 31));
        Ok(basedate + chrono::Duration::days(excel_date as i64 + 31))
      }
      Err(err) => {
        let cell = excel_object.unwrap_cell().unwrap().data;
        match cell {
          CellType::String(value) => {
            // assuming the default of m/d/y
            if value.len() == 0 {
              Err(SubparError::NullValue(
                "Received an empty DateTime to parse".to_string(),
              ))?
            }
            let mut tokens: Vec<Result<u32, _>> =
              value.split("/").map(|token| token.parse()).collect();
            match tokens.len() {
              3 => {
                let year = tokens.pop().unwrap()?;
                let day = tokens.pop().unwrap()?;
                let month = tokens.pop().unwrap()?;
                let y4 = match (year < 30, year < 100) {
                  (true, _) => 2000 + year,
                  (_, true) => 1900 + year,
                  (false, false) => year,
                };

                Ok(Utc.ymd(y4 as i32, month, day).and_hms(0, 0, 0).naive_utc())
              }
              _ => {
                let msg = format!(
                  "could not format '{}' into a date formatted as m/d/y.",
                  value
                );
                log::error!("{}", msg);
                Err(SubparError::ParsingError(msg))?
              }
            }
          }
          _ => Err(err)?,
        }
      }
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for a DateTime cell, which makes no sense")
  }
}

impl SubparTable for f64 {
  fn from_excel(excel_object: &ExcelObject) -> Result<f64, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => match value.parse::<f64>() {
          Ok(x) => Ok(x),
          Err(_) => {
            match value.len() {
              0 => Err(SubparError::NullValue(
                "Cannot convert an empty string into an f64".to_string(),
              ))?,
              _ => (),
            }
            let cleaned = value.replace(',', "");
            Ok(cleaned.parse::<f64>()?)
          }
        },
        CellType::Number(value) => Ok(value.clone()),
        x => Err(SubparError::InvalidCellType(format!(
          "\n!!! Cannot turn {:?} into a f64",
          x
        ))),
      },
      _ => panic!("Tried to parse an f64 object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an f64 cell, which makes no sense")
  }
}

impl SubparTable for f32 {
  fn from_excel(excel_object: &ExcelObject) -> Result<f32, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => match value.parse::<f32>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        CellType::Number(value) => Ok(value.clone() as f32),
        x => Err(SubparError::InvalidCellType(format!(
          "\n!!! Cannot turn {:?} into a f32",
          x
        ))),
      },
      _ => panic!("Tried to parse an f32 object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an f32 cell, which makes no sense")
  }
}

impl SubparTable for i64 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i64, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => match value.parse::<i64>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        CellType::Number(value) => Ok(value.round() as i64),
        x => Err(SubparError::InvalidCellType(format!(
          "\n!!! Cannot turn {:?} into a i64",
          x
        ))),
      },
      _ => panic!("Tried to parse an i64 object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an i64 cell, which makes no sense")
  }
}

impl SubparTable for i32 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i32, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => {
          if value.len() == 0 {
            Err(SubparError::NullValue(
              "Received an empty string to parse into an i32".to_string(),
            ))?
          }
          match value.parse::<i32>() {
            Ok(x) => Ok(x),
            Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
          }
        }
        CellType::Number(value) => Ok(value.round() as i32),
        x => Err(SubparError::InvalidCellType(format!(
          "\n!!! Cannot turn {:?} into a i32",
          x
        ))),
      },
      _ => panic!("Tried to parse an i32 object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an i32 cell, which makes no sense")
  }
}

impl SubparTable for i16 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i16, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => match value.parse::<i16>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        CellType::Number(value) => Ok(value.round() as i16),
        x => Err(SubparError::InvalidCellType(format!(
          "\n!!! Cannot turn {:?} into a i16",
          x
        ))),
      },
      _ => panic!("Tried to parse an i16 object from a non-cell ExcelObject"),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an i16 cell, which makes no sense")
  }
}

pub fn cell_csv_to_vec(_cell: Cell) -> Result<Vec<String>, SubparError> {
  Err(SubparError::NotImplemented(
    "cell_csv_to_vec is not yet implemented".to_string(),
  ))
}

/// Wrappers for the various types of Excel Resources so we can pass them around more easily
///
/// This allows us to generically iterate through the conversions
impl<'a> ExcelObject {
  pub fn get_sheet(&'a self, sheet_name: String) -> Result<Self, SubparError> {
    match self {
      ExcelObject::Workbook(wb) => Ok(ExcelObject::Sheet(
        wb.read_sheet(sheet_name).expect("Could not get sheet"),
      )),
      _ => Err(SubparError::IncorrectExcelObject(
        "Can only call get_sheet on ExcelObject::Workbook Objects".to_string(),
      )),
    }
  }

  pub fn unwrap_cell(&'a self) -> Result<Cell, SubparError> {
    match self {
      ExcelObject::Cell(cell) => Ok(cell.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_cell expects a cell object but received something different".to_string(),
      )),
    }
  }

  pub fn unwrap_row(&'a self) -> Result<HashMap<String, Cell>, SubparError> {
    match self {
      ExcelObject::Row(row) => Ok(row.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_row expects a row hash but received something different".to_string(),
      )),
    }
  }
}

pub fn to_row(raw: Vec<Cell>, headers: &HashMap<String, usize>) -> ExcelObject {
  let mut row_data: HashMap<String, Cell> = HashMap::new();
  let row_length = raw.len();
  for (key, i) in headers.iter() {
    // Sheets does not return square ranges so we have to test that the row is actually long enough
    let value = match *i >= row_length {
      true => {
        // log::debug!(
        //   "Got a header longer than the length: {} >= {}",
        //   i,
        //   row_length
        // );
        Cell {
          location: (raw[0].location.0, i.clone()),
          data: CellType::Null,
        }
      }
      false => raw[*i].clone(),
    };
    match row_data.insert(key.clone(), value) {
      None => (),
      Some(_) => panic!("Managed a duplicate entry in to_row. Should be impossible"),
    }
  }
  ExcelObject::Row(row_data)
}

pub fn get_cell(excel_object: ExcelObject, cell_name: String) -> Result<ExcelObject, SubparError> {
  match excel_object {
    ExcelObject::Row(row) => match row.get(&cell_name) {
      Some(value) => Ok(ExcelObject::Cell(value.clone())),
      None => {
        let mut keys: Vec<String> = row.keys().into_iter().map(|key| key.clone()).collect();
        keys.sort();
        log::debug!(
          "\n\n!!!!\nFailed column lookup '{}' in: {:#?}",
          &cell_name,
          keys,
        );
        Err(SubparError::UnknownColumn(
          String::from("Could not find column named '") + &cell_name + "'",
        ))
      }
    },
    _ => panic!("Expected a row to pull a cell from. Fix the branch or the code"),
  }
}

// #[cfg(test)]
// mod tests {
//   // Note this useful idiom: importing names from outer (for mod tests) scope.
//   use super::*;

//   #[derive(Debug, Clone, SubparTable)]
//   // #[derive(Debug, Clone, SubparTable)]
//   pub struct Payment {
//     guid: String,
//     payer: String,
//     #[subpar(column="I'm the Payment", parser="cell_csv_to_vec")]
//     number_list: Vec<String>
//     // payee: String,
//     // method: String,
//     // amount: f64,
//     // comment: Option<String>,
//     // date_received: NaiveDateTime,
//   }

//   #[derive(Debug, Clone)]
//   // #[derive(Debug, Clone, SubparTable)]
//   pub struct Submission {
//     guid: String,
//     submitting_org: String,
//     // payee: String,
//     // method: String,
//     amount: f64,
//     // comment: Option<String>,
//     // date_received: NaiveDateTime,
//   }

//   // #[derive(Debug, Clone)]
//   #[derive(Debug, Clone, SubparTable)]
//   pub struct DB {
//     payments: Vec<Payment>,
//     // submissions: Vec<Submission>,
//   }

//   #[test]
//   fn test_payment() {
//     let wb = MetaWorkbook::new("../subpar_test/data/test_db.xlsx".to_string());
//     let db = DB::from_excel(&ExcelObject::Workbook(wb));
//     println!("db:\n{:#?}", db);
//   }
// }

// Custom cloning since calamine doesn't implement it for the reader
// impl Clone for Workbook {
//   fn clone(&self) -> Self {
//     Workbook {
//       _metadata: self._metadata.clone(),
//       config: self.config.clone(),
//       workbook: match &self.workbook {
//         WorkbookWrapper::Error(x) => WorkbookWrapper::Error(x.clone()),
//         WorkbookWrapper::Unopened => WorkbookWrapper::Unopened,
//         WorkbookWrapper::Csv => WorkbookWrapper::Csv,
//         WorkbookWrapper::Sheets => WorkbookWrapper::Sheets,
//         WorkbookWrapper::Excel => match self.config.clone() {
//           WorkbookConfig::Excel(conf) => match excel::ExcelWorkbook::open(conf.path.clone()) {
//             Ok(workbook) => WorkbookWrapper::Excel,
//             Err(err) => WorkbookWrapper::Error(err),
//           },
//           x => WorkbookWrapper::Error(SubparError::WorkbookMismatch(format!(
//             "Attempted to clone an excel workbook but didn't have an excel config: {:#?}",
//             x
//           ))),
//         },
//       },
//     }
//   }
// }
