//! Generic Sheet Data
//!
//!

use crate::local::*;
use anyhow::{Context, Result};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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
  order_map: HashMap<String, Option<usize>>,

  /// Alternate header names each template may use
  ///
  /// Sometimes the incoming file may have different names for the same column, so we change the
  /// lookup
  aliases: HashMap<Uuid, HashMap<String, String>>,

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
  pub fn set_headers(&self, _names: Vec<String>) -> Result<Headers> {
    unimplemented!("'set_headers' still needs to be implemented")
  }

  pub fn append(&self, _name: String) -> Result<Headers> {
    unimplemented!("'set_headers' still needs to be implemented")
  }

  pub fn insert(&self, _name: String, _position: usize) -> Result<Headers> {
    unimplemented!("'insert' still needs to be implemented")
  }

  pub fn alias(&self, _name: String, _alt: String) -> Result<Headers> {
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

/// How the sheet is accessed.
pub enum SheetAccessor {
  /// CSV requires the exact location of the file
  Csv(PathBuf),
  // /// Google sheets requires local creds and a url to connect to
  // Sheets(PathBuf, Url),
}

/// Abstract data about a sheet
///
/// This tells a reader/writer how the table should look
// THINK: Is cached data useful?
pub struct Sheet {
  /// The name the workbook knows the sheet by
  ///
  /// In GUI based workbooks such as Excel and Sheets this is the name on the tab. For CSV, it is
  /// the file stem (the piece before .csv)
  name: String,

  /// The IO pipeline
  /// FIXME: This is not optional
  _accessor: Option<SheetAccessor>,

  /// This is the expectations for the given sheet
  ///
  /// This can either be calculated from the registered templates or created manually
  base: Option<SheetTemplate>,

  /// Templates that are associated with this sheet and the modes allowed with them
  ///
  /// This is merely a lookup and the template will need to be passed in. All data from the source
  /// is consumed and the untracked data received is thrown out with the row
  _templates: HashMap<Uuid, HashSet<Mode>>,

  /// An amalgam of all the information known about the sheet data
  _metadata: Option<SheetMetadata>,
}

impl std::fmt::Debug for Sheet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Sheet").field("name", &self.name).finish()
  }
}

impl std::fmt::Display for Sheet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Sheet {
  // Map a template
  pub fn new(name: &String, _accessor: Option<SheetAccessor>) -> Sheet {
    Sheet {
      name: name.clone(),
      _accessor,
      base: None,
      _templates: HashMap::new(),
      _metadata: None,
    }
  }

  /// Sets the base template of the sheet
  pub fn set_base<Row: SubparRow>(&mut self) -> Result<()> {
    self.base = Some(Row::get_template());
    Ok(())
  }

  /// Sets up the format of the sheet using a row template.
  ///
  /// This validates that the sheets are compatible if the base is populated, based on the modes
  pub fn add_template<Row: SubparRow>(&self, modes: Vec<Mode>) -> Result<()> {
    let template_id = Row::get_id();
    log::debug!(
      "Applying template '{}' to sheet '{}'",
      template_id,
      self.name
    );

    // must have at least one mode allowed
    if modes.len() == 0 {
      err_ctx!(
        SubparError::BadValue,
        "Sheet templates ({}) must be registered with at least one mode",
        template_id
      )?;
    }

    // Is the template already registered

    Ok(())
  }
}

impl SubparSheet for Sheet {}

/// A wrapper for the reader implementations
#[derive(Debug)]
pub enum ReaderWrapper {
  Csv(CsvReader),
  // Sheets(SheetsReader),
}

impl Default for ReaderWrapper {
  fn default() -> ReaderWrapper {
    unimplemented!("Default is not implemented for ")
  }
}

#[derive(Debug)]
pub struct Reader {
  sheet_name: String,
  state: Rc<std::cell::RefCell<State>>,
  internal: ReaderWrapper,
}

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

/// Wrap up the internal reader (CSV)
impl Iterator for Reader {
  type Item = Row;

  fn next(&mut self) -> Option<Self::Item> {
    unimplemented!("'Reader Iterator.next' still needs to be implemented")
  }
}

/// Tag the readers/writers so they know to close the sheet at the end of life
pub trait SheetModifier {}

/// Add the generics needed to make
macro_rules! is_sheet_modifier {
  ($ty:ty) => {
    impl SheetModifier for $ty {}

    impl Drop for $ty {
      fn drop(&mut self) {
        let _ = (*self.state).borrow_mut().close();
      }
    }
  };
}

is_sheet_modifier! {Reader}
