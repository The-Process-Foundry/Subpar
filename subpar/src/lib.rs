use calamine::{DataType, Reader};
use chrono::{NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;

#[doc(hidden)]
pub use subpar_derive::FromExcel;

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone)]
pub enum SubparError {
  IncorrectExcelObject(String),
  InvalidCellType(String),
  InvalidPath(String),
  FileReadOnly(String),
  NotFound(String),
  NotImplemented(String),
  NullValue(String),
  FloatParseError(String),
  UnknownColumn(String),
  UnexpectedError(String),
}

/// Metadata around the workbook so we can implement a clone function for the excel object
//  We can add caching and smarter lookups in the future. I find that a struct with guaranteed
//  values is easier to use than an enum
//  use in
pub struct MetaWorkbook {
  path: String,
  // TODO: Can this really be cloned?
  workbook: Option<calamine::Xlsx<std::io::BufReader<std::fs::File>>>,
}

impl MetaWorkbook {
  pub fn new(path: String) -> Self {
    MetaWorkbook {
      path: path.clone(),
      workbook: None,
    }
  }

  fn open(&self) -> Result<Self, SubparError> {
    match calamine::open_workbook(self.path.clone()) {
      Ok(wb) => Ok(MetaWorkbook {
        path: self.path.clone(),
        workbook: Some(wb),
      }),
      Err(err) => Err(SubparError::InvalidPath(
        format!("There was a problem opening the workbook: {:#?}", err).to_string(),
      )),
    }
  }

  pub fn get_sheet(
    &self,
    sheet_name: String,
  ) -> Result<calamine::Range<calamine::DataType>, SubparError> {
    // Always open this. Cloning and borrowing are painful to try and debug, so make it work
    let wb = match self.open() {
      Ok(opened) => match opened.workbook {
        Some(inner) => Ok(inner),
        None => panic!("Tried to open workbook but still received None after a success"),
      },
      Err(err) => Err(err),
    };

    match wb {
      Err(err) => Err(err),
      Ok(mut wb) => match wb.worksheet_range(&sheet_name[..]) {
        Some(Ok(range)) => {
          let (height, width) = range.get_size();
          println!("Rows: {} x {} ", height, width);
          Ok(range)
        }
        Some(Err(err)) => panic!(
          "Got an unknown error retrieving the sheet {}:\n{:#?}",
          sheet_name, err
        ),
        None => panic!(
          "Get sheet returned None when trying to get sheet '{}'. Valid members are {:#?}",
          sheet_name,
          wb.sheet_names()
        ),
      },
    }
  }
}

impl Clone for MetaWorkbook {
  fn clone(&self) -> Self {
    let cloned = MetaWorkbook {
      path: self.path.clone(),
      workbook: None,
    };
    match &self.workbook {
      None => cloned,
      // Does this make sense? Multiple writers pointing to the same object can be quite ugly
      // Possibility that we should panic if the state is not closed
      Some(_) => match cloned.open() {
        Ok(wb) => wb,
        Err(err) => panic!("there was an issue cloning the workbook:\n{:#?}", err),
      },
    }
  }
}

/// Wrappers for the various types of Excel Resources so we can pass them around more easily
///
/// This allows us to generically iterate through the conversions
#[derive(Clone)]
pub enum ExcelObject {
  Cell(calamine::DataType),
  Sheet(calamine::Range<calamine::DataType>),
  Row(std::collections::HashMap<String, calamine::DataType>),
  Workbook(MetaWorkbook),
}

impl ExcelObject {
  pub fn get_sheet<'a>(&'a self, sheet_name: String) -> Result<Self, SubparError> {
    match self {
      ExcelObject::Workbook(wb) => Ok(ExcelObject::Sheet(
        wb.get_sheet(sheet_name).expect("Could not get sheet"),
      )),
      _ => Err(SubparError::IncorrectExcelObject(
        "Can only call get_sheet on ExcelObject::Workbook Objects".to_string(),
      )),
    }
  }

  pub fn unwrap_cell<'a>(&'a self) -> Result<calamine::DataType, SubparError> {
    match self {
      ExcelObject::Cell(cell) => Ok(cell.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_cell expects a cell object but received something different".to_string(),
      )),
    }
  }

  pub fn unwrap_row<'a>(&'a self) -> Result<HashMap<String, calamine::DataType>, SubparError> {
    match self {
      ExcelObject::Row(row) => Ok(row.clone()),
      _ => Err(SubparError::IncorrectExcelObject(
        "unwrap_row expects a row hash but received something different".to_string(),
      )),
    }
  }
}

