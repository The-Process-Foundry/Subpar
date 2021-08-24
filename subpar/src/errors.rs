// use anyhow::Result;
use thiserror::Error;

use std::boxed::Box;

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

/// Hold flattened results returned from processing an iterator
#[derive(Debug)]
pub struct SplitResult<T, E>
where
  T: std::fmt::Debug,
  E: std::fmt::Debug + std::error::Error,
{
  success: Vec<T>,
  error: Option<ErrorGroup<E>>,
}

impl<T, E> SplitResult<T, E>
where
  T: std::fmt::Debug,
  E: std::fmt::Debug + std::error::Error,
{
  pub fn new() -> SplitResult<T, E> {
    SplitResult {
      success: vec![],
      error: None,
    }
  }

  /// Place the result of applying the function to each
  pub fn map<B, F>(iter: impl Iterator<Item = B>, func: F) -> Self
  where
    F: Fn(B) -> Result<T, E>,
  {
    let mut success = vec![];
    let mut error = ErrorGroup::new();

    for item in iter {
      match func(item) {
        Ok(val) => success.push(val),
        Err(err) => error.push(err),
      }
    }

    SplitResult {
      success,
      error: match error.len() {
        0 => None,
        _ => Some(error),
      },
    }
  }

  /// Get the values returned
  ///
  /// This is essentially the '?'/Try operator and will be replaced by it
  pub fn as_result<F: From<ErrorGroup<E>>>(self) -> Result<Vec<T>, F> {
    match self.error {
      Some(e) => Err(From::from(e)),
      None => Ok(self.success),
    }
  }
}

/// An error accumulator
///
/// This is intended to enumerate all the errors found in a transaction rather than failing on
/// the first
#[derive(Debug, Clone)]
pub struct ErrorGroup<E> {
  errors: Vec<E>,
}

impl<E> ErrorGroup<E> {
  pub fn new() -> ErrorGroup<E> {
    ErrorGroup { errors: vec![] }
  }

  pub fn append(mut self, error: E) -> ErrorGroup<E> {
    self.errors.push(error);
    self
  }

  pub fn push(&mut self, error: E) -> () {
    self.errors.push(error);
  }

  /// Pull the error out from the result and append it to the group
  pub fn extract_err<T, F: Into<E>>(&mut self, result: Result<T, F>) -> Result<T, ()> {
    match result {
      Ok(t) => Ok(t),
      Err(err) => {
        self.push(err.into());
        Err(())
      }
    }
  }

  pub fn len(&self) -> usize {
    self.errors.len()
  }

  pub fn flatten(_list: impl Iterator<Item = E>) -> ErrorGroup<E> {
    unimplemented!("'' still needs to be implemented")
  }

  /// Unwrap a list of results, splitting it into unwrapped values and an optional flattened error
  ///
  /// If there are no errors, an Ok(()) is returned by the error
  pub fn unwrap_all<'a, T>(results: impl Iterator<Item = Result<T, E>>) -> (Vec<T>, Option<Self>) {
    let mut result = vec![];
    let mut errors = ErrorGroup::new();
    for item in results {
      match item {
        Ok(x) => result.push(x),
        Err(err) => errors.push(err),
      }
    }

    match errors.len() {
      0 => (result, None),
      _ => (result, Some(errors)),
    }
  }
}

// impl Clone for ErrorGroup<anyhow::Error> {
//   fn clone(&self) -> Self {
//     unimplemented!("'AnyhowError.clone' still needs to be implemented")
//   }
// }

/// The full set of exceptions that can be raised at any step in this process
#[derive(Debug, Clone, Error)]
pub enum SubparError {
  #[error("GenericError")]
  GenericError,
  #[error("A group of errors assembled from a transaction/iteration loop")]
  ErrorList(Box<ErrorGroup<SubparError>>),
  #[error("A situation which should return one item returned multiple")]
  AmbiguousResult,
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
  #[error("There a problem with locking an object for read/write due to an uncaught error")]
  RwLockError,
}

