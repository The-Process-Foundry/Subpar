//! Define a set of related tables
//!
//! A workbook contains one or more sheets.

use crate::local::*;
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::RwLock;

/// Mutably borrow the state or sheet
macro_rules! borrow {
  ($item:expr) => {
    $item.read().or(Err(SubparError::RwLockError))?
  };
  ($var:ident, $item:expr) => {
    let $var = borrow!($item);
  };
  ("r", $var:ident, $item:expr) => {
    borrow!($var, $item)
  };
  ("w", $var:ident, $item:expr) => {
    let mut $var = $item.write().or(Err(SubparError::RwLockError))?;
  };
}

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

  /// Sheet objects organized by a pretty name
  sheets: HashMap<String, Rc<Sheet>>,

  /// A typed workbook configuration
  instance: Rc<dyn SubparWorkbook>,

  /// Management info of the included sheets and facilitator of intra-workbook communication
  state: RwLock<State>,
}

impl std::fmt::Display for Workbook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl Workbook {
  /// Create a new workbook
  pub fn new(params: BuildParams) -> Result<Workbook> {
    let instance = match params {
      BuildParams::CSV(path) => Rc::new(csv::CsvWorkbook::new(path)?),
      BuildParams::Built(instance) => instance,
    };

    let state = State::new(instance.get_name()?);

    let mut wb = Workbook {
      guid: instance.get_id()?,
      instance: instance.to_owned(),
      sheets: HashMap::new(),
      state: RwLock::new(state),
    };

    wb.init()?;

    Ok(wb)
  }

  /// Initialize the workbook state based on its type and mode
  ///
  /// This validates the setting, gathers the metadata, and sets the initial cursor
  fn init(&mut self) -> Result<()> {
    borrow!("w", state, self.state);
    let inst: &dyn SubparWorkbook = self.instance.borrow();

    // Get the first glance at the sheets, no deep scans
    let (_, err) = ErrorGroup::unwrap_all(
      inst
        .list_sheets()?
        .iter()
        .map(|sheet| state.add_sheet(sheet.to_owned(), None)),
    );
    err
      .map(|errors| Err(SubparError::to_group(errors)).ctx("Failed to add all the sheets"))
      .unwrap_or(Ok(()))?;

    Ok(())
  }

  /// Get a list of the currently known sheet names
  pub fn list_sheets(&mut self) -> Result<Vec<String>> {
    Ok(borrow!(self.state).list_sheets())
  }

  // /// Get a file modifier reader/writer for the given sheet
  // pub fn open(&self, sheet_name: &String, mode: Mode) -> Result<()> {
  //   borrow!("w", state, self.state);
  //   Ok(())
  // }

  /// Quickly read a full sheet and return
  ///
  /// THINK: Should this return rows or the actual split result? I lean toward the latter as I
  /// usually want to do some post-processing after reading, even if some errors are acceptable
  pub fn slurp<Row: SubparRow>(&mut self, sheet_name: &String) -> Result<SplitResult<Row>> {
    log::debug!("Starting to slurp {}", sheet_name);

    borrow!("w", state, self.state);
    state.open(sheet_name, Mode::Read)?;

    // Map the row type to the sheet name
    // state.add_template::<Row>(sheet_name, vec![Mode::Read])?;

    // Open the sheet in read mode
    let accessor = self.instance.get_sheet_accessor(sheet_name)?;
    let reader = CsvReader::slurp(accessor, None)?;

    Ok(SplitResult::map(reader, |line| {
      // log::debug!("Processing Row: {:#?}", line);
      match line {
        Ok(row) => {
          let row: Result<Row> = TryFrom::try_from(row);
          // log::debug!("Converted to: {:#?}", row);
          row
        }
        Err(err) => Err(err),
      }
    }))
  }

  /// Write a list of rows to a sheet, replacing the existing data
  ///
  /// This is for quick and dirty writing tables with default options
  pub fn dump<Row: SubparRow>(&mut self, _sheet_name: String, _data: Vec<Row>) -> Result<()> {
    unimplemented!("'Workbook::dump' is not implemented yet")
  }

  /// A simple way read a CSV file
  pub fn read_csv<Row: SubparRow>(path: &str) -> Result<Vec<Row>> {
    let mut wb = Workbook::new(BuildParams::CSV(path))?;
    log::debug!("New Workbook in read_csv: {:?}", wb);

    // Check to make sure there is only one sheet
    let mut sheets = wb.list_sheets()?;
    let sheet_name = match sheets.len() {
      0 => {
        Err(SubparError::NotFound).ctx(format!("read_csv could not find any sheets at {}", path))
      }
      1 => Ok(sheets.pop().unwrap()),
      _ => Err(SubparError::AmbiguousResult).ctx(format!(
        "read_csv expects one sheet and received multiple for path {}",
        path,
      )),
    }?;

    log::debug!("Reading the CSV from sheet '{}'", sheet_name);
    let rows: SplitResult<Row> = wb.slurp::<Row>(&sheet_name)?;
    // log::debug!("Finished reading the rows: {:#?}", rows);
    let result: Result<Vec<Row>> = rows.as_result();
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
