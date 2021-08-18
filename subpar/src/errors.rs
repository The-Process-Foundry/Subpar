use thiserror::Error;

// FIXME: This is ugly, but this is the intermediate step to converting everything to use anyhow
// #[macro_export]
// macro_rules! to_subpar_error {
//   ($result:expr) => {
//     match $result {
//       Ok(x) => x,
//       Err(err) => {
//         let msg = format!("{:#?}", &err);
//         Err(<SubparError>::from(err)).context(msg)?
//       }
//     }
//   };
// }

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone, Error)]
pub enum SubparError {
  #[error("GenericError")]
  GenericError,
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
  #[error("FileReadOnly")]
  FileReadOnly,
  #[error("NetworkError")]
  NetworkError,
  #[error("NotFound")]
  NotFound,
  #[error("NotImplemented")]
  NotImplemented,
  #[error("NullValue")]
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
  #[error("CSVError")]
  CSVError,
  #[error("WorkbookMismatch")]
  WorkbookMismatch,
  #[error("ParsingError")]
  ParsingError,
}

impl From<wrapi::WrapiError> for SubparError {
  fn from(err: wrapi::WrapiError) -> SubparError {
    match err {
      wrapi::WrapiError::Connection(_) => SubparError::NetworkError,
      wrapi::WrapiError::Json(_) => SubparError::ParsingError,
      wrapi::WrapiError::Http(_) => SubparError::NetworkError,
      wrapi::WrapiError::General(_) => SubparError::UnexpectedError,
    }
  }
}

impl From<chrono::ParseError> for SubparError {
  fn from(_err: chrono::ParseError) -> SubparError {
    SubparError::ParsingError
  }
}

impl From<std::num::ParseIntError> for SubparError {
  fn from(_err: std::num::ParseIntError) -> SubparError {
    SubparError::ParsingError
  }
}

impl From<std::num::ParseFloatError> for SubparError {
  fn from(_err: std::num::ParseFloatError) -> SubparError {
    SubparError::ParsingError
  }
}

impl From<std::string::String> for SubparError {
  fn from(_err: std::string::String) -> SubparError {
    SubparError::GenericError
  }
}
