//! Definition and options for CSV files

use crate::prelude::*;
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::path::PathBuf;

use crate::base::instance::*;
use helpers::*;

/// Configuration settings for a CSV instance
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Options {
  delimiter: u8,
  multisheet: bool,
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
      multisheet: false,
    }
  }
}

impl Options {
  pub fn new() -> Options {
    Default::default()
  }
}

/// Some data management tools for cleaning up the data
///
/// These are mostly wrappers to existing methods add on better validation errors without cluttering
/// up the business code.
pub mod helpers {
  use crate::prelude::*;
  use anyhow::{Context, Result};

  use std::path::{Path, PathBuf};

  /// Create a canonical path buffer from a string
  pub fn canonicalize(path: &str) -> Result<PathBuf> {
    Path::new(&path).canonicalize().context(format!(
      "Could not canonicalize path '{}'. Current directory is: {:?}",
      path,
      std::env::current_dir()?
    ))
  }

  /// Check a file extension matches in a case insensitive fashion
  ///
  /// This can potentially fail if the extension cannot be converted to ascii. Erroring is more
  /// useful than calling it "false"
  pub fn cmp_extension(path: &PathBuf, extension: &str) -> Result<bool> {
    path.extension().map_or(Ok(false), |x| {
      x.to_ascii_lowercase()
        .to_str()
        .map(|ext| ext == extension)
        .ok_or(SubparError::ConversionError)
        .context("'csv::Location::new' could not convert extension to ascii")
    })
  }

  /// Takes a path and returns the file stem to be used as a name
  pub fn to_sheet_name(path: &PathBuf) -> Result<&str> {
    log::debug!("File Stem: {:?}", path.file_stem());
    path
      .file_stem()
      .ok_or_else(|| SubparError::InvalidLocation)
      .context(format!(
        "csv::to_sheet_name file {:?} did not have a file stem",
        path
      ))?
      .to_str()
      .clone()
      .ok_or(SubparError::ConversionError)
      .context("'csv::Location::to_sheet_name' could not convert file name to ascii")
  }

  /// Find all the files in a directory with a CSV extension
  pub fn list_csv_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    if !path.is_dir() {
      Err(SubparError::InvalidPath).context("The path {:?} is not a directory")?
    };

    let mut result = vec![];
    for entry in path.read_dir()? {
      match entry {
        Ok(e) => match cmp_extension(&e.path(), "csv")? {
          true => result.push(e.path().clone()),
          false => (),
        },
        Err(_) => {
          entry.context("error in list_csv_files")?;
          ()
        }
      }
    }
    Ok(result)
  }

  pub fn path_to_str(path: &PathBuf) -> Result<&str> {
    path
      .to_str()
      .clone()
      .ok_or(SubparError::ConversionError)
      .context("csv::helpers::path_to_str could not convert file name to ascii")
  }
}

/// A thin wrapper around the csv module, shaping it to be used by Subpar
///
/// This design emulates a workbook by including all of the files with a "csv" extension in the
/// root directory
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CsvWorkbook {
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
  fn parse_directory(path: &str) -> Result<(PathBuf, Vec<PathBuf>)> {
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
    let (directory, csv_files) = CsvWorkbook::parse_directory(path)?;

    let mut sheets = HashMap::new();
    for file in csv_files {
      let key = to_sheet_name(&file)?;
      match sheets.insert(key.clone().to_string(), file.to_owned()) {
        None => (),
        Some(_) => Err(SubparError::DuplicateKey).context(format!(
          "There were two CSV files with the same name '{}' in directory '{}'",
          key,
          path_to_str(&directory)?
        ))?,
      }
    }

    Ok(CsvWorkbook {
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
}

impl SubparWorkbook for CsvWorkbook {
  fn get_id(&self) -> Result<Uuid> {
    Ok(Uuid::new_v5(
      &Uuid::NAMESPACE_OID,
      ["csv", "|", path_to_str(&self.directory)?]
        .iter()
        .fold("".to_string(), |mut acc, item| {
          acc.push_str(item);
          acc
        })
        .as_bytes(),
    ))
  }

  fn scan(&self, state: State) -> Result<State> {
    Ok(state)
  }
}
