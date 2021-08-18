//! Traits and functions associated with a given instance
//!
//! Commonize the interface for manipulating an instance of a tabular.

use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::sheet::Sheet;

/// Whether the current instance is the primary owner of the workbook
///
/// Since this is tabular data, it's going to be treated as brittle. Only handle on instance may
/// operating on a workbook at a time. Anything more complex should be using some other solution.
///
/// Creating locking on a particular resource is difficult and beyond my current scope given there
/// are multiple ways to access a workbook, such as cli or running the same app multiple times. The
/// SubparServer solves part of this as it can lock instances by their UUID
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum ConnectionState {
  /// The current instance has "locked" the workbook for its own use
  Open,
  /// The pipeline to the instance is not active and anybody may grab it
  Closed,
  /// There is somebody else using this workbook instance
  Blocked,
  /// Waiting for the workbook to become available
  Pending,
}

impl Default for ConnectionState {
  fn default() -> ConnectionState {
    ConnectionState::Closed
  }
}

/// Generic status of the workbook
///
///
#[derive(Debug, Default)]
pub struct State {
  // Open/Closed
  connection: ConnectionState,
  // Sheets
  sheets: HashMap<String, Box<Sheet>>, // Active Sheet
                                       // Current Line Number
}

impl std::fmt::Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl State {
  pub fn new() -> State {
    State {
      connection: ConnectionState::Closed,
      sheets: HashMap::<String, Box<Sheet>>::new(),
    }
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
