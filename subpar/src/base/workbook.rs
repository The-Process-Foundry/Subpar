//! Define a set of related tables
//!
//! A workbook contains one or more sheets.

use crate::prelude::*;
use anyhow::{Context, Result};

use std::cell::RefCell;

/// A meta-parameter for creating a workbook
#[derive(Debug)]
pub enum BuildParams<'a> {
  // Excel,
  // Excel360,
  // GoogleSheets,
  CSV(&'a str),
  /// A premade instance that needs to be wrapped
  Built(Rc<dyn SubparWorkbook>),
}

/// A meta-workbook, wrapping the different readers/writers
#[derive(Debug)]
pub struct Workbook {
  guid: Uuid,
  /// Sheet objects organized by name
  ///
  /// These are initialized Just in Time as getting the metadata for them can be a bit heavy.
  // sheets: HashMap<String, Option<Rc<Sheet>>>,
  /// A typed workbook
  instance: Rc<dyn SubparWorkbook>,
  state: RefCell<State>,
}

impl std::fmt::Display for Workbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Workbook {
  /// Create a new workbook
  pub fn new(params: BuildParams) -> Result<Workbook, SubparError> {
    let instance = match params {
      BuildParams::CSV(path) => Rc::new(csv::CsvWorkbook::new(path)?),
      BuildParams::Built(instance) => instance,
    };

    let state = State::new(instance.get_name()?);

    // This line is needed because borrow cannot be chained
    let inst: &dyn SubparWorkbook = instance.borrow();

    let mut wb = Workbook {
      guid: inst.get_id()?,
      instance,
      state: RefCell::new(state),
    };

    wb.init()?;

    Ok(wb)
  }

  /// Initialize the workbook state based on its type and mode
  ///
  /// This validates the setting, gathers the metadata, and sets the initial cursor
  fn init(&mut self) -> Result<()> {
    let mut state = self.state.borrow_mut();
    let inst: &dyn SubparWorkbook = self.instance.borrow();

    // Get the first glance at the sheets, no deep scans
    let (_, err) = ErrorGroup::unwrap_all(
      inst
        .list_sheets()?
        .iter()
        .map(|sheet| state.add_sheet(sheet.to_owned(), None)),
    );
    err
      .map(|errors| Err(SubparError::to_group(errors)).context("Failed to add all the sheets"))
      .unwrap_or(Ok(()))?;

    Ok(())
  }

  /// Get a list of the currently known sheet names
  pub fn list_sheets(&mut self) -> Vec<String> {
    let state = self.state.borrow();
    state.list_sheets()
  }

  /// Quickly read a full sheet and return
  pub fn slurp<Row: SubparRow>(&mut self, sheet: String) -> SplitResult<Row, SubparError> {
    SplitResult::new()
  }

  /// A quick way to slurp a CSV file
  pub fn read_csv<Row: SubparRow>(path: &str) -> Result<Vec<Row>, SubparError> {
    let mut wb = Workbook::new(BuildParams::CSV(path))?;
    log::debug!("New Workbook in read_csv: {:?}", wb);

    // Check to make sure there is only one sheet
    let mut sheets = wb.list_sheets();
    let sheet_name = match sheets.len() {
      0 => Err(SubparError::NotFound)
        .context(format!("read_csv could not find any sheets at {}", path)),
      1 => Ok(sheets.pop().unwrap()),
      _ => Err(SubparError::AmbiguousResult).context(format!(
        "read_csv expects one sheet and received multiple for path {}",
        path,
      )),
    }?;

    log::debug!("Reading the CSV from sheet '{}'", sheet_name);
    let rows: SplitResult<Row, SubparError> = wb.slurp::<Row>(sheet_name);
    let result: Result<Vec<Row>, SubparError> = rows.as_result();
    Ok(result?)
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
  fn new(config: &WorkbookInstance) -> Result<Workbook, SubparError>;
  fn open(config: &WorkbookInstance) -> Result<Workbook, SubparError>;
  fn read_metadata(config: &WorkbookInstance) -> Result<WorkbookMetadata, SubparError>;
  fn read_sheet(&self, sheet_name: String) -> Result<Sheet, SubparError>;
  fn update_workbook(
    &self,
    requests: Vec<sheets_db::BatchUpdateRequestItem>,
  ) -> Result<Box<sheets_db::BatchUpdateResponse>, SubparError>;

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
