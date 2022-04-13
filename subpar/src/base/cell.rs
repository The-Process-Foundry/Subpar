//! An individual cell in a table
//!
//! This is a wrapped value, designed to hold the data in intermediate form

use crate::local::*;

use std::convert::TryFrom;

use schemars::schema::*;
use serde_json::Number;
use serde_json::Value as JsonValue;

/// Annotate cells, so we can also add custom deserialization
/// TODO: Change Serialize/Deserialize into From/Into traits
pub trait SubparCell: Serialize + serde::de::DeserializeOwned + Clone + Send + Sync {}

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

impl CellValue {
  /// Change the cell into the expected Json type
  fn convert(&self, i_type: Option<&InstanceType>) -> Result<JsonValue> {
    // pub enum InstanceType {
    //   Null,
    //   Boolean,
    //   Object,
    //   Array,
    //   Number,
    //   String,
    //   Integer,

    log::trace!("Trying to convert {:?}: {:?}", i_type, self);
    match i_type {
      Some(InstanceType::Null) => match self {
        CellValue::Null | CellValue::Empty => Ok(JsonValue::Null),
        _ => Err(err!(
          ConversionError,
          "Cannot reasonably convert a value into a null. Try again"
        )),
      },
      Some(InstanceType::Number) => match self {
        CellValue::String(val) | CellValue::Raw(val) => {
          use core::str::FromStr;
          Ok(JsonValue::Number(err_into!(
            serde_json::Number::from_str(val),
            "Failed to convert {:?} into number",
            val
          )?))
        }
        CellValue::Number(num) => Ok(JsonValue::Number(num.clone())),
        _ => Err(err!(
          ConversionError,
          "Cannot reasonably convert a value into a number. Try again"
        )),
      },
      Some(InstanceType::Integer) => match self {
        CellValue::String(val) | CellValue::Raw(val) => {
          let int = err_into!(val.parse::<i64>())?;
          Ok(JsonValue::Number(serde_json::Number::from(int)))
        }
        CellValue::Number(num) => Ok(JsonValue::Number(num.clone())),
        _ => Err(err!(
          ConversionError,
          "Cannot reasonably convert a value into a number. Try again"
        )),
      },
      Some(InstanceType::String) => match self {
        CellValue::Null => Err(err!(
          BadValue,
          "Strings are not allowed to be null. Try again"
        )),
        CellValue::Raw(val) | CellValue::String(val) => Ok(JsonValue::String(val.clone())),
        CellValue::Number(num) => Ok(JsonValue::Number(num.clone())),
        CellValue::Empty => Ok(JsonValue::Null),
      },
      // None just returns what serde_json guessed
      None => match self {
        CellValue::Raw(val) | CellValue::String(val) => Ok(JsonValue::String(val.clone())),
        CellValue::Number(num) => Ok(JsonValue::Number(num.clone())),
        CellValue::Null | CellValue::Empty => Ok(JsonValue::Null),
      },
      _ => unimplemented!("'Other conversions' still needs to be implemented"),
    }
  }
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

  pub fn name(&self) -> String {
    self.name.clone()
  }

  // Parse the cell into serde_json value using the schema, validating it as needed
  pub fn to_value(&self, schema: &SchemaObject) -> Result<JsonValue> {
    match &schema.instance_type {
      Some(SingleOrVec::Vec(i_types)) => {
        let mut result = Err(err!(
          NotFound,
          "Could not find a valid type to convert the cell into: {:#?}",
          self
        ));
        for i_type in i_types {
          if let Ok(val) = self.value.convert(Some(i_type)) {
            result = Ok(val);
            break;
          };
        }
        let msg = format!(
          "Could not convert cell {:?} into any of {:?}",
          self, i_types
        );
        if result.is_err() {
          log::debug!("{}", msg);
        };
        result.context(msg)
      }
      Some(SingleOrVec::Single(i_type)) => self.value.convert(Some(i_type)),
      None => self.value.convert(None),
    }
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
// use serde::{de::DeserializeOwned, Deserialize};

impl TryFrom<Cell> for JsonValue {
  type Error = SubparError;

  fn try_from(_cell: Cell) -> Result<JsonValue> {
    unimplemented!("'Cell.try_from' still needs to be implemented")
  }
}

pub struct ToValue<'de> {
  _input: &'de Cell,
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

// Subpar Cell implementations for the basics
impl SubparCell for bool {}
impl SubparCell for isize {}
impl SubparCell for i8 {}
impl SubparCell for i16 {}
impl SubparCell for i32 {}
impl SubparCell for i64 {}
impl SubparCell for i128 {}
impl SubparCell for usize {}
impl SubparCell for u8 {}
impl SubparCell for u16 {}
impl SubparCell for u32 {}
impl SubparCell for u64 {}
impl SubparCell for u128 {}
impl SubparCell for f32 {}
impl SubparCell for f64 {}
impl SubparCell for char {}

impl SubparCell for String {}

// impl<'de: 'a, 'a, T> SubparCell for &'a T where T: SubparCell {}
impl<T> SubparCell for Vec<T> where T: SubparCell {}
