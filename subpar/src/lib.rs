// use calamine::{DataType, Xlsx};
// use chrono::{NaiveDateTime, TimeZone, Utc};

#[doc(hidden)]
pub use subpar_derive::FromExcel;

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone)]
pub enum SubparError {
  InvalidExcelObject(String),
  InvalidPath(String),
  FileReadOnly(String),
  UnexpectedError(String),
}

/// Wrapper for the various types of Excel Resources
///
/// This allows us to generically iterate through the conversions
pub enum ExcelObject {
  Cell,
  Sheet,
  Row,
  FilePath(String),
  Workbook(calamine::Xlsx<std::io::BufReader<std::fs::File>>),
}

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

  fn from_excel(_excel_object: ExcelObject) -> Result<String, SubparError> {
    panic!("String.from_excel is not implemented".to_string())
  }

  fn get_object_name() -> String {
    panic!("Tried get_sheet_name for a String, which makes no sense")
  }
}

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
      let _wb = match excel_object {
        ExcelObject::FilePath(_) => match open_workbook(excel_object) {
          Ok(ExcelObject::Workbook(wb)) => wb,
          Ok(_) => panic!("Impossible - got an OK/non Workbook object from Subpar::open_workbook"),
          Err(err) => panic!(format!("Error opening the workbook: {:#?}", err)),
        },
        ExcelObject::Workbook(wb) => wb,
        _ => panic!("This cannot create a workbook. Fix the branch or the code"),
      };
      Ok(DB {
        payments: Vec::default(),
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

/*

fn get_cell_value(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Result<DataType, String> {
  // Find the position of the value of the row
  match lookup.get(&name.to_ascii_lowercase()) {
    Some(index) => return Ok(row[*index].clone()),
    None => {
      println!("\n\n!!!!\nFailed column lookup: {:#?}", lookup);
      return Err(String::from("Could not find column named '") + name + "'");
    }
  };
}


impl FromExcel for NaiveDateTime {
  fn from_excel(excel_date: f64) -> NaiveDateTime {
  // let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
  // assert_eq!(Utc.timestamp(61, 0), dt);

  // var basedate = new Date(1899, 11, 30, 0, 0, 0); // 2209161600000
  // Ported from
  // https://github.com/SheetJS/js-xlsx/blob/3438923e5138f10de0aa70b35a8f56eedcfc320d/bits/20_jsutils.js#L34-L45
  let basedate = Utc.ymd(1899, 11, 30).and_hms(0, 0, 0).naive_utc();
  // println!("The ExcelDate is {:?} and the BaseDate is: {:?}", excel_date, basedate);
  // println!("The parsed date is {:?}", basedate + chrono::Duration::days(excel_date as i64 + 31));
  basedate + chrono::Duration::days(excel_date as i64 + 31)

  // var dnthresh = basedate.getTime() + (new Date().getTimezoneOffset() - basedate.getTimezoneOffset()) * 60000;
  // function datenum(v/*:Date*/, date1904/*:?boolean*/)/*:number*/ {
  //   var epoch = v.getTime();
  //   if(date1904) epoch -= 1462*24*60*60*1000;
  //   return (epoch - dnthresh) / (24 * 60 * 60 * 1000);
  // }
  // function numdate(v/*:number*/)/*:Date*/ {
  //   var out = new Date();
  //   out.setTime(v * 24 * 60 * 60 * 1000 + dnthresh);
  // 	return out;
  // }
}

pub fn cell_to_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> NaiveDateTime {
  f64_to_date(cell_to_f64(lookup, row, name))
}

pub fn cell_to_opt_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<NaiveDateTime> {
  match cell_to_opt_f64(lookup, row, name) {
    Some(days) => Some(f64_to_date(days)),
    None => None,
  }
}

pub fn cell_to_string(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> String {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value,
    Ok(DataType::Float(value)) => value.to_string(),
    Ok(DataType::Int(value)) => value.to_string(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value),
    Ok(DataType::Float(value)) => Some(value.to_string()),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Option String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_f32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f32 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<f32>().unwrap(),
//     Ok(DataType::Float(value)) => value as f32,
//     Ok(DataType::Int(value)) => value as f32,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_f32(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<f32> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<f32>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as f32),
//     Ok(DataType::Int(value)) => Some(value as f32),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_f64(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f64 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<f64>().unwrap(),
    Ok(DataType::Float(value)) => value,
    Ok(DataType::Int(value)) => value as f64,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_f64(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<f64> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<f64>().unwrap()),
    Ok(DataType::Float(value)) => Some(value),
    Ok(DataType::Int(value)) => Some(value as f64),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell into f64: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_i32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i32 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<i32>().unwrap(),
    Ok(DataType::Float(value)) => value as i32,
    Ok(DataType::Int(value)) => value as i32,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_i32(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<i32> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<i32>().unwrap()),
    Ok(DataType::Float(value)) => Some(value as i32),
    Ok(DataType::Int(value)) => Some(value as i32),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_i16(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i16 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<i16>().unwrap(),
//     Ok(DataType::Float(value)) => value as i16,
//     Ok(DataType::Int(value)) => value as i16,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_i16(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<i16> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<i16>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as i16),
//     Ok(DataType::Int(value)) => Some(value as i16),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_vec_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Vec<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value
      .trim_matches(|c| c == '[' || c == ']')
      .replace(" ", "")
      .split(",")
      .filter(|&x| x != "")
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Vec<String> for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! ;;Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}
fn get_cell_value(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Result<DataType, String> {
  // Find the position of the value of the row
  match lookup.get(&name.to_ascii_lowercase()) {
    Some(index) => return Ok(row[*index].clone()),
    None => {
      println!("\n\n!!!!\nFailed column lookup: {:#?}", lookup);
      return Err(String::from("Could not find column named '") + name + "'");
    }
  };
}

fn f64_to_date(excel_date: f64) -> NaiveDateTime {
  // let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
  // assert_eq!(Utc.timestamp(61, 0), dt);

  // var basedate = new Date(1899, 11, 30, 0, 0, 0); // 2209161600000
  // Ported from
  // https://github.com/SheetJS/js-xlsx/blob/3438923e5138f10de0aa70b35a8f56eedcfc320d/bits/20_jsutils.js#L34-L45
  let basedate = Utc.ymd(1899, 11, 30).and_hms(0, 0, 0).naive_utc();
  // println!("The ExcelDate is {:?} and the BaseDate is: {:?}", excel_date, basedate);
  // println!("The parsed date is {:?}", basedate + chrono::Duration::days(excel_date as i64 + 31));
  basedate + chrono::Duration::days(excel_date as i64 + 31)

  // var dnthresh = basedate.getTime() + (new Date().getTimezoneOffset() - basedate.getTimezoneOffset()) * 60000;
  // function datenum(v/*:Date*/, date1904/*:?boolean*/)/*:number*/ {
  //   var epoch = v.getTime();
  //   if(date1904) epoch -= 1462*24*60*60*1000;
  //   return (epoch - dnthresh) / (24 * 60 * 60 * 1000);
  // }
  // function numdate(v/*:number*/)/*:Date*/ {
  //   var out = new Date();
  //   out.setTime(v * 24 * 60 * 60 * 1000 + dnthresh);
  // 	return out;
  // }
}

pub fn cell_to_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> NaiveDateTime {
  f64_to_date(cell_to_f64(lookup, row, name))
}

pub fn cell_to_opt_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<NaiveDateTime> {
  match cell_to_opt_f64(lookup, row, name) {
    Some(days) => Some(f64_to_date(days)),
    None => None,
  }
}

pub fn cell_to_string(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> String {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value,
    Ok(DataType::Float(value)) => value.to_string(),
    Ok(DataType::Int(value)) => value.to_string(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value),
    Ok(DataType::Float(value)) => Some(value.to_string()),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Option String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_f32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f32 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<f32>().unwrap(),
//     Ok(DataType::Float(value)) => value as f32,
//     Ok(DataType::Int(value)) => value as f32,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_f32(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<f32> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<f32>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as f32),
//     Ok(DataType::Int(value)) => Some(value as f32),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_f64(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f64 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<f64>().unwrap(),
    Ok(DataType::Float(value)) => value,
    Ok(DataType::Int(value)) => value as f64,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_f64(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<f64> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<f64>().unwrap()),
    Ok(DataType::Float(value)) => Some(value),
    Ok(DataType::Int(value)) => Some(value as f64),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell into f64: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_i32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i32 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<i32>().unwrap(),
    Ok(DataType::Float(value)) => value as i32,
    Ok(DataType::Int(value)) => value as i32,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_i32(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<i32> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<i32>().unwrap()),
    Ok(DataType::Float(value)) => Some(value as i32),
    Ok(DataType::Int(value)) => Some(value as i32),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_i16(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i16 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<i16>().unwrap(),
//     Ok(DataType::Float(value)) => value as i16,
//     Ok(DataType::Int(value)) => value as i16,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_i16(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<i16> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<i16>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as i16),
//     Ok(DataType::Int(value)) => Some(value as i16),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_vec_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Vec<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value
      .trim_matches(|c| c == '[' || c == ']')
      .replace(" ", "")
      .split(",")
      .filter(|&x| x != "")
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Vec<String> for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! ;;Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}
fn get_cell_value(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Result<DataType, String> {
  // Find the position of the value of the row
  match lookup.get(&name.to_ascii_lowercase()) {
    Some(index) => return Ok(row[*index].clone()),
    None => {
      println!("\n\n!!!!\nFailed column lookup: {:#?}", lookup);
      return Err(String::from("Could not find column named '") + name + "'");
    }
  };
}

fn f64_to_date(excel_date: f64) -> NaiveDateTime {
  // let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
  // assert_eq!(Utc.timestamp(61, 0), dt);

  // var basedate = new Date(1899, 11, 30, 0, 0, 0); // 2209161600000
  // Ported from
  // https://github.com/SheetJS/js-xlsx/blob/3438923e5138f10de0aa70b35a8f56eedcfc320d/bits/20_jsutils.js#L34-L45
  let basedate = Utc.ymd(1899, 11, 30).and_hms(0, 0, 0).naive_utc();
  // println!("The ExcelDate is {:?} and the BaseDate is: {:?}", excel_date, basedate);
  // println!("The parsed date is {:?}", basedate + chrono::Duration::days(excel_date as i64 + 31));
  basedate + chrono::Duration::days(excel_date as i64 + 31)

  // var dnthresh = basedate.getTime() + (new Date().getTimezoneOffset() - basedate.getTimezoneOffset()) * 60000;
  // function datenum(v/*:Date*/, date1904/*:?boolean*/)/*:number*/ {
  //   var epoch = v.getTime();
  //   if(date1904) epoch -= 1462*24*60*60*1000;
  //   return (epoch - dnthresh) / (24 * 60 * 60 * 1000);
  // }
  // function numdate(v/*:number*/)/*:Date*/ {
  //   var out = new Date();
  //   out.setTime(v * 24 * 60 * 60 * 1000 + dnthresh);
  // 	return out;
  // }
}

pub fn cell_to_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> NaiveDateTime {
  f64_to_date(cell_to_f64(lookup, row, name))
}

pub fn cell_to_opt_date(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<NaiveDateTime> {
  match cell_to_opt_f64(lookup, row, name) {
    Some(days) => Some(f64_to_date(days)),
    None => None,
  }
}

pub fn cell_to_string(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> String {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value,
    Ok(DataType::Float(value)) => value.to_string(),
    Ok(DataType::Int(value)) => value.to_string(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value),
    Ok(DataType::Float(value)) => Some(value.to_string()),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Option String for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_f32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f32 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<f32>().unwrap(),
//     Ok(DataType::Float(value)) => value as f32,
//     Ok(DataType::Int(value)) => value as f32,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_f32(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<f32> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<f32>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as f32),
//     Ok(DataType::Int(value)) => Some(value as f32),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f32 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_f64(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> f64 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<f64>().unwrap(),
    Ok(DataType::Float(value)) => value,
    Ok(DataType::Int(value)) => value as f64,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_f64(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<f64> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<f64>().unwrap()),
    Ok(DataType::Float(value)) => Some(value),
    Ok(DataType::Int(value)) => Some(value as f64),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a f64 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell into f64: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_i32(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i32 {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value.parse::<i32>().unwrap(),
    Ok(DataType::Float(value)) => value as i32,
    Ok(DataType::Int(value)) => value as i32,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

pub fn cell_to_opt_i32(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Option<i32> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => Some(value.parse::<i32>().unwrap()),
    Ok(DataType::Float(value)) => Some(value as i32),
    Ok(DataType::Int(value)) => Some(value as i32),
    Ok(DataType::Empty) => None,
    Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i32 for {}", x, name)),
    Err(x) => panic!(format!(
      "\n!!! Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}

// pub fn cell_to_i16(lookup: &HashMap<String, usize>, row: &[DataType], name: &str) -> i16 {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => value.parse::<i16>().unwrap(),
//     Ok(DataType::Float(value)) => value as i16,
//     Ok(DataType::Int(value)) => value as i16,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

// pub fn cell_to_opt_i16(
//   lookup: &HashMap<String, usize>,
//   row: &[DataType],
//   name: &str,
// ) -> Option<i16> {
//   match get_cell_value(lookup, row, name) {
//     Ok(DataType::String(value)) => Some(value.parse::<i16>().unwrap()),
//     Ok(DataType::Float(value)) => Some(value as i16),
//     Ok(DataType::Int(value)) => Some(value as i16),
//     Ok(DataType::Empty) => None,
//     Ok(x) => panic!(format!("\n!!! Cannot turn {:?} into a i16 for {}", x, name)),
//     Err(x) => panic!(format!(
//       "\n!!! Received error converting cell: {:?} for {}",
//       x, name
//     )),
//   }
// }

pub fn cell_to_vec_string(
  lookup: &HashMap<String, usize>,
  row: &[DataType],
  name: &str,
) -> Vec<String> {
  match get_cell_value(lookup, row, name) {
    Ok(DataType::String(value)) => value
      .trim_matches(|c| c == '[' || c == ']')
      .replace(" ", "")
      .split(",")
      .filter(|&x| x != "")
      .map(|s| s.to_string())
      .collect::<Vec<_>>(),
    Ok(x) => panic!(format!(
      "\n!!! Cannot turn {:?} into a Vec<String> for {}",
      x, name
    )),
    Err(x) => panic!(format!(
      "\n!!! ;;Received error converting cell: {:?} for {}",
      x, name
    )),
  }
}
*/
