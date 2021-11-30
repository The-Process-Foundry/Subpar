//! Row operations
//!
//! An individual row on a sheet. This is composed of cells encoded into a serde_json::Value. From
//! this value, we can convert any subset into a specific type or even send it across the wire.

use crate::local::*;

use std::{
  any::{Any, TypeId},
  collections::HashMap,
  convert::{TryFrom, TryInto},
  hash::{Hash, Hasher},
};

use serde_json::Value as JsonValue;

use schemars::schema::{
  InstanceType, Metadata, ObjectValidation, RootSchema, Schema, SchemaObject, SingleOrVec,
};

/// Some traits to attach to for a schemars RootSchema
// This is hacky, but just need to get it working as validation is not yet stored
trait SchemaTools {
  /// Retrieve the schema at the location of the dot separated path. Empty returns the root
  fn get_subschema(&self, path: &str) -> Result<&SchemaObject>;

  /// Use the schema validation at the given path to convert the value into the type found
  fn convert<T>(&self, path: &str, value: &mut JsonValue) -> Result<T>
  where
    T: serde::de::DeserializeOwned,
  {
    let schema = self.get_subschema(path)?;
    Validator::validate(schema, value)?;
    err_into!(
      serde_json::from_value(value.clone()),
      "Could not convert value '{}' at path '{}'",
      value,
      path
    )
  }
}

impl SchemaTools for RootSchema {
  fn get_subschema(&self, path: &str) -> Result<&SchemaObject> {
    let tokens = path.split('.');
    let res = tokens.fold(Ok(&self.schema), |acc, token| {
      if let Ok(current) = acc {
        let schema = current.object.as_ref().unwrap();
        match schema.properties.get(path) {
          Some(Schema::Object(next)) => Ok(next),
          Some(_) => Err(err!(
            NotFound,
            "Could not find object schema for token {} in path {}",
            token,
            path
          )),
          None => Err(err!(
            NotFound,
            "Could not find schema for token {} in path {}",
            token,
            path
          )),
        }
      } else {
        acc
      }
    })?;
    Ok(res)
  }
}

/// Attachment point for validation tools
pub struct Validator<'a> {
  value: &'a JsonValue,
  schema: &'a SchemaObject,
  itype: SingleOrVec<InstanceType>,
}

impl<'a> Validator<'a> {
  fn run(&mut self) -> Result<()> {
    let _log = format!(
      "Running Validator:\n\tschema: {:?}\n\tvalue: {:?}\n\tIType: {:?}",
      self.schema, self.value, self.itype
    );

    Ok(())
  }

  /// Checks the the value against the schema request, modifying it as necessary.
  ///
  /// This use case is for when not deserializing such as when receiving generic data.
  pub fn validate(schema: &SchemaObject, value: &mut JsonValue) -> Result<()> {
    let mut validator = Validator {
      value,
      schema,
      itype: schema
        .instance_type
        .clone()
        .unwrap_or_else(|| SingleOrVec::Single(Box::new(InstanceType::String))),
    };

    validator.run()
  }
}

/// A tag that allows us to link a struct with a row
pub trait SubparRow: TryFrom<Row> + TryInto<Row> + Any + std::fmt::Debug + Sized {
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

/// Defines the column and validation (for quick lookup)
#[derive(Clone, Debug)]
struct Column {
  _name: String,
  _validation: Box<SchemaObject>,
  required: bool,
}

/// Describe how a row is expected to look
///
/// This is used the expectation of the contents of each cell. Internally, this is a JSON schema and
/// can be built on the fly.
#[derive(Clone, Debug)]
pub struct RowTemplate {
  /// A pretty name for the template
  name: String,
  /// A lookup for each column's validation
  columns: HashMap<String, Column>,
  /// the full schema of the row
  schema: Rc<RootSchema>,
}

impl std::fmt::Display for RowTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl RowTemplate {
  pub fn new(name: String, schema: Option<RootSchema>) -> RowTemplate {
    let (columns, schema) = match schema {
      Some(schema) => {
        let mut headers = HashMap::new();
        if let Some(validation) = &schema.schema.object {
          for (key, value) in validation.properties.iter() {
            if let Schema::Object(obj) = value {
              headers.insert(
                key.clone(),
                Column {
                  _name: key.clone(),
                  _validation: Box::new(obj.clone()),
                  required: false,
                },
              );
            };
          }
          // And tag the required columns
          for name in &validation.required {
            if let Some(col) = headers.get_mut(name) {
              (*col).required = true
            }
          }
        };
        (headers, schema)
      }
      None => (HashMap::new(), RowTemplate::blank_schema(name.clone())),
    };

    RowTemplate {
      name,
      columns,
      schema: Rc::new(schema),
    }
  }

