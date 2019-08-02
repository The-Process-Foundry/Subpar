use calamine::Reader;
// use chrono::{NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;

#[doc(hidden)]
pub use subpar_derive::FromExcel;

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone)]
pub enum SubparError {
  InvalidCellType(String),
  InvalidExcelObject(String),
  InvalidPath(String),
  FileReadOnly(String),
  NotFound(String),
  UnknownColumn(String),
  UnexpectedError(String),
  UnimplementedError(String),
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
        format!("There was a probjelm opening the workbook: {:#?}", err).to_string(),
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
      _ => Err(SubparError::InvalidExcelObject(
        "Can only call get_sheet on ExcelObject::Workbook Objects".to_string(),
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
      _ => panic!("We expected an excel sheet here"),
    }
  }

  fn get_object_name() -> String {
    U::get_object_name()
  }
}

impl FromExcel for String {
  fn from_excel(excel_object: &ExcelObject) -> Result<String, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        calamine::DataType::String(value) => Ok(value.to_string()),
        calamine::DataType::Float(value) => Ok(value.to_string()),
        calamine::DataType::Int(value) => Ok(value.to_string()),
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



#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, FromExcel)]
pub struct Payment {
  guid: String,
  payer: String,
  // payee: String,
  // method: String,
  // amount: f64,
  // comment: Option<String>,
  // date_received: NaiveDateTime,
}



// #[derive(Debug, Clone)]
#[derive(Debug, Clone, FromExcel)]
pub struct Submission {
  guid: String,
  submitting_org: String,
  // payee: String,
  // method: String,
  // amount: f64,
  // comment: Option<String>,
  // date_received: NaiveDateTime,
}

// #[derive(Debug, Clone)]
#[derive(Debug, Clone, FromExcel)]
pub struct DB {
  payments: Vec<Payment>,
  submissions: Vec<Submission>,
}

  #[test]
  fn test_payment() {
    let wb = MetaWorkbook::new("../subpar_test/data/test_db.xlsx".to_string());
    let db = DB::from_excel(&ExcelObject::Workbook(wb));
    println!("db:\n{:#?}", db);
  }
}
