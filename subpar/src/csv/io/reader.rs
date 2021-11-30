//! Read from a CSV file
//!
//! This wraps the csv::Reader into the common subpar model
//! TODO: Convert this to use Reader::from_reader and create a std::io::Read value

pub use crate::local::*;

pub use ::csv::{Error as CsvError, Reader, ReaderBuilder, StringRecord};
pub use std::collections::HashMap;
pub use std::path::PathBuf;

/// Specific options used for creating the reader/writer
///
/// The are mapped directly from https://docs.rs/csv/1.1.6/csv/struct.ReaderBuilder.html
#[derive(Debug)]
pub struct FileOptions {
  /// Use Ascii delimited text
  pub is_ascii: bool,
  /// Set the reader's buffer capacity. Rust decides by default.
  pub buffer_size: Option<usize>,
  /// Ignore lines that begin with this character array. Defaults to None
  pub comment: Option<u8>,
  /// The cell separator byte. Defaults to ','
  pub delimiter: u8,
  /// Enable double quote escapes. Default is true
  pub double_quotes: bool,
  /// Change the escape character from the default '\'
  pub escape: u8,
  /// Whether the number of fields per line can change. Default is false
  pub flexible: bool,
  /// If the first line should be headers. Default is true
  pub has_headers: bool,
  /// The quoting character. Default is '""
  pub quote: u8,
  /// Items between quote characters do not need escaping (except the quote). Default is true
  pub quoting: bool,
  /// The end of line byte array. The default of None matches any of [`\r`, `\n`, `\r\n`]
  pub terminator: Option<u8>,
  /// Removes leading/trailing whitespace (['\t', '\n', '\v', '\f', '\r', ' ']) from each cell.
  /// Default is turned on
  pub trim: bool,
}

impl Default for FileOptions {
  fn default() -> FileOptions {
    FileOptions {
      is_ascii: false,
      buffer_size: None,
      comment: None,
      delimiter: b',',
      double_quotes: true,
      escape: b'\\',
      flexible: false,
      has_headers: true,
      quote: b'"',
      quoting: true,
      terminator: None,
      trim: true,
    }
  }
}

#[derive(Debug, Default)]
pub struct Options {
  /// reader specific options
  pub file_options: FileOptions,

  // Validation options
  /// Add unknown columns to the row without validation. If false, they are just ignored.
  pub keep_unknown: bool,
}

/// An open iterator pointing a data stream which returns rows of data
///
/// This can use a json schema as a template to validate items as they go along, as well as coerce
/// ambiguous items into their
pub struct CsvReader {
  /// The location of the file on the filesystem
  path: PathBuf,

  /// Configuration settings for the reader
  options: Options,

  /// The first line of the file if it has a header row, otherwise a stringified list of column numbers.
  headers: Vec<String>,

  /// An iterator that will loop over the contents of CSV file, emitting Row objects
  reader: Box<dyn Iterator<Item = Result<StringRecord, CsvError>>>,

  /// A counter pointing to the last line red
  current_line: i64,

  /// Define the expected fields
  template: Rc<RowTemplate>,
}

impl std::fmt::Debug for CsvReader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CsvReader")
      .field("path", &self.path)
      .field("headers", &self.headers)
      .field("options", &self.options)
      .field("current_line", &self.current_line)
      .finish()
  }
}

impl std::fmt::Display for CsvReader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl CsvReader {
  /// Create a new reader
  pub fn new(
    accessor: Accessor,
    template: Option<Rc<RowTemplate>>,
    opts: Option<Options>,
  ) -> Result<CsvReader> {
    // Set up the base reader
    let (options, builder) = match opts {
      None => (Options::default(), ReaderBuilder::new()),
      Some(options) => {
        let mut builder = ReaderBuilder::new();
        if options.file_options.is_ascii {
          builder.ascii();
        };

        (options, builder)
      }
    };

    // Get the canonicalized path of the location
    let canon = &accessor.canonicalize(false)?;
    let Accessor::Csv(path) = canon;
    // This type check will be needed later
    // _ => Err(Kind::Impossible).context("Tried to build a CSV Reader with an invalid accessor"),

    let mut reader = err_into!(builder.from_path(path.as_path()))?;

    // Get or create a schema for the file
    let (template, headers) = match options.file_options.has_headers {
      true => {
        let headers = err_into!(reader.headers())?
          .iter()
          .map(|x| x.to_owned())
          .collect();
        match template {
          Some(schema) => {
            schema.validate_headers(&headers).context(format!(
              "Could not validate the headers for {}",
              schema.name()
            ))?;
            (schema, headers)
          }
          None => {
            let schema = BatchResult::fold(
              RowTemplate::new(canon.name(), None),
              headers.iter(),
              |acc: &mut RowTemplate, item| acc.add_column(item, None, false),
            )
            .as_result::<SubparError>()?;
            (Rc::new(schema), headers)
          }
        }
      }
      false => match template {
        Some(schema) => {
          let headers = schema.get_headers()?;
          (schema, headers)
        }
        None => {
          return Err(err!(
            NotImplemented,
            "Cannot read CSV file '{:?}' because it doesn't have either headers or a template",
            path.to_str()
          ))
        }
      },
    };

    Ok(CsvReader {
      path: path.clone(),
      headers,
      options,
      reader: Box::new(reader.into_records()),
      current_line: 0,
      template,
    })
  }

  pub fn slurp<T: SubparRow>(path: &str, _opts: Option<Options>) -> Result<Vec<T>>
  where
    T: TryFrom<Row, Error = SubparError>,
  {
    let accessor = Accessor::new_csv(path);
    let reader = CsvReader::new(accessor, Some(Rc::new(T::get_template())), None)?;

    SplitResult::map(reader, |line| {
      // log::debug!("Processing Row: {:#?}", line);
      match line {
        Ok(row) => {
          let row: Result<T> = TryFrom::try_from(row);
          // log::debug!("Converted to: {:#?}", row);
          row
        }
        Err(err) => Err(err),
      }
    })
    .context(format!("Failed to slurp CSV file at '{}'", path))
    .as_result()
  }
}

/// Loop through the reader, returning generic rows that can be converted into specific structs
impl Iterator for CsvReader {
  type Item = Result<Row>;

  fn next(&mut self) -> Option<Self::Item> {
    self.current_line += 1;
    // log::debug!("Trying to read data line {}", self.current_line);
    let record = match self.reader.next() {
      Some(Ok(rec)) => rec,
      None => return None,
      Some(Err(err)) => {
        return Some(err_into!(
          Err(err),
          "Error reading record {} from file {}",
          self.current_line,
          self.path.to_string_lossy()
        ));
      }
    };

    // Create a hashmap of cells to be processed
    let cells =
      self
        .headers
        .iter()
        .enumerate()
        .fold(HashMap::<String, Cell>::new(), |mut acc, (i, name)| {
          let val = match record.get(i).unwrap() {
            "" => CellValue::Empty,
            x => CellValue::Raw(x.to_string()),
          };
          acc.insert(name.clone(), Cell::new(name.clone(), val));
          acc
        });

    Some(self.template.to_row(cells).context(format!(
      "Could not convert record {} from file {} into a row",
      self.current_line,
      self.path.to_string_lossy(),
    )))
  }
}
