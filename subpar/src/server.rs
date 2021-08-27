//! A hub that can manage multiple open workbooks
//!
//! The server gives a kind of cursor/pointer to open items. It uses a single entry point that
//! accepts Actions and emits Events as a result. Direct management of individual workbooks is
//! possible and recommended if it is a one off.
//! This is used as a Point of Interest for Cartogenic, which allows the server to run in another
//! process or even as a stand-alone micro-service

use crate::prelude::*;
use anyhow::{Context, Result};

use std::collections::HashMap;

// ----------------------------- Action Parameters  -----------------------------

// ----------------------------- Event Parameters  -----------------------------

#[derive(Debug)]
pub struct SubparServer {
  config: (),
  handles: HashMap<Uuid, Workbook>,
}

impl std::fmt::Display for SubparServer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl SubparServer {
  pub fn new() -> SubparServer {
    SubparServer {
      config: (),
      handles: HashMap::new(),
    }
  }

  pub fn run(&self, _act: Action) -> Result<()> {
    err_ctx!(
      SubparError::NotImplemented,
      "SubparServer::run is not implemented"
    )
  }
}

/* Add the cartograpic information

impl
*/
