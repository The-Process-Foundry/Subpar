//! Read from a CSV file
//!
//! This wraps the csv::Reader into the common subpar model
//! TODO: Convert this to use Reader::from_reader and create a std::io::Read value

pub use crate::prelude::*;
pub use anyhow::{Context, Result};

pub use ::csv::{Error as CsvError, Reader, ReaderBuilder, StringRecord};
pub use std::path::PathBuf;

// enum Sources {
//   File(Reader<std::fs::File>),
//   Stream(Reader<>)
// }

/// Specific options used for creating the reader/writer
///
/// These are mapped directly from https://docs.rs/csv/1.1.6/csv/struct.ReaderBuilder.html
#[derive(Debug)]
pub struct Options {
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

impl Default for Options {
  fn default() -> Options {
    Options {
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

/// An open iterator pointing to the CSV file
pub struct CsvReader {
  /// The location of the file on the filesystem
  path: PathBuf,
  /// Configuration settings for the reader
  options: Options,
  /// The first line of the file, if the "has_headers" option is true. Blank if false.
  headers: Vec<String>,
  /// An iterator that will loop over the contents of CSV file, emitting Row objects
  reader: Box<dyn Iterator<Item = Result<StringRecord, CsvError>>>,
  /// A counter pointing to the last line red
  current_line: i64,
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
  pub fn new(path: &PathBuf, opts: Option<Options>) -> Result<CsvReader> {
    let (options, builder) = match opts {
      None => (Options::default(), ReaderBuilder::new()),
      Some(options) => {
        let mut builder = ReaderBuilder::new();
        if options.is_ascii {
          builder.ascii();
        };

        (options, builder)
      }
    };

    let mut reader = builder.from_path(path.as_path())?;

    let headers = match options.has_headers {
      true => reader.headers()?.iter().map(|x| x.to_owned()).collect(),
      false => vec![],
    };

    Ok(CsvReader {
      path: path.clone(),
      headers,
      options,
      reader: Box::new(reader.into_records()),
      current_line: 0,
    })
  }
}

/// Loop through the reader, returning generic rows that can be converted into specific structs
impl Iterator for CsvReader {
  type Item = Result<Row, SubparError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.current_line += 1;
    self.reader.next().map(|record| match record {
      Ok(record) => record
        .iter()
        .enumerate()
        .fold(Ok(Row::new()), |acc, (i, val)| match acc {
          Ok(row) => {
            let cell = Cell::new(serde_json::Value::String(val.to_string()));
            row.add_cell(cell, &self.headers.get(i))
          }
          err => err,
        })
        .context(format!(
          "CsvReader had an error reading row {}",
          self.current_line
        ))
        .map_err(|err| From::from(err)),

      Err(err) => Err(From::from(err)),
    })
  }
}
