//! Row operations
//!
//! An individual row on a sheet. This is composed of wrapped/encoded cells. This is the level
//! we place the top level serialization

use crate::prelude::*;
use anyhow::Result;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A tag that allows us to link a struct with a row
pub trait SubparRow: Any + std::fmt::Debug + Sized {
  /// Get a unique id for the template type
  ///
  /// This defaults to automatically create a Uuid by hashing the any::TypeId
  fn get_id() -> Uuid {
    let id = TypeId::of::<Self>();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);

    Uuid::new_v5(&Uuid::NAMESPACE_OID, &hasher.finish().to_be_bytes())
  }

  /// Get the mapping
  fn get_template() -> SheetTemplate {
    unimplemented!("'get_template' is not implemented yet")
  }

  /// to_cells
  fn to_cells(&self) -> Result<Row, SubparError> {
    unimplemented!("'to_cells' is not implemented yet<")
  }

  /// from_cells
  fn from_row(row: Row) -> Result<Self, SubparError>;
}

#[derive(Debug)]
pub struct Row {
  /// A list of cells, in the order they appear
  cells: Vec<Rc<Cell>>,

  /// A lookup of cells
  by_name: HashMap<String, Rc<Cell>>,
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
      cells: vec![],
      by_name: HashMap::new(),
    }
  }

  /// Append a cell to the end of the row
  pub fn add_cell(&mut self, value: Cell, name: Option<&str>) -> Result<()> {
    let cell = Rc::new(value);
    if let Some(name) = name {
      if let Some(_) = self.by_name.insert(name.to_string(), cell.clone()) {
        Err(SubparError::DuplicateKey)
          .context(format!("Attempted to add a cell named '{}' twice", name))?
      }
    }
    self.cells.push(cell);
    Ok(())
  }

  /// Retrieve a cell from the row
  pub fn get_cell(&self, cell_name: &str) -> Result<Cell> {
    let cell = ok_or!(
      { self.by_name.get(cell_name) },
      SubparError::NotFound,
      "Could not find cell named '{}' in row",
      cell_name
    )?;
    Ok(Cell::clone(&cell))
  }
}
