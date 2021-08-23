//! An individual cell in a table
//!
//! This is a wrapped value, designed to hold the data in intermediate form

// use crate::prelude::*;
// use anyhow::Result;

/// A tag that allows us to link a struct with a row
pub trait SubparCell: std::fmt::Debug {}

/// This is the base of the serialization. The common types are listed here.
#[derive(Debug)]
pub enum Cell {
  Null,
  Empty,
  Number(f64),
  String(String),
  Bool(bool),
  Struct(String),
  List(Vec<Cell>),
}

impl SubparCell for Cell {}
