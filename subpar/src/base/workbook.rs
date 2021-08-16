//! Define a set of related tables
//!
//! A workbook contains one or more sheets.

use anyhow::Result;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base::instance::*;
use crate::csv;

/// A common accessor for a set of tabular data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkbookInstance {
  // Excel(excel::Config),
  // Excel360(excel360::Config),
  // GoogleSheets(sheets::Config),
  CSV(csv::Instance),
}

impl WorkbookInstance {}

#[derive(Clone, Debug)]
pub struct Workbook {
  guid: Uuid,
  /// Table Definition and options
  instance: WorkbookInstance,
  state: State,
}

impl std::fmt::Display for Workbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Workbook {
  pub fn open_csv(path: &str, mode: Mode) -> Result<Workbook> {
    log::debug!("Opening A CSV at {:?}", path);
    let location = csv::Location::new(path)?;
    let inst = csv::Instance::new(location, mode)?;
    Ok(Workbook {
      guid: inst.get_id()?,
      state: inst.get_state()?,
      instance: WorkbookInstance::CSV(inst),
    })
  }

  /// Initialize the workbook state based on its type and mode
  ///
  /// This validates the setting, gathers the metadata, and sets the initial cursor
  pub fn init(&mut self) -> Result<Workbook> {
    match &self.instance {
      WorkbookInstance::CSV(inst) => {
        self.state = inst.get_state()?;
        Ok(self.clone())
      }
      _ => unimplemented!("'Workbook::init' only implemented for CSV"),
    }
  }
}

/*


/// Define the workbook we want to
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct WorkbookInstance {}

impl std::fmt::Display for WorkbookInstance {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl WorkbookInstance {}

/// A tabular workbook API
///
/// All worksheets are
pub trait MetaWorkbook {
  fn new(config: &WorkbookInstance) -> Result<Workbook>;
  fn open(config: &WorkbookInstance) -> Result<Workbook>;
  fn read_metadata(config: &WorkbookInstance) -> Result<WorkbookMetadata>;
  fn read_sheet(&self, sheet_name: String) -> Result<Sheet>;
  fn update_workbook(
    &self,
    requests: Vec<sheets_db::BatchUpdateRequestItem>,
  ) -> Result<Box<sheets_db::BatchUpdateResponse>>;

  // write_sheet(&self)
  // fn insert_row(&self, sheet_name: String, row_number: i32)
  // fn update_row(&self, sheet_name: String, row_data)
  // fn update_cell(&self, sheet_name: String, column_name: String, value)
}

/// The data type of the cell. Everything is kept in strings since I expect it to be parsed later
/// using the from_cell/to_cell macros
///
/// This is borrowed from Google's Protocol buffer, since I'm doing the most work with that
/// https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Value
#[derive(Clone, Debug)]
pub enum CellType {
  Null,
  Number(f64),
  String(String),
  Bool(bool),
  Struct(String),
  List(Vec<CellType>),
}

#[derive(Clone, Debug)]
pub struct Cell {
  pub location: (usize, usize),
  pub data: CellType,
}

/// The full data set included in the sheet
#[derive(Clone, Debug)]
pub struct Sheet {
  pub header_map: std::collections::HashMap<String, usize>,
  pub header_vec: Vec<String>,
  pub data: Vec<Vec<Cell>>,
}

*/
