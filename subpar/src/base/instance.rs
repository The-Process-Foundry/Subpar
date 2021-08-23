//! Traits and functions associated with a given instance
//!
//! Commonize the interface for manipulating an instance of a tabular.

use crate::prelude::*;
use anyhow::Result;

/// Define the basic file access mode - read or write
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Mode {
  Read,
  Append,
  Overwrite,
}

impl Default for Mode {
  fn default() -> Mode {
    Mode::Read
  }
}

/// Define the interface for a workbook instance
///
/// This maps directly with Action messages.
/// THINK: Does the builder pattern make sense for this trait?
pub trait SubparWorkbook: std::fmt::Debug {
  /// Get an unique identifier based on the workbook
  fn get_id(&self) -> Result<Uuid>;

  /// A name that can be used for debugging/logging
  fn get_name(&self) -> Result<String>;

  //-- Actions

  // /// This is a validation that the current config can be opened in the given mode
  // /// Lock the workbook for use in the given mode
  // fn open(&self, mode: Mode) -> Result<(), SubparError>;

  // /// Release the lock on the workbook
  // fn close(&self) -> Result<(), SubparError>

  // /// Do an initial scan based on the given configuration
  // fn init(&self) -> Result<State, SubparError>;

  // Compare the internal state with the reality
  // fn scan(&mut self) -> Result<(), SubparError>;

  // /// Add a new sheet
  // fn add_sheet(&self, sheet_name: &str) -> Result<(), SubparError>

  /// Get the current list of known sheets
  fn list_sheets(&self) -> Result<Vec<String>>;

  //-- Data
  //
}