fn to_row(raw: &[calamine::DataType], headers: &HashMap<String, usize>) -> ExcelObject {
  let mut row_data: HashMap<String, calamine::DataType> = HashMap::new();
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

/// Convert a row from a given table into the given struct
pub trait FromExcel<SubClass = Self> {
  fn from_excel(from_obj: &ExcelObject) -> Result<SubClass, SubparError>;
  fn get_object_name() -> String;
}

impl<U> FromExcel for Vec<U>
where
  U: FromExcel,
{
  fn from_excel(excel_object: &ExcelObject) -> Result<Vec<U>, SubparError> {
    let sheet_name = U::get_object_name();
    println!("In vec::<{}>::from_excel", sheet_name);

    match excel_object {
      ExcelObject::Sheet(sheet) => {
        let mut rows = sheet.rows();
        let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        if let Some(headers) = rows.next() {
          for (i, cell) in headers.iter().enumerate() {
            match cell {
              calamine::DataType::String(value) => {
                lookup.insert(value.to_lowercase().trim().to_string(), i);
              }
              calamine::DataType::Empty => (),
              _ => println!("Cell '{:?}' is not a string", cell),
            }
          }
        }

        let mut result: Vec<U> = Vec::new();
        for row in rows {
          let value = U::from_excel(&to_row(row, &lookup)).expect("Error parsing row");
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

impl<U> FromExcel for Option<U>
where
  U: FromExcel,
{
  fn from_excel(excel_object: &ExcelObject) -> Result<Option<U>, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        calamine::DataType::Empty => Ok(None),
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

impl FromExcel for String {
  fn from_excel(excel_object: &ExcelObject) -> Result<String, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        calamine::DataType::String(value) => Ok(value.to_string()),
        calamine::DataType::Float(value) => Ok(value.to_string()),
        calamine::DataType::Int(value) => Ok(value.to_string()),
        calamine::DataType::Empty => Err(SubparError::NullValue(
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

impl FromExcel for NaiveDateTime {
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

impl FromExcel for f64 {
  fn from_excel(excel_object: &ExcelObject) -> Result<f64, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        DataType::String(value) => match value.parse::<f64>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        DataType::Float(value) => Ok(*value),
        DataType::Int(value) => Ok(*value as f64),
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

impl FromExcel for f32 {
  fn from_excel(excel_object: &ExcelObject) -> Result<f32, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        DataType::String(value) => match value.parse::<f32>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        DataType::Float(value) => Ok(*value as f32),
        DataType::Int(value) => Ok(*value as f32),
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

impl FromExcel for i64 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i64, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        DataType::String(value) => match value.parse::<i64>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        DataType::Float(value) => Ok(*value as i64),
        DataType::Int(value) => Ok(*value as i64),
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

impl FromExcel for i32 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i32, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        DataType::String(value) => match value.parse::<i32>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        DataType::Float(value) => Ok(*value as i32),
        DataType::Int(value) => Ok(*value as i32),
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

impl FromExcel for i16 {
  fn from_excel(excel_object: &ExcelObject) -> Result<i16, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        DataType::String(value) => match value.parse::<i16>() {
          Ok(x) => Ok(x),
          Err(err) => Err(SubparError::FloatParseError(format!("{:#?}", err))),
        },
        DataType::Float(value) => Ok(*value as i16),
        DataType::Int(value) => Ok(*value as i16),
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

pub fn cell_csv_to_vec(_cell: DataType) -> Result<Vec<String>, SubparError> {
  Err(SubparError::NotImplemented(
    "cell_csv_to_vec is not yet implemented".to_string(),
  ))
}

// #[cfg(test)]
// mod tests {
//   // Note this useful idiom: importing names from outer (for mod tests) scope.
//   use super::*;

//   #[derive(Debug, Clone, FromExcel)]
//   // #[derive(Debug, Clone, FromExcel)]
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
//   // #[derive(Debug, Clone, FromExcel)]
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
//   #[derive(Debug, Clone, FromExcel)]
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
