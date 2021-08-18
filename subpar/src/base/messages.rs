//! Action and event message deffinitions
//!
//! Messages are an indirect method of communicating between components. This allows subpar to be
//! operated remotely via API.

// use anyhow::Result;

use serde::{Deserialize, Serialize};
// use uuid::Uuid;

/// Commands a workbook knows how to accept
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Action {
  //---- Bulk actions
  Read,

  /// Write a workbook into the specified format
  Write,

  //---- Workbook Actions
  /// Open a Workbook
  Open,

  /// Retrieve the metadata of the workbook
  /// TODO: Break this down into different gets (eg. GetSheetMetadata)
  GetMetadata,

  /// Change the active sheet in the workbook
  SetSheet,

  /// Read the next row of the active worksheet
  GetNextRow,

  /// Append a row to an open sheet
  WriteRow,

  /// Append a group of rows to the current sheet
  WriteBulk,

  /// Close an open reader/writer.
  Close,
}

/// Messages that can be emitted/returned from a workbook
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Event {
  /// Found and opened a new workbook
  WorkbookOpened,

  /// Processed the workbook for reading/writing
  ReadMetadata,

  /// Set the current sheet of a workbook
  SheetChange,

  /// Closed the file handle accessing the workbook. Metadata may be cached,
  ///
  /// This may occur from circumstances outside the server, so this can be used to trigger a
  /// re-open, if desired
  ClosedWorkbook,

  /// Read one or more rows from a sheet
  ReadRowData,

  /// Read an entire sheet of data
  ReadSheet,

  /// Wrote one or more rows to a sheet
  WroteData,
}
