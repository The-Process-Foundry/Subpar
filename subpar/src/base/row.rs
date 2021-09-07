//! Row operations
//!
//! An individual row on a sheet. This is composed of cells encoded into a serde_json::Value. From
//! this value, we can convert any subset into a specific type or even send it across the wire.

use crate::local::*;
use anyhow::{Context, Result};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};

pub use schemars::schema::{RootSchema, Schema, SchemaObject, SubschemaValidation};
pub use serde_json::Value as JsonValue;

/// A tag that allows us to link a struct with a row
pub trait SubparRow:
  TryFrom<Row, Error = AnyhowError>
  + TryInto<Row, Error = AnyhowError>
  + Any
  + std::fmt::Debug
  + Sized
{
  /// Get a unique id for the template type
  ///
  /// This defaults to automatically create a Uuid by hashing the any::TypeId
  fn get_id() -> Uuid {
    let id = TypeId::of::<Self>();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);

    Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_be_bytes())
  }

  /// Get the expected headers, mapping and order of a row
  fn get_template() -> RowTemplate {
    unimplemented!("'get_template' is not implemented yet")
  }
}

/// Describe how a row is expected to look
///
/// This is used the expectation of the contents of each cell. Internally, this is a JSON schema and
/// can be built on the fly.
#[derive(Debug)]
pub struct RowTemplate(RootSchema);

impl std::fmt::Display for RowTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl RowTemplate {
  pub fn new(schema: Option<RootSchema>) -> RowTemplate {
    RowTemplate(schema.unwrap_or(RootSchema {
      meta_schema: None,
      schema: SchemaObject {
        metadata: None,
        instance_type: None,
        format: None,
        enum_values: None,
        const_value: None,
        subschemas: None,
        number: None,
        string: None,
        array: None,
        object: None,
        reference: None,
        extensions: schemars::Map::new(),
      },
      definitions: schemars::Map::new(),
    }))
  }

  /// Attempt to convert the contents of a cell to its definition held at column name
  ///
  /// This both converts and runs any validation listed in the schema, accumulating any validation
  /// errors found
  pub fn to_row(&self, cells: HashMap<String, Cell>) -> Result<Row> {
    // let params: (header, Cell, SchemaObject) =
    // ;
    // let row = AnnotatedResult::fold(
    //   Row::new(),
    //   params,
    //   |acc, (cell: Cell, schema: SchemaObject)| {
    //     match cell.to_value(schema) {
    //       Ok(value) => acc.add_cell(value)
    //     }
    //   }
    // )

    unimplemented!("'to_row' still needs to be implemented")
    // Deserialize

    // Ok(serde_json::to_value(value)?)
  }

  pub fn get_headers(&self) -> Result<Vec<String>> {
    match self.0.definitions.len() {
      0 => Err(Kind::BadValue).context("Headers were not set in the schema"),
      _ => Ok(self.0.definitions.keys().map(|x| x.clone()).collect()),
    }
  }

  /// Check that all the headers are found in the schema
  pub fn validate_headers(&self, headers: &Vec<String>) -> Result<()> {
    unimplemented!("'validate_headers' still needs to be implemented")
  }

  /// Add a new column definition. This should build a proper SchemaObject
  pub fn add_column(&mut self, _name: String, position: Option<u16>) -> Result<RowTemplate> {
    unimplemented!("'RowTemplate::add_column' still needs to be implemented")
  }

  /// Let serde deceide what it should be, if there is no hint given
  ///
  /// This is generally for a quick scan and transmission elsewhere. Additional error handling
  /// would have to be done in case serde guessed wrong.
  pub fn parse_raw(val: &str) -> Result<JsonValue> {
    Ok(serde_json::to_value(val)?)
  }
}

impl From<RootSchema> for RowTemplate {
  fn from(schema: RootSchema) -> RowTemplate {
    RowTemplate(schema)
  }
}

/// An intermediate data state
///
/// Since (de)serializing is expensive, I'm adding an intermediate storage state so we can split
/// a single row into multiple pieces.
#[derive(Debug)]
pub struct Row {
  // /// Pointer to the row's template
  // template: Option<Rc<RefCell<RowTemplate>>>
  /// The contents of the parsed structure
  cells: serde_json::Value,
}

impl std::fmt::Display for Row {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Row {
  /// Create a new instance
  pub fn new() -> Row {
    Row {
      cells: serde_json::to_value("{}").unwrap(),
    }
  }

  /// Append a cell to the end of the row
  pub fn add_cell(self, _cell: serde_json::Value, _name: String) -> Result<Self> {
    // let mut root = self
    //   .cells
    //   .as_object_mut()
    //   .ok_or({
    //     Err(SubparError::Impossible).ctx(format!(
    //       "A row root was not an object, it was {:#?}",
    //       self.cells
    //     ))
    //   })
    //   .map_err(|err| {
    //     log::error!("{:#?}", err);
    //     err
    //   })?;
    // if let Some(_) = root.insert(name.clone(), cell) {
    //   Err(SubparError::DuplicateKey).ctx(format!(
    //     "Attempted to add a cell named '{}' twice to a row",
    //     name
    //   ))?
    // };
    Ok(self)
  }

  /// Retrieve a cell from the row
  pub fn get_cell(&self, cell_name: &str) -> Result<serde_json::Value> {
    let cell = self
      .cells
      .get(cell_name)
      .ok_or(Kind::NotFound)
      .context(format!("Could not find cell named '{}' in row", cell_name))?;
    Ok(cell.clone())
  }

  /// Deserialize a row into a serde_json::Value
  pub fn from_str(raw: &str, template: Option<RowTemplate>) -> Result<Row> {
    match template {
      None => Ok(Row {
        cells: serde_json::to_value(raw)?,
      }),
      Some(_str) => {
        unimplemented!("'from_str -> Has Template' still needs to be implemented")
      }
    }
  }
}