impl SubparError {
  pub fn to_group(group: ErrorGroup<anyhow::Error>) -> SubparError {
    From::from(group)
  }
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

impl From<anyhow::Error> for SubparError {
  fn from(err: anyhow::Error) -> SubparError {
    match err.downcast::<SubparError>() {
      Ok(e) => e,
      Err(msg) => panic!("Couldn't convert anyhow::Error to SubparError: {:?}", msg),
    }
  }
}

impl From<std::sync::PoisonError<crate::prelude::State>> for SubparError {
  fn from(_err: std::sync::PoisonError<crate::prelude::State>) -> SubparError {
    SubparError::RwLockError
  }
}

impl From<serde_json::Error> for SubparError {
  fn from(err: serde_json::Error) -> SubparError {
    unimplemented!("'' still needs to be implemented")
  }
}

/// Pausing on this, as it currently works
impl<E> From<ErrorGroup<E>> for SubparError {
  fn from(err: ErrorGroup<E>) -> SubparError {
    let group = ErrorGroup::new();

    for _x in err.errors {
      // match x.downcast::<SubparError>() {
      //   Ok(e) => group.push(e),
      //   Err(msg) => panic!("Couldn't convert anyhow::Error to SubparError: {:?}", msg),
      // }
    }
    SubparError::ErrorList(Box::new(group))
  }
}

/*
impl From<ErrorGroup<anyhow::Error>> for SubparError {
  fn from(err: ErrorGroup<anyhow::Error>) -> SubparError {
    let mut group = ErrorGroup::new();
    for x in err.errors {
      group.push(From::from(x))
    }
    SubparError::ErrorList(Box::new(group))
  }
}

impl From<ErrorGroup<SubparError>> for SubparError {
  fn from(group: ErrorGroup<SubparError>) -> SubparError {
    SubparError::ErrorList(Box::new(group))
  }
}
*/

// impl<E> From<ErrorGroup<E>> for SubparError {
//   fn from(group: ErrorGroup<E>) -> SubparError {}
// }

// impl From<ErrorGroup<SubparError>> for SubparError {
//   fn from(err: ErrorGroup<SubparError>) -> SubparError {
//     let converted: ErrorGroup<anyhow::Error> = err;
//     SubparError::ErrorList(Box::new(converted))
//   }
// }

/*
TODO: Move this code into anyhow and figure out how to make it work
/// Handle the ? operations for SplitResult
///
/// This is experimental, but handy enough so I'm going to use the current version, as defined here
/// Current implementation: https://github.com/rust-lang/rfcs/pull/1859
/// In progress V2 https://github.com/rust-lang/rust/issues/84277
/// Example: https://rust-lang.github.io/rfcs/3058-try-trait-v2.html


use std::ops::{ControlFlow, FromResidual, Try};


impl<T, E> Try for SplitResult<T, E>
where
  T: std::fmt::Debug,
  E: std::fmt::Debug,
{
  type Output = Vec<T>;
  type Residual = <Result<T, SubparError> as Try>::Residual;

  fn from_output(output: Self::Output) -> Self {
    SplitResult {
      success: output,
      error: None,
    }
  }

  fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
    match self.error {
      Some(err) => ControlFlow::Break(Err(From::from(err))),
      None => ControlFlow::Continue(self.success),
    }
  }
}

impl<T, E> FromResidual<Result<!, SubparError>> for SplitResult<T, E>
where
  T: std::fmt::Debug,
  E: std::fmt::Debug + From<SubparError>,
{
  fn from_residual(residual: Result<!, E>) -> Self {
    match residual {
      Err(e) => SplitResult {
        success: vec![],
        error: Some(From::from(e)),
      },
    }
  }
}

impl From<SubparError> for ErrorGroup<anyhow::Error> {
  fn from(err: SubparError) -> Self {
    match &err {
      SubparError::ErrorList(e) => Box::into_inner(e.to_owned()),
      _ => ErrorGroup { errors: vec![err] },
    }
  }
}

*/