  pub fn blank_schema(name: String) -> RootSchema {
    RootSchema {
      meta_schema: Some("http://json-schema.org/draft/2019-09/schema#".to_string()),
      schema: SchemaObject {
        metadata: Some(Box::new(Metadata {
          title: Some(name.clone()),
          ..Metadata::default()
        })),
        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
        object: Some(Box::new(ObjectValidation::default())),
        ..SchemaObject::default()
      },
      ..RootSchema::default()
    }
  }

  pub fn name(&self) -> String {
    self.name.clone()
  }

  pub fn get_cell_schema(&self, name: &str) -> Result<&SchemaObject> {
    let root = match &self.schema.schema.object {
      Some(validation) => validation,
      None => {
        return Err(err!(
          NotFound,
          "Row Template {} does not have a useable schema",
          self.name
        ))
      }
    };

    let schema = (*root).properties.get(name).ok_or_else(|| {
      err!(
        NotFound,
        "No column named '{}' in row template '{}'",
        name,
        self.name
      )
    })?;

    match schema {
      Schema::Object(value) => Ok(&value),
      _ => Err(err!(
        NotFound,
        "The column named '{}' in row template '{}' did not have a proper schema configured",
        name,
        self.name
      )),
    }
  }

  pub fn get_validation(&self) -> Result<&ObjectValidation> {
    match &self.schema.schema.object {
      Some(validation) => Ok(validation),
      None => {
        return Err(err!(
          NotFound,
          "Row Template {} does not have a useable schema",
          self.name
        ))
      }
    }
  }

  /// Attempt to convert the contents of a cell to its definition held at column name
  ///
  /// This both converts and runs any validation listed in the schema, accumulating any validation
  /// errors found
  pub fn to_row(&self, cells: HashMap<String, Cell>) -> Result<Row> {
    let root = self.get_validation()?;

    // THINK: Is this best moved to schemars as a generic?
    let row = BatchResult::fold(
      Row::new(Some(self)),
      root.properties.iter(),
      |row: &mut Row, (name, obj)| {
        let cell = match cells.get(name) {
          Some(val) => val,
          None => {
            if root.required.contains(name) {
              return Err(err!(
                NotFound,
                "Row Template '{}' requires a column named '{}' but did not receive one",
                self.name,
                name
              ));
            } else {
              return Ok(());
            }
          }
        };

        let schema = match &obj {
          Schema::Object(val) => val,
          _ => {
            return Err(err!(
              NotFound,
              "Cell {} Row Template {} does not have a useable schema",
              name,
              self.name
            ))
          }
        };

        match cell.to_value(schema) {
          Ok(value) => row.add_cell(name, value),
          Err(err) => Err(err).context(format!(
            "Could not convert cell '{}' to a value for row '{}'",
            name, self.name
          )),
        }
      },
    )
    .context("Unable to convert cells to a row".to_string())
    .as_result::<SubparError>()?;
    Ok(row)
  }

  pub fn get_headers(&self) -> Result<Vec<String>> {
    match self.columns.len() {
      0 => Err(err!(
        BadValue,
        "The table template does not have any headers set"
      )),
      _ => Ok(self.columns.keys().cloned().collect()),
    }
  }

  /// Check that all the headers are found in the schema
  pub fn validate_headers(&self, headers: &Vec<String>) -> Result<()> {
    let root = self.get_validation()?;
    let mut remaining = root.required.clone();

    for header in headers {
      if root.properties.contains_key(header) {
        remaining.remove(header);
      };
    }
    if !remaining.is_empty() {
      return Err(err!(
        BadValue,
        "The following required headers were not found: {:?}",
        remaining
      ));
    }
    Ok(())
  }

