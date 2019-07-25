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
    // For ease of use, this lets us ignore the fact it's already been opened
    ExcelObject::Workbook(wb) => Ok(ExcelObject::Workbook(wb)),
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
  match open_workbook(excel_object).expect("Could not open the workbook when getting sheet") {
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
  fn from_excel(from_obj: ExcelObject) -> Result<SubClass, SubparError>;
  fn get_object_name() -> String;
}

impl<U> FromExcel for Vec<U>
where
  U: FromExcel,
{
  fn from_excel(excel_object: ExcelObject) -> Result<Vec<U>, SubparError> {
    let sheet_name = U::get_object_name();
    println!("In vec::<{}>::from_excel", sheet_name);

    match get_sheet(excel_object, sheet_name).expect("Failed to get the sheet") {
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
          let value = U::from_excel(to_row(row, &lookup)).expect("Error parsing row");
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

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[derive(Debug, Clone)]
  pub struct Payment {
    guid: String,
    payer: String,
    // payee: String,
    // method: String,
    // amount: f64,
    // comment: Option<String>,
    // date_received: NaiveDateTime,
  }

  impl FromExcel for Payment {
    fn from_excel(excel_object: ExcelObject) -> Result<Self, SubparError> {
      let row = match excel_object {
        ExcelObject::Row(row) => row,
        _ => panic!("Payment did not receive a row"),
      };
      let guid = String::from_excel(
        get_cell(ExcelObject::Row(row.clone()), "guid".to_string())
          .expect("Could not find guid column for Payment"),
      )
      .expect("Error converting Payment.guid to a string");
      let payer = String::from_excel(
        get_cell(ExcelObject::Row(row.clone()), "payer".to_string())
          .expect("Could not find payer column for Payment"),
      )
      .expect("Error converting Payment.guid to a string");
      Ok(Payment {
        guid: guid,
        payer: payer,
      })
    }

    fn get_object_name() -> String {
      "Payment".to_string()
    }
  }

  #[derive(Debug, Clone)]
  pub struct Submission {
    guid: String,
    submitting_org: String,
    // payee: String,
    // method: String,
    // amount: f64,
    // comment: Option<String>,
    // date_received: NaiveDateTime,
  }

  impl FromExcel for Submission {
    fn from_excel(excel_object: ExcelObject) -> Result<Self, SubparError> {
      let row = match excel_object {
        ExcelObject::Row(row) => row,
        _ => panic!("Submission did not receive a row"),
      };
      let guid = String::from_excel(
        get_cell(ExcelObject::Row(row.clone()), "guid".to_string())
          .expect("Could not find guid column for Submission"),
      )
      .expect("Error converting Submission.guid to a string");
      let submitting_org = String::from_excel(
        get_cell(ExcelObject::Row(row.clone()), "submitting_org".to_string())
          .expect("Could not find submitting_org column for Submission"),
      )
      .expect("Error converting Submission.submitting_org to a string");
      Ok(Submission {
        guid: guid,
        submitting_org: submitting_org,
      })
    }

    fn get_object_name() -> String {
      "Submission".to_string()
    }
  }

  #[derive(Debug, Clone)]
  pub struct DB {
    payments: Vec<Payment>,
    submissions: Vec<Submission>,
  }

  impl FromExcel for DB {
    fn from_excel(excel_object: ExcelObject) -> Result<DB, SubparError> {
      let path = match excel_object {
        ExcelObject::FilePath(path) => path,
        _ => panic!("Error: Received something other than a file path for DB"),
      };

      // Ugly, but we can only use the workbook once per sheet since it cannot be cloned
      let payments = match open_workbook(ExcelObject::FilePath(path.clone())) {
        Ok(wb) => Vec::<Payment>::from_excel(wb).expect("Couldn't make the vector of payments"),
        Err(_) => panic!("Could not open the workbook for Payments"),
      };
      let submissions = match open_workbook(ExcelObject::FilePath(path.clone())) {
        Ok(wb) => {
          Vec::<Submission>::from_excel(wb).expect("Couldn't make the vector of submissions")
        }
        Err(_) => panic!("Could not open the workbook for Submission"),
      };

      Ok(DB {
        payments: payments,
        submissions: submissions,
      })
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

// match {
//   ExcelObject::Sheet(sheet) => {
//     let mut rows = sheet.rows();
//     let mut lookup: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
//     if let Some(headers) = rows.next() {
//         for (i, cell) in headers.iter().enumerate() {
//             match cell {
//                 calamine::DataType::String(value) => {
//                     lookup.insert(value.to_lowercase().trim().to_string(), i);
//                 }
//                 calamine::DataType::Empty => (),
//                 _ => println!("Cell '{:?}' is not a string", cell),
//             }
//         }
//     }

//     let mut result: Vec<Payment> = Vec::new();
//     for row in rows {
//       let excel_row = to_row(row, &lookup);

//       let guid = String::from_excel(get_cell(excel_row, "guid".to_string()).expect("Could not find guid column for Payment")).expect("Error converting Payment.guid to a string");
//       result.push(Payment{guid: guid});
//     }
//     result
//   },
//   _ => panic!("We expected an excel sheet here")
// };
