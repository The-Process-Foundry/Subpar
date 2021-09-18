//! A custom error wrapper to unify errors and warnings

/// A grouping of the items needed to use the errors
pub mod prelude {
  pub use super::Kind;
  // Alias the error name to make explicit that it should always be the same type
  pub use allwhat::prelude::*;
  pub use anyhow::{Error as AnyhowError, Result};
}

use allwhat::prelude::ErrorGroup;
// use anyhow::{anyhow, Error as AnyhowError};

/// The full set of exceptions that can be raised at any step in this process
///
/// This will be used as the "source" of the SubparError error
#[derive(Debug, thiserror::Error)]
pub enum Kind {
  #[error("GenericError")]
  GenericError,

  #[error("A group of errors assembled from a transaction/iteration loop")]
  ErrorList(ErrorGroup),

  #[error("A situation which should return a unique item returned multiple")]
  AmbiguousResult,

  #[error("An invalid parameter was received by the current function")]
  BadValue,

  #[error("Tried to use an item already in use")]
  Busy,

  #[error("An item being used as a key was not unique")]
  DuplicateKey,

  #[error("Internal error converting one type to another")]
  ConversionError,

  #[error("EmptyWorksheet")]
  EmptyWorksheet,

  #[error("IncorrectExcelObject")]
  IncorrectExcelObject,

  #[error("InvalidCellType")]
  InvalidCellType,

  #[error("Problem with the workbook location value")]
  InvalidLocation,

  #[error("InvalidPath")]
  InvalidPath,

  #[error("The current condition is supposed to be impossible")]
  Impossible,

  #[error("FileReadOnly")]
  FileReadOnly,

  #[error("NetworkError")]
  NetworkError,

  #[error("NotFound")]
  NotFound,

  #[error("NotImplemented")]
  NotImplemented,

  #[error("A required value was not set")]
  NullValue,

  #[error("FloatParseError")]
  FloatParseError,
  #[error("ReadOnly")]
  ReadOnly,
  #[error("UnknownColumn")]
  UnknownColumn,
  #[error("UnknownSheet")]
  UnknownSheet,
  #[error("UnexpectedError")]
  UnexpectedError,
  #[error("ExcelError")]
  ExcelError,
  #[error("SheetsError")]
  SheetsError,
  #[error("WorkbookMismatch")]
  WorkbookMismatch,
  #[error("ParsingError")]
  ParsingError,
  #[error("There a problem with locking an object for read/write due to an uncaught error")]
  RwLockError,
  #[error("An error generated by a CSV reader")]
  CsvError(#[from] ::csv::Error),
  #[error("URL could not be processed")]
  UrlError,
  #[error("JSON (de)serializing Error")]
  JsonError(#[from] serde_json::Error),
}

impl From<ErrorGroup> for Kind {
  fn from(group: ErrorGroup) -> Kind {
    log::warn!("In convert error group to kind");
    Kind::ErrorList(group)
  }
}
