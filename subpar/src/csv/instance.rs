//! Implementation of a CSV backed workbook

use crate::local::*;
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::path::PathBuf;

use crate::base::instance::*;
use helpers::*;

/// Configuration settings for a CSV instance
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Options {
  delimiter: u8,
  has_headers: bool,
}

impl std::fmt::Display for Options {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Default for Options {
  fn default() -> Options {
    Options {
      delimiter: b',',
      has_headers: true,
    }
  }
}

impl Options {
  pub fn new() -> Options {
    Default::default()
  }
}

/// A thin wrapper around the csv module, shaping it to be used by Subpar
///
/// This design emulates a workbook by including all of the files with a "csv" extension in the
/// root directory
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CsvWorkbook {
  /// A unique identifier for the workbook
  guid: Uuid,

  /// A simple debugging/logging name
  name: String,

  /// The default path for the individual "sheets"
  ///
  /// This is a canonical local directory where new sheets are written by default. The "scan"
  /// function checks this path for new files.
  directory: PathBuf,

  /// The individual sheet locations
  ///
  /// These can be anywhere on the local filesystem. The string is the file_stem of the "physical"
  /// file. This is a lookup for the workbook itself, which houses the RowTemplate. If not
  /// canonical, the path is relative to self.directory.
  sheets: HashMap<String, PathBuf>,

  /// CSV Specific Options
  ///
  /// These are designed to be defaults, and some can be overridden at sheet/row level
  options: Options,
}

impl std::fmt::Display for CsvWorkbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl CsvWorkbook {
  fn parse_path(path: &str) -> Result<(PathBuf, Vec<PathBuf>)> {
    // FIXME: This only works if the directory exists. Either it needs to be added before creating
    //        the workbook, on write, or check harder
    let abs_path = helpers::canonicalize(path)?;
    if abs_path.is_dir() {
      Ok((abs_path, vec![]))
    } else {
      let directory = Box::new(abs_path.parent().unwrap().clone());
      Ok((directory.to_path_buf(), vec![abs_path]))
    }
  }

  /// Builds a new instance and validates the location
  pub fn new(path: &str) -> Result<CsvWorkbook> {
    let (directory, csv_files) = CsvWorkbook::parse_path(path)?;
    let guid = path_to_id(&directory)?;
    let name = guid.to_string();

    let mut sheets = HashMap::new();
    for file in csv_files {
      let key = to_sheet_name(&file)?;
      match sheets.insert(key.clone().to_string(), file.to_owned()) {
        None => (),
        Some(_) => Err(SubparError::DuplicateKey).ctx(format!(
          "There were two CSV files with the same name '{}' in directory '{}'",
          key,
          path_to_str(&directory)?
        ))?,
      }
    }

    Ok(CsvWorkbook {
      guid,
      name,
      directory,
      sheets,
      options: Options::new(),
    })
  }

  //---  Option setters/getters
  pub fn set_delimeter(&mut self, delimeter: u8) -> Result<CsvWorkbook> {
    self.options.delimiter = delimeter;
    Ok(self.clone())
  }

  /// Change the workbooks printable name
  pub fn set_name(self, name: String) -> Result<CsvWorkbook> {
    Ok(CsvWorkbook { name, ..self })
  }
}

impl SubparWorkbook for CsvWorkbook {
  /// Get the workbook's UUID
  fn get_id(&self) -> Result<Uuid> {
    Ok(self.guid.clone())
  }

  /// A pretty name for debugging/logging
  ///
  /// The default here is to use the guid
  fn get_name(&self) -> Result<String> {
    Ok(self.name.clone())
  }

  /// Return a list of registered sheet names
  fn list_sheets(&self) -> Result<Vec<String>> {
    Ok(self.sheets.keys().map(|key| key.clone()).collect())
  }

  fn get_sheet_accessor(&self, sheet_name: &String) -> Result<SheetAccessor> {
    let path = ok_or!(
      self.sheets.get(sheet_name),
      SubparError::NotFound,
      "Could not get a sheet path for {}",
      sheet_name
    )?;
    Ok(SheetAccessor::Csv(path.clone()))
  }
}
