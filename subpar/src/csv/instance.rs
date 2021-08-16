//! Definition and options for CSV files

use crate::prelude::*;
use anyhow::{Context, Result};
use std::boxed::Box;
use std::path::{Path, PathBuf};

use crate::base::instance::*;

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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Location {
  path: Box<PathBuf>,
}

impl Location {
  /// Wrap the file/directory where the csv file(s) belong
  pub fn new(path: &str) -> Result<Location> {
    // Canonical is better for creating a UUID
    let abs_path = Path::new(&path).canonicalize().context(format!(
      "Could not canonicalize path '{}'. Current directory is: {:?}",
      path,
      std::env::current_dir()?
    ))?;
    Ok(Location {
      path: Box::new(abs_path),
    })
  }

  pub fn to_str(&self) -> Result<&str> {
    match self.path.as_path().to_str() {
      Some(path_str) => Ok(path_str),
      None => Err(SubparError::ConversionError).context(format!(
        "Path '{:#?}' could not be converted to a string",
        self.path
      ))?,
    }
  }
}

/// A thin wrapper around the csv module, shaping it to be used by Subpar
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Instance {
  /// The location of the CSV file
  location: Location,
  /// Read or Write
  mode: Mode,
  /// CSV Specific options
  options: Options,
}

impl std::fmt::Display for Instance {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Instance {
  //---  Option setters/getters
  pub fn set_delimeter(&mut self, delimeter: u8) -> Result<Instance> {
    self.options.delimiter = delimeter;
    Ok(self.clone())
  }
}

impl WbInstance for Instance {
  type LocationType = Location;

  /// Builds a new instance and validates the location
  fn new(location: Location, mode: Mode) -> Result<Instance> {
    Ok(Instance {
      location,
      mode,
      options: Options::new(),
    })
  }

  fn get_id(&self) -> Result<Uuid> {
    Ok(Uuid::new_v5(
      &Uuid::NAMESPACE_OID,
      ["csv", "|", self.location.to_str()?]
        .iter()
        .fold("".to_string(), |mut acc, item| {
          acc.push_str(item);
          acc
        })
        .as_bytes(),
    ))
  }

  fn get_state(&self) -> Result<State> {
    Ok(State::new())
  }
}
