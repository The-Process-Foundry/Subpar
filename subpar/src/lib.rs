//! A tabular data manager
//!
//!

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

pub mod prelude {
  // Everything should be identified uniquely
  pub use uuid::Uuid;

  // The standard serializer derive macros
  pub use serde::{Deserialize, Serialize};

  // Simple alias so we don't have to use crate::
  pub use crate::{base, csv, errors, server};

  // Everything should be converting to a SubparError
  pub use errors::SubparError;

  pub use base::{
    cell::{Cell, SubparCell},
    instance::{Mode, State},
    messages::{Action, Event},
    row::{Row, SubparRow},
    sheet::{Sheet, SubparSheet},
    workbook::{SubparWorkbook, Workbook, WorkbookInstance},
  };

  #[cfg(feature = "cartography")]
  pub use {crate::cartograph, cartograph::ServerPoI};
}
