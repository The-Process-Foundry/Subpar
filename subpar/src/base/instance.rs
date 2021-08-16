//! Traits and functions associated with a given instance
//!
//! Commonize the interface for manipulating an instance of a tabular.

use anyhow::Result;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic status of the workbook
///
///
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub struct State {
  // Open/Closed
// Active Sheet
// Current Line Number
}

impl std::fmt::Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl State {
  pub fn new() -> State {
    State {}
  }
}

/// Define the basic file access mode - read or write
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Mode {
  Read,
  Write,
}

impl Default for Mode {
  fn default() -> Mode {
    Mode::Read
  }
}

/// A mapping to a live workbook
pub trait WbInstance: Sized {
  type LocationType;

  /// Create a new instance with default options
  fn new(loc: Self::LocationType, mode: Mode) -> Result<Self>;

  /// Get the current state of the workbook instance
  ///
  /// This checks things like network connectivity, file existence, workbook metadata and the like.
  /// It will throw errors for misconfigurations and errors that cannot be auto-corrected
  fn get_state(&self) -> Result<State>;

  /// Get a unique identifier for an instance
  fn get_id(&self) -> Result<Uuid>;
}
