//! Metadata about a workbook
//!
//! This is both configuration and state information about a workbook. It is used for validation
//! and transformations between row and struct

use crate::local::*;
use anyhow::{Context, Result};

use std::collections::HashMap;

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
  /// The instance was created but has not been initialized. Implies Closed
  New,
  /// The current instance has "locked" the workbook for its own use
  Open(String, Mode),
  /// The pipeline to the instance is not active and anybody may grab it
  Closed,
  /// There is somebody else using this workbook instance
  Blocked,
  /// Waiting for the workbook to become available
  Pending,
  /// This workbook cannot be used due to an error
  Error(String),
}

impl Default for ConnectionState {
  fn default() -> ConnectionState {
    ConnectionState::New
  }
}

/// This is the meta-data of the workbook
///
/// This is where we keep the generic information about sheets, columns, and options for use with
/// all workbook components.
#[derive(Debug, Default)]
pub(crate) struct State {
  /// A workbook name for debugging/logging
  name: String,

  /// Open/Closed
  connection: ConnectionState,

  /// The sheet the workbook is currently pointing at
  active_sheet: Option<String>,

  /// All the known sheets in the workbook, and their underlying metadata (if available)
  sheets: HashMap<String, Box<Sheet>>,

  /// Alternate names a sheet can be known by
  aliases: HashMap<String, String>,
}

impl std::fmt::Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl State {
  pub fn new(name: String) -> State {
    State {
      name,
      connection: ConnectionState::New,
      active_sheet: None,
      sheets: HashMap::<String, Box<Sheet>>::new(),
      aliases: HashMap::<String, String>::new(),
    }
  }

  //--- Actions
  /// Get a SheetProcessor for the given sheet
  ///
  /// TODO: This should be a SheetProcessor, but for the moment just trying to get CSV to work
  pub fn open(&mut self, sheet_name: &String, mode: Mode) -> Result<()> {
    // Check the connection is available
    match &self.connection {
      ConnectionState::New | ConnectionState::Closed => (),
      ConnectionState::Open(active, _) => err_ctx!(
        SubparError::Busy,
        "Sheet {} cannot be opened because sheet {} is already active",
        sheet_name,
        active
      )?,
      _ => unimplemented!("'State::open' still have not implemented all states"),
    }

    let _sheet = self.get_sheet(sheet_name)?;
    self.connection = ConnectionState::Open(sheet_name.clone(), mode);
    // sheet.get_processor().map_err(|err| {
    //   self.connection = ConnectionState::Error(format!(
    //     "Getting processor for sheet {} failed:\n{:#?}",
    //     sheet_name, err
    //   ));
    //   err_ctx!(
    //     err,
    //     "Could not open sheet {} in {:?} mode",
    //     sheet_name,
    //     mode
    //   )
    // })
    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    match self.connection {
      ConnectionState::Open(_, _) => self.connection = ConnectionState::Closed,
      _ => err_ctx!(
        SubparError::Impossible,
        "State tried to close the active sheet, but none were open"
      )?,
    }
    Ok(())
  }

  //--- Sheet management
  pub fn list_sheets(&self) -> Vec<String> {
    self.sheets.keys().map(|key| key.clone()).collect()
  }

  /// Let the workbook know about a new sheet
  pub fn add_sheet(&mut self, name: String, sheet: Option<Sheet>) -> Result<()> {
    match self.sheets.insert(
      name.clone(),
      Box::new(sheet.unwrap_or(Sheet::new(&name, None))),
    ) {
      None => Ok(()),
      Some(_) => Err(SubparError::DuplicateKey).ctx(format!(
        "A sheet with the name {} already exists in the workbook {}",
        name, self.name,
      ))?,
    }
  }

  /// Get or create a mutable blank sheet
  ///
  /// This gets a reference to the contents of the RC. If not found
  fn get_sheet(&mut self, sheet_name: &String) -> Result<&Box<Sheet>> {
    let sheet = self
      .sheets
      .get(sheet_name)
      .ok_or(SubparError::NotFound)
      .ctx(format!("Could not find sheet '{}'", sheet_name))?;
    Ok(sheet)
  }

  /// Apply a template to a sheet
  pub fn _add_template<Row: SubparRow>(
    &mut self,
    sheet_name: &String,
    modes: Vec<Mode>,
  ) -> Result<()> {
    let sheet = self.get_sheet(sheet_name)?;

    sheet.add_template::<Row>(modes)
  }
}
