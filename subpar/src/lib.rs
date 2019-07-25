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
  UnknownColumn(String),
  UnexpectedError(String),
  UnimplementedError(String),
}

/// Wrapper for the various types of Excel Resources
///
/// This allows us to generically iterate through the conversions
pub enum ExcelObject {
  Cell(calamine::DataType),
  Sheet(calamine::Range<calamine::DataType>),
  Row(std::collections::HashMap<String, calamine::DataType>),
  FilePath(String),
  Workbook(calamine::Xlsx<std::io::BufReader<std::fs::File>>),
}

// TODO: Make a ifOk trait.
// pub fn ifOk(value: Result<A, SubparError>, func: &dyn Fn(A) -> Result<B: SubparError>) -> Result<B, SubparError> {
//   match value
// }

pub fn open_workbook(excel_object: ExcelObject) -> Result<ExcelObject, SubparError> {
  match excel_object {
    ExcelObject::FilePath(path) => match calamine::open_workbook(path) {
      Ok(wb) => Ok(ExcelObject::Workbook(wb)),
      Err(err) => Err(SubparError::InvalidPath(
        format!("There was a probjelm opening the workbook: {:#?}", err).to_string(),
      )),
    },
    _ => Err(SubparError::UnexpectedError(format!(
      "open_workbook can only be called with FilePath or Workbook",
    ))),
  }
}

pub fn get_sheet(
  excel_object: ExcelObject,
  sheet_name: String,
) -> Result<ExcelObject, SubparError> {
  match excel_object {
    ExcelObject::Workbook(mut wb) => match wb.worksheet_range(&sheet_name[..]) {
      Some(Ok(range)) => {
        let (height, width) = range.get_size();
        println!("Rows: {} x {} ", height, width);
        Ok(ExcelObject::Sheet(range))
      }
      Some(Err(err)) => panic!(
        "Got an unknown error retrieving the sheet {}:\n{:#?}",
        sheet_name, err
      ),
      None => panic!(
        "Get sheet returned None when trying to get sheet {}",
        sheet_name
      ),
    },
    _ => panic!("Expected a workbook to pull a sheet from. Fix the branch or the code"),
  }
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
pub trait FromExcel {
  type T;

  fn from_excel(from_obj: ExcelObject) -> Result<Self::T, SubparError>;
  fn get_object_name() -> String;
}

// impl<T> FromExcel for Vec<T>
// where
//   T: FromExcel,
// {
//   fn from_excel(excel_object: ExcelObject) -> Vec<T> {
//     println!("In vec::from_excel");
//     let wb = match excel_object {
//       ExcelObject::FilePath(_) => match open_workbook(excel_object) {
//         Ok(wb) => wb,
//         Err(err) => panic!(format!("{:#?}", err)),
//       },
//       ExcelObject::Workbook(wb) => wb,
//       _ => panic!("Unimplemented branch of Vec<T> from_excel"),
//     };

//     vec![]
//   }

//   fn get_sheet_name() -> String {
//     T::get_sheet_name()
//   }
// }

impl FromExcel for String {
  type T = String;

  fn from_excel(excel_object: ExcelObject) -> Result<String, SubparError> {
    match excel_object {
      ExcelObject::Cell(cell) => match cell {
        calamine::DataType::String(value) => Ok(value),
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

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[derive(Debug, Clone)]
  pub struct Payment {
    guid: String,
    // payer: String,
    // payee: String,
    // method: String,
    // amount: f64,
    // comment: Option<String>,
    // date_received: NaiveDateTime,
  }

  #[derive(Debug, Clone)]
  pub struct DB {
    payments: Vec<Payment>,
  }

  impl FromExcel for DB {
    type T = DB;

    fn from_excel(excel_object: ExcelObject) -> Result<DB, SubparError> {
      let wb = match excel_object {
        ExcelObject::FilePath(_) => match open_workbook(excel_object) {
          Ok(ExcelObject::Workbook(wb)) => ExcelObject::Workbook(wb),
          Ok(_) => panic!("Impossible - got an OK/non Workbook object from Subpar::open_workbook"),
          Err(err) => panic!(format!("Error opening the workbook: {:#?}", err)),
        },
        ExcelObject::Workbook(_) => excel_object,
        _ => panic!("This cannot create a workbook. Fix the branch or the code"),
      };

      // Payments is vec: takes range/sheet
      let payments =
        // if vec, get the sheet, loop row by row
        match get_sheet(wb, "Payment".to_string()).expect("Failed to get the sheet") {
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

            let mut result: Vec<Payment> = Vec::new();
            for row in rows {
              let excel_row = to_row(row, &lookup);

              let guid = String::from_excel(get_cell(excel_row, "guid".to_string()).expect("Could not find guid column for Payment")).expect("Error converting Payment.guid to a string");
              result.push(Payment{guid: guid});
            }
            result
          },
          _ => panic!("We expected an excel sheet here")
        };

      Ok(DB { payments: payments })
    }

    fn get_object_name() -> String {
      "DB".to_string()
    }
  }

  #[test]
  fn test_payment() {
    let db = DB::from_excel(ExcelObject::FilePath(
      "../subpar_test/data/test_db.xlsx".to_string(),
    ));
    println!("db:\n{:#?}", db);
  }
}