  /// Add a new column definition. This should build a proper SchemaObject".to_string()
  ///
  /// This uses try_mut to attempt to clean up after any failed
  /// TODO: Move the schema maniplation this to schemars
  pub fn add_column(
    &mut self,
    name: &str,
    col_schema: Option<SchemaObject>,
    required: bool,
  ) -> Result<()> {
    log::debug!("Adding column '{}' to '{}'", name, self.name);
    log::trace!("With col_schema: {:#?}", col_schema);

    let schema = col_schema.unwrap_or(SchemaObject {
      instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
      ..SchemaObject::default()
    });

    let column = Column {
      _name: name.to_string(),
      _validation: Box::new(schema.clone()),
      required,
    };

    if self.columns.insert(name.to_string(), column).is_some() {
      return Err(err!(
        DuplicateKey,
        "Duplicate headers named '{}' in table '{}'",
        name,
        self.name
      ));
    };

    // We expect the root to be an object with the column headers as keys
    let mut validation: Box<ObjectValidation> =
      self.schema.schema.object.clone().unwrap_or_default();

    validation
      .properties
      .insert(name.to_string(), Schema::Object(schema.clone()));

    if required {
      validation.required.insert(name.to_string());
    }
    /*
    Ok(RowTemplate {
      schema: RootSchema {
        schema: SchemaObject {
          object: Some(validation),
          ..self.schema.schema
        },
        ..self.schema
      },
      ..self
    })
    */
    unimplemented!("'Add Column' still needs to be implemented")
  }

  /// Let serde deceide what it should be, if there is no hint given
  ///
  /// This is generally for a quick scan and transmission elsewhere. Additional error handling
  /// would have to be done in case serde guessed wrong.
  pub fn parse_raw(val: &str) -> Result<JsonValue> {
    err_into!(serde_json::to_value(val), "Parse raw error: {}", val)
  }
}

impl From<RootSchema> for RowTemplate {
  fn from(schema: RootSchema) -> RowTemplate {
    let name = match &schema.schema.metadata {
      Some(meta) => meta
        .title
        .clone()
        .unwrap_or_else(|| "Not Named".to_string()),
      None => "Not Named".to_string(),
    };
    RowTemplate::new(name, Some(schema))
  }
}

/// An intermediate data state
///
/// Since (de)serializing is expensive, I'm adding an intermediate storage state so we can split
/// a single row into multiple pieces.
#[derive(Debug)]
pub struct Row {
  /// Pointer to the row's template
  schema: Rc<RootSchema>,

  /// The contents of the parsed structure
  cells: JsonValue,
}

impl std::fmt::Display for Row {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Row {
  /// Create a new instance
  pub fn new(template: Option<&RowTemplate>) -> Row {
    Row {
      schema: match template {
        Some(tmp) => tmp.schema.clone(),
        None => Rc::new(RowTemplate::blank_schema("No Name".to_string())),
      },
      cells: JsonValue::Null,
    }
  }

  /// Append a cell to the end of the row
  pub fn add_cell(&mut self, name: &str, cell: serde_json::Value) -> Result<()> {
    match self.cells.as_object_mut() {
      Some(root) => {
        if let Some(old) = root.insert(name.to_string(), cell) {
          let _ = root.insert(name.to_string(), old);

          return Err(err!(
            DuplicateKey,
            "Attempted to add a second column named '{}'",
            name
          ));
        };
      }
      None => {
        let mut root = serde_json::map::Map::new();
        root.insert(name.to_string(), cell);
        self.cells = JsonValue::Object(root);
      }
    }

    Ok(())
  }

  /// Retrieve a cell from the row
  pub fn get_cell(&self, cell_name: &str) -> Result<JsonValue> {
    let cell = self.cells.get(cell_name).ok_or_else(|| {
      err!(
        NotFound,
        "Could not find cell named '{}' in row. The options are: {:#?}",
        cell_name,
        self.cells
      )
    })?;
    Ok(cell.clone())
  }

  /// Convert a generic cell into the type defined at the schema
  pub fn cell_into<T>(&self, cell_name: &str) -> Result<T>
  where
    T: serde::de::DeserializeOwned,
  {
    // println!("Converting cell into: {:?}", cell_name);
    let mut value = self.get_cell(cell_name)?;

    let root = &self.schema;
    root.convert(cell_name, &mut value)
  }

  /// Use serde to convert the cells into type T
  pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T>
  where
    T: std::fmt::Debug + serde::de::DeserializeOwned,
  {
    Ok(unwrap!(
      serde_json::from_value(self.cells.clone()),
      "Could not deserialize {:?}",
      self.cells
    ))
  }
  // /// Deserialize a row into a serde_json::Value
  // pub fn from_str(raw: &str, template: Option<RowTemplate>) -> Result<Row> {
  //   match template {
  //     None => Ok(Row {
  //       cells: serde_json::to_value(raw)?,
  //     }),
  //     Some(_str) => {
  //       unimplemented!("'from_str -> Has Template' still needs to be implemented")
  //     }
  //   }
  // }
}
