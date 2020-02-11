/*! Subpar - A Tabular Data manager

TODO: Break CSV, Excel and Sheets into separate modules

!*/

use chrono::{NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;

#[doc(hidden)]
pub use subpar_derive::SubparTable;

pub mod common;
pub mod excel;

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone)]
pub enum SubparError {
  EmptyWorksheet(String),
  IncorrectExcelObject(String),
  InvalidCellType(String),
  InvalidPath(String),
  FileReadOnly(String),
  NotFound(String),
  NotImplemented(String),
  NullValue(String),
  FloatParseError(String),
  ReadOnly(String),
  UnknownColumn(String),
  UnexpectedError(String),
  ExcelError(String),
  SheetsError(String),
  CSVError(String),
}

/// A simple trait defining the functions needing to be implemented for each type of configuration
// pub trait WorkbookConfig<SubClass = Self> {
//   fn empty() -> WorkbookType<SubClass>;
// }

/// A trait to define the generic tabular workbook API
pub trait MetaWorkbook {
  fn new(config: &WorkbookConfig) -> Result<Workbook, SubparError>;
  fn open(config: &WorkbookConfig) -> Result<Workbook, SubparError>;
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
pub struct SheetsConfig {
  workbook_id: Option<String>,
  credential_path: String,
}

#[derive(Debug, Clone)]
pub enum WorkbookConfig {
  Excel(excel::ExcelConfig),
  GoogleSheets(SheetsConfig),
  CSV(CsvConfig),
}

// TODO: Should the params be validated at runtime?
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
  pub fn new_sheets_config(workbook_id: Option<String>, path: String) -> WorkbookConfig {
    WorkbookConfig::GoogleSheets(SheetsConfig {
      workbook_id: workbook_id,
      credential_path: path,
    })
  }
}

#[derive(Debug)]
pub enum WorkbookWrapper {
  Unopened,
  Csv,
  Excel(excel::ExcelWorkbook),
  Sheets, //(api: WrapiApi),
}

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
  header_map: std::collections::HashMap<String, usize>,
  header_vec: Vec<String>,
  data: Vec<Vec<Cell>>,
}

/// A place to store generic information about the workbook.
/// This is needed for items like CSV where row/column information disappears after it is read
#[derive(Clone, Debug)]
struct WorkbookMetadata {
  // Sheet Map
// HeaderMap
// Last Modified
}

//
#[derive(Debug)]
pub struct Workbook {
  _metadata: WorkbookMetadata,
  config: WorkbookConfig,
  workbook: WorkbookWrapper,
}

// Just for Copy/Paste
// match self {
//   Excel => unimplemented!(),
//   GoogleSheets => unimplemented!(),
//   CSV => unimplemented!(),
// }

impl MetaWorkbook for Workbook {
  /// Attempt to validate the config passed in and create workbook object ready to run
  fn new(config: &WorkbookConfig) -> Result<Workbook, SubparError> {
    match config {
      WorkbookConfig::Excel(_) => Err(SubparError::ReadOnly(
        "Excel workbooks cannot be created, as they are currently read-only".to_string(),
      )),
      _ => Ok(Workbook {
        // This needs to be derived by SubparTable
        _metadata: WorkbookMetadata {},
        config: config.clone(),
        workbook: WorkbookWrapper::Unopened,
      }),
    }
  }

  /// Open an existing workbook
  fn open(config: &WorkbookConfig) -> Result<Workbook, SubparError> {
    // Reread the metadata if last modified has changed since the last time we changed
    // match self.workbook {
    // WorkbookWrapper::Unopened =>
    match config {
      WorkbookConfig::Excel(_conf) => unimplemented!(),
      WorkbookConfig::GoogleSheets(_conf) => unimplemented!(),
      WorkbookConfig::CSV(_conf) => unimplemented!(),
    }
  }

