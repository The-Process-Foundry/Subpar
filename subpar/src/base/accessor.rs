//! Parameters needed to build an IO connection to a specific sheet
//!
//! These need to be customized for each type of tabular data, as CSV keeps its sheets in different
//! files while Google Sheets uses URL keys.

use crate::local::*;
use anyhow::Context;
use std::path::PathBuf;

pub enum Accessor {
  Csv(PathBuf),
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
      Accessor::Csv(path) => Ok(Accessor::Csv(canonicalize(path)?)),
    }
  }
}

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
