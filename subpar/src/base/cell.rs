//! An individual cell in a table
//!
//! This is a wrapped value, designed to hold the data in intermediate form

use crate::local::*;
use anyhow::Result;

use std::convert::TryFrom;

use schemars::schema::SchemaObject;
use serde_json::Number;
use serde_json::Value as JsonValue;

/// A wrapper to enclose the raw data received from a reader
///
/// Some forms of reader are given metadata to differentiate between types, so we want to use those
/// as hints. Writers use this for encoding purposes.
#[derive(Debug)]
pub enum CellValue {
  /// Column doesn't exist, though the cell is expected
  Null,
  /// The reader explicitly detected the field as having no value
  Empty,
  /// Completely unprocessed string (such as from a CSV)
  Raw(String),
  /// Any of the numeric subtypes (float, )
  Number(Number),
  /// The default string type
  String(String),
}

#[derive(Debug)]
pub struct Cell {
  name: String,
  value: CellValue,
}

impl Cell {
  pub fn new(name: String, value: CellValue) -> Cell {
    Cell { name, value }
  }

  // Parse the cell into serde_json value using the schema
  pub fn to_value(self) -> Result<JsonValue> {
    unimplemented!("'to_value' still needs to be implemented")
  }

  // /// Parse from an unknown string into an intermediate form.
  // ///
  // /// This is primarily for the CSV reader. If we don't know the type it's eventually going to
  // /// deserialize into, we have to make an educated guess.
  // pub fn from_str(value: &str) -> Result<Cell> {
  //   match value.len() {
  //     0 => Ok(Cell { value: Value::Null }),
  //     _ => {
  //       let x: Result<Value, serde_json::Error> = serde_json::from_str(value);
  //       let y: Result<Value> = Ok(Value::String(value.to_string()));
  //       Ok(Cell { value: x.or(y)? })
  //     }
  //   }
  // }
}

// Begin the deserializer for the schema
use serde::{de::DeserializeOwned, Deserialize};

impl TryFrom<Cell> for JsonValue {
  type Error = AnyhowError;

  fn try_from(cell: Cell) -> Result<JsonValue> {
    unimplemented!("'Cell.try_from' still needs to be implemented")
  }
}
struct ToValue<'de> {
  input: &'de Cell,
}

// Bug with generics and TryFrom: https://github.com/rust-lang/rust/issues/50133
// pub(crate) struct Wrapper<T>(T);

// impl<T> Wrapper<T> {
//   /// unwrap
//   pub fn unwrap(result: Result<Wrapper<T>>) -> Result<T> {
//     unimplemented!("'unwrap' is not implemented yet")
//   }
// }

// /// As a row keeps it's cells in a box, it's
// use std::convert::TryFrom;
// impl<'de, T: Deserialize<'de>> TryFrom<Box<Cell>> for Wrapper<T> {
//   type Error = SubparError;

//   fn try_from(cell: Box<Cell>) -> Result<Wrapper<T>> {
//     Ok(Wrapper(T::deserialize(*cell)?))
//   }
// }

// macro_rules! from_serde {

//   ($to:ident) => {
//     impl std::convert::TryFrom<Cell> for $to {
//       type Error = SubparError;

//       fn try_from(cell: Cell) -> Result<$to> {
//         let Cell(value) = cell;
//         Ok(value.try_from()?)
//       }
//     }
//   };
//   ($full:ident<$($generic:tt),+>) => {
//     impl<$($generic), +> std::convert::TryFrom<Cell> for $full<$($generic), +>  {
//       type Error = SubparError;

//       fn try_from(cell: Cell) -> Result<$full<$($generic), +>> {
//         let Cell(value) = cell;
//         Ok(value.try_from()?)
//       }
//     }
//   };
// }
