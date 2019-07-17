#[allow(unused_imports)]
#[macro_use]
extern crate subpar_derive;

// Re-export the macros so we only have to do one import/use in clients
#[doc(hidden)]
pub use subpar_derive::*;

/// Convert a row from a given table into the given struct
pub trait FromExcel {
  fn from_excel_row(file_name: String) -> Self;
}


#[derive(FromExcel)]
pub struct Payment {
  guid: String,
  // payer: String,
  // payee: String,
  // method: String,
  // amount: f64,
  // comment: Option<String>,
  // date_received: NaiveDateTime,
}

// /// Convert the struct into an excel row
// pub trait ToExcel {
//   fn to_excel();
// }

// /// Take the given struct and load it up with data from an excel spreadsheet
// pub trait FromWorkbook {
//   fn from_excel(file_name: String) -> Self;
// }
