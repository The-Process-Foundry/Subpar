//! Some data management tools for cleaning up the data
//!
//! These are mostly wrappers to existing methods add on better validation errors without cluttering
//! up the business logic.

use crate::local::*;
use anyhow::Context;

use std::path::PathBuf;

/// Create a canonical PathBuf
pub fn canonicalize(buf: PathBuf) -> Result<PathBuf> {
  let path = buf.as_path();

  let exists =
    path.is_file() || path.is_dir() || { path.parent().map(|x| x.is_dir()).unwrap_or(false) };
  if !exists {
    Err(Kind::InvalidPath).context(format!(
      "path {} does not exist on the system and cannot be canonicalized",
      path.to_str().unwrap()
    ))?
  } else {
    Ok(path.canonicalize().context(format!(
      "Could not canonicalize path '{}'. Current directory is: {:?}",
      path.to_str().unwrap(),
      std::env::current_dir()
    ))?)
  }
}

/// Takes a path and returns the file stem to be used as a name
pub fn to_sheet_name(path: &PathBuf) -> Result<&str> {
  log::debug!("File Stem: {:?}", path.file_stem());
  path
    .file_stem()
    .ok_or(Kind::InvalidLocation)
    .context(format!(
      "csv::to_sheet_name file {:?} did not have a file stem",
      path
    ))?
    .to_str()
    .clone()
    .ok_or(Kind::ConversionError)
    .context("'csv::Location::to_sheet_name' could not convert file name to ascii")
}

/*
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
      .ctx("'csv::Location::new' could not convert extension to ascii")
  })
}


/// Find all the files in a directory with a CSV extension
pub fn list_csv_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
  if !path.is_dir() {
    Err(SubparError::InvalidPath).ctx("The path {:?} is not a directory")?
  };

  let mut result = vec![];
  for entry in path.read_dir()? {
    match entry {
      Ok(e) => match cmp_extension(&e.path(), "csv")? {
        true => result.push(e.path().clone()),
        false => (),
      },
      Err(_) => {
        entry.ctx("error in list_csv_files")?;
        ()
      }
    }
  }
  Ok(result)
}

/// Convert a Path/PathBuf to a string, failing if it can't
pub fn path_to_str(path: &PathBuf) -> Result<&str> {
  path
    .to_str()
    .clone()
    .ok_or(SubparError::ConversionError)
    .ctx("csv::helpers::path_to_str could not convert file name to ascii")
}

/// Use a path to create a unique id
pub fn path_to_id(path: &PathBuf) -> Result<Uuid> {
  Ok(Uuid::new_v5(
    &Uuid::NAMESPACE_OID,
    ["csv", "|", path_to_str(path)?]
      .iter()
      .fold("".to_string(), |mut acc, item| {
        acc.push_str(item);
        acc
      })
      .as_bytes(),
  ))
}
*/