  // Read the worksheet
  fn read_sheet(&self, sheet_name: String) -> Result<Sheet, SubparError> {
    match &self.workbook {
      WorkbookWrapper::Unopened => unimplemented!(),
      WorkbookWrapper::Excel(wb) => unimplemented!(),
      WorkbookWrapper::Sheets => unimplemented!(),
      WorkbookWrapper::Csv => unimplemented!(),
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
pub trait SubparTable<SubClass = Self> {
  fn from_excel(from_obj: &ExcelObject) -> Result<SubClass, SubparError>;
  fn get_object_name() -> String;
}

/// Special case - This is usually used for converting a Sheet into a range
impl<U> SubparTable for Vec<U>
where
  U: SubparTable,
{
  fn from_excel(excel_object: &ExcelObject) -> Result<Vec<U>, SubparError> {
    let sheet_name = U::get_object_name();
    println!("In vec::<{}>::from_excel", sheet_name);

    match excel_object {
      ExcelObject::Sheet(sheet) => {
        let mut result: Vec<U> = Vec::new();
        for row in sheet.data.clone() {
          let value = U::from_excel(&to_row(row, &sheet.header_map)).expect("Error parsing row");
          result.push(value);
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
          Err(err) => Err(err),
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
    match f64::from_excel(excel_object) {
      Ok(excel_date) => {
        // https://github.com/SheetJS/js-xlsx/blob/3438923e5138f10de0aa70b35a8f56eedcfc320d/bits/20_jsutils.js#L34-L45
        let basedate = Utc.ymd(1899, 11, 30).and_hms(0, 0, 0).naive_utc();
        // println!("The ExcelDate is {:?} and the BaseDate is: {:?}", excel_date, basedate);
        // println!("The parsed date is {:?}", basedate + chrono::Duration::days(excel_date as i64 + 31));
        Ok(basedate + chrono::Duration::days(excel_date as i64 + 31))
      }
      Err(err) => Err(err),
    }
  }

  fn get_object_name() -> String {
    panic!("Tried get_object_name for an Option cell, which makes no sense")
  }
}

impl SubparTable for f64 {
  fn from_excel(excel_object: &ExcelObject) -> Result<f64, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match &cell.data {
        CellType::String(value) => match value.parse::<f64>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
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
        CellType::String(value) => match value.parse::<i32>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
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
impl ExcelObject {
  pub fn get_sheet<'a>(&'a self, sheet_name: String) -> Result<Self, SubparError> {
    match self {
      ExcelObject::Workbook(wb) => Ok(ExcelObject::Sheet(
        wb.read_sheet(sheet_name).expect("Could not get sheet"),
      )),
      _ => Err(SubparError::IncorrectExcelObject(
        "Can only call get_sheet on ExcelObject::Workbook Objects".to_string(),
      )),
    }
  }

  pub fn unwrap_cell<'a>(&'a self) -> Result<Cell, SubparError> {
    match self {
      ExcelObject::Cell(cell) => Ok(cell.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_cell expects a cell object but received something different".to_string(),
      )),
    }
  }

  pub fn unwrap_row<'a>(&'a self) -> Result<HashMap<String, Cell>, SubparError> {
    match self {
      ExcelObject::Row(row) => Ok(row.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_row expects a row hash but received something different".to_string(),
      )),
    }
  }
}

fn to_row(raw: Vec<Cell>, headers: &HashMap<String, usize>) -> ExcelObject {
  let mut row_data: HashMap<String, Cell> = HashMap::new();
  for (key, i) in headers.iter() {
    match row_data.insert(key.clone(), raw[*i].clone()) {
      None => (),
      Some(_) => panic!("Managed a duplicate entry in to_row. Should be impossible"),
    }
  }
  ExcelObject::Row(row_data)
}

pub fn get_cell(excel_object: ExcelObject, cell_name: String) -> Result<ExcelObject, SubparError> {
  match excel_object {
    ExcelObject::Row(row) => match row.get(&cell_name.to_ascii_lowercase()) {
      Some(value) => Ok(ExcelObject::Cell(value.clone())),
      None => {
        println!("\n\n!!!!\nFailed column lookup: {:#?}", row);
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
