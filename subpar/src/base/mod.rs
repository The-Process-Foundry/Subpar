//! Generic items used regardless of tabular data

// Data definitions of a generic workbook
pub mod workbook;

// A sheet contained in a workbook
pub mod sheet;

// A row of a single sheet
pub mod row;

// A cell in a row
pub mod cell;

// The interface of a single workbook
pub mod instance;

// The communication api
pub mod messages;
