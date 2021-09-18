//! Parameters needed to build an IO connection to a specific sheet
//!
//! These need to be customized for each type of tabular data, as CSV keeps its sheets in different
//! files while Google Sheets uses URL keys.

use crate::local::*;
// use anyhow::Context;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Accessor {
  Csv(PathBuf),
  // SheetsWorkbook
  // SheetsSheet ?
  // ExcelWorkbook
  // ExcelSheet
  // Excel360Workbook
  // Excel360Sheet
}

impl std::fmt::Display for Accessor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Accessor {
  pub fn new_csv(path: &str) -> Accessor {
    Accessor::Csv(PathBuf::from(path))
  }

  /// Changes all relative paths to absolute paths
  ///
  /// URLs are simply filled in (if known)
  /// Files require the existence of a directory. If create_missing is true, they will be created
  /// on the local file system but will not create the file itself.
  pub fn canonicalize(self, _create_missing: bool) -> Result<Accessor> {
    match self {
      Accessor::Csv(path) => Ok(Accessor::Csv(helpers::canonicalize(path)?)),
    }
  }

  /// Get a pretty name as defined by the accessor to use as the default for a workbook
  /// TODO: Move this to Workbook metadata, since the user may want to change it for logging purposes
  pub fn name(&self) -> String {
    match self {
      Accessor::Csv(path) => match helpers::to_sheet_name(path) {
        Ok(t) => t.to_string(),
        Err(_) => {
          let lossy = path.to_string_lossy().to_string();
          log::warn!(
            "Could not figure out a file name for path '{}', using lossy",
            lossy
          );
          lossy
        }
      },
    }
  }
}
