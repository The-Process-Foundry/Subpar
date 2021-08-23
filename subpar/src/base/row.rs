//! Row operations
//!
//! An individual row on a sheet. This is composed of wrapped/encoded cells. This is the level
//! we place the top level serialization

use crate::prelude::*;
// use anyhow::Result;

use std::boxed::Box;
use std::collections::HashMap;

/// A tag that allows us to link a struct with a row
pub trait SubparRow: std::fmt::Debug {}

#[derive(Debug)]
pub struct Row {
  /// A list of cells, in the order they appear
  cells: Vec<Box<Cell>>,

  /// A lookup of cells
  by_name: HashMap<String, Box<Cell>>,
}

impl std::fmt::Display for Row {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Row {
  /// Create a new instance
  pub fn new() -> Row {
    unimplemented!("'Row::new' is not implemented yet")
  }
}

impl SubparRow for Row {}
