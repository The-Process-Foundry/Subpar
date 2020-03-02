/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone)]
pub enum SubparError {
  EmptyWorksheet(String),
  IncorrectExcelObject(String),
  InvalidCellType(String),
  InvalidPath(String),
  FileReadOnly(String),
  NetworkError(String),
  NotFound(String),
  NotImplemented(String),
  NullValue(String),
  FloatParseError(String),
  ReadOnly(String),
  UnknownColumn(String),
  UnknownSheet(String),
  UnexpectedError(String),
  ExcelError(String),
  SheetsError(String),
  CSVError(String),
  WorkbookMismatch(String),
}

impl From<wrapi::WrapiError> for SubparError {
  fn from(err: wrapi::WrapiError) -> SubparError {
    match err {
      wrapi::WrapiError::Connection(msg) => SubparError::NetworkError(msg),
      wrapi::WrapiError::Json(msg) => SubparError::NetworkError(msg),
      wrapi::WrapiError::Http(msg) => SubparError::NetworkError(msg),
      wrapi::WrapiError::General(msg) => SubparError::UnexpectedError(msg),
    }
  }
}
