//! Generic Sheet Data
//!
//!

use crate::prelude::*;
use anyhow::Result;

use std::collections::{HashMap, HashSet};

/// Annotating an object as a Sheet
///
/// This is used for traversing a workbook when parsing. Because of aliases
pub trait SubparSheet: std::fmt::Debug {}

/// Describe a worksheets headers
///
/// This serves both as a template expectation and a metadata read
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Headers {
  /// A name to column lookup
  ///
  /// This is derived from the "ordered" field
  values: HashMap<String, Option<usize>>,

  /// Alternate header names
  ///
  /// Sometimes the incoming file may have different names for the same column, so we change the
  /// lookup
  aliases: HashMap<String, String>,

  /// A complete list of headers
  ///
  /// This is the complete list of headers, in the order they should be read/written.
  ordered: Vec<String>,

  /// Whether case should matter when matching headers
  case_insensitive: bool,
}

impl std::fmt::Display for Headers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Headers {
  pub fn new() -> Headers {
    Default::default()
  }

  /// Replace the current list of column names with a new one
  pub fn set_headers(&self, _names: Vec<String>) -> Result<Headers, SubparError> {
    unimplemented!("'set_headers' still needs to be implemented")
  }

  pub fn append(&self, _name: String) -> Result<Headers, SubparError> {
    unimplemented!("'set_headers' still needs to be implemented")
  }

  pub fn insert(&self, _name: String, _position: usize) -> Result<Headers, SubparError> {
    unimplemented!("'insert' still needs to be implemented")
  }

  pub fn alias(&self, _name: String, _alt: String) -> Result<Headers, SubparError> {
    unimplemented!("'insert' still needs to be implemented")
  }
}

/// Information known about a saved sheet and pointers to it
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SheetMetadata {
  name: String,
  current_line: usize,
}

impl std::fmt::Display for SheetMetadata {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl SheetMetadata {}

/// Describe how a row is expected to look
///
/// This sets the expectation
#[derive(Debug, Default)]
pub struct SheetTemplate {
  headers: Headers,
}

impl std::fmt::Display for SheetTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl SheetTemplate {}

#[derive(Debug, Default)]
pub struct Sheet {
  name: String,
  rows: HashSet<Uuid>,
  metadata: SheetMetadata,
  data: Option<Vec<Box<dyn SubparRow>>>,
}

impl std::fmt::Display for Sheet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Sheet {
  // Map a template
}

impl SubparSheet for Sheet {}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Reader {}

impl std::fmt::Display for Reader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Reader {
  /// Create a new instance
  pub fn new() -> Reader {
    unimplemented!("'Reader::new' is not implemented yet")
  }
}
