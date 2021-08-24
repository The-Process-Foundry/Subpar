//! An individual cell in a table
//!
//! This is a wrapped value, designed to hold the data in intermediate form

use crate::prelude::*;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};

// By directly using serde, we can send reads across the wire to remote requestors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
  value: serde_json::Value,
}

impl Cell {
  pub fn new(value: serde_json::Value) -> Cell {
    Cell { value }
  }

  pub fn to_value<T: DeserializeOwned>(self) -> Result<T, SubparError> {
    match serde_json::from_value(self.value) {
      Ok(val) => Ok(val),
      Err(err) => Err(From::from(err)),
    }
  }
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

//   fn try_from(cell: Box<Cell>) -> Result<Wrapper<T>, SubparError> {
//     Ok(Wrapper(T::deserialize(*cell)?))
//   }
// }

// macro_rules! from_serde {

//   ($to:ident) => {
//     impl std::convert::TryFrom<Cell> for $to {
//       type Error = SubparError;

//       fn try_from(cell: Cell) -> Result<$to, SubparError> {
//         let Cell(value) = cell;
//         Ok(value.try_from()?)
//       }
//     }
//   };
//   ($full:ident<$($generic:tt),+>) => {
//     impl<$($generic), +> std::convert::TryFrom<Cell> for $full<$($generic), +>  {
//       type Error = SubparError;

//       fn try_from(cell: Cell) -> Result<$full<$($generic), +>, SubparError> {
//         let Cell(value) = cell;
//         Ok(value.try_from()?)
//       }
//     }
//   };
// }
