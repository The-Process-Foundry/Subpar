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

// The Subpar implementation of allwhat
pub mod errors;

// Items common to all Table Types
pub mod base;

// (De)serialization row
pub mod json;

// CSV table parsers
#[cfg(feature = "csv_tables")]
pub mod csv;

// Some generic code I use throughout
pub(crate) mod helpers;

// Handle multiple csv files simultaneously
// pub mod server;

// Cartographic wrapper
// #[cfg(feature = "cartography")]
// pub mod cartograph;

// Some macros I use everywhere. To be moved to other projects
pub mod macros;

mod local {
  pub use anyhow::{anyhow, Error as AnyhowError};

  // Make all the types safe to use
  pub use std::borrow::{Borrow, BorrowMut};
  pub use std::cell::RefCell;
  pub use std::rc::Rc;

  // Everything should be identified uniquely
  pub use uuid::Uuid;

  // The standard serializer derive macros
  pub use serde::{Deserialize, Serialize};

  // Everything should be using a converting to a SubparError
  pub(crate) use crate::helpers;
  pub use crate::prelude::*;

  pub use allwhat::prelude::{BatchResult, ErrorGroup, Grouper, SplitResult};
}

/// All the basic items needed to use subpar derive
pub mod prelude {
  // Used by SubparRow trait, so it shows up everywhere
  pub use std::convert::{TryFrom, TryInto};

  pub use crate::{
    base::{
      self,
      accessor::Accessor,
      cell::{Cell, CellValue},
      //   instance::{Mode, SubparWorkbook},
      //   messages::{Action, Event},
      //   sheet::{Sheet, SheetAccessor, SheetTemplate, SubparSheet},
      //   workbook::Workbook,
      row::{Row, RowTemplate, SubparRow},
    },
    errors::{self, prelude::*},
  };

  #[cfg(feature = "csv_tables")]
  pub use crate::csv::{self, io::CsvReader};

  // pub(crate) use base::state::State;

  // #[cfg(feature = "cartography")]
  // pub use {crate::cartograph, cartograph::ServerPoI};
}
