//! A tabular data manager
//!
//!

// Features needed to enable the ? handling of ErrorGroups
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(never_type)]
#![feature(box_into_inner)]

#[doc(hidden)]
pub use subpar_derive::SubparTable;

// The Subpar wrapped errors
pub mod errors;

// Items common to all Table Types
pub mod base;

// CSV table parsers
pub mod csv;

// Handle multiple csv files simultaneously
pub mod server;

// Cartographic wrapper
#[cfg(feature = "cartography")]
pub mod cartograph;

// Some macros I use everywhere. To be moved to other projects
pub mod macros;

pub mod prelude {
  // Make all the types safe to use
  pub use std::borrow::{Borrow, BorrowMut};
  pub use std::cell::RefCell;
  pub use std::rc::Rc;

  // Used by SubparRow trait
  pub use std::convert::TryFrom;

  // Everything should be identified uniquely
  pub use uuid::Uuid;

  // The standard serializer derive macros
  pub use serde::{Deserialize, Serialize};

  // Simple alias so we don't have to use crate::
  pub use crate::{base, errors, server};

  pub use crate::macros::use_all_macros::*;

  // Everything should be converting to a SubparError
  pub use errors::{ErrorGroup, SplitResult, SubparError};

  pub use base::{
    cell::Cell,
    instance::{Mode, SubparWorkbook},
    messages::{Action, Event},
    row::{Row, SubparRow},
    sheet::{Sheet, SheetTemplate, SubparSheet},
    workbook::Workbook,
  };

  pub(crate) use base::state::State;

  #[cfg(feature = "cartography")]
  pub use {crate::cartograph, cartograph::ServerPoI};

  // CSV Feature
  pub use crate::csv;
  pub use crate::csv::{instance::CsvWorkbook, io::reader::Reader};

  pub use crate::csv::*;
}
