//! Some common macros I use
//!
//! These need to move off somewhere else. See each individual one for where I think it should live

pub mod use_all_macros {
  pub use crate::{err_ctx, ok_or};
}

/// Turn an Option<x> into anyhow::Result<x> with with context
///
/// # Examples
///
/// ```
/// # use subpar::ok_or;
/// # use anyhow::{anyhow, Context, Result};
///
/// # fn test_error(result: Result<&str>, as_str: &str) {
/// #   match &result {
/// #     Err(err) => {
/// #        let value = format!("{:?}", err);
/// #        println!("{}", value);
/// #        assert_eq!(value, as_str.to_string());
/// #        println!("{:#?}", result)
/// #      },
/// #     Ok(_) => panic!("There was no error: {:#?}", result)
/// #   }
/// # }
///
/// let result: Result<&str> = ok_or!(
///   Some("Test1"),
///   anyhow!("No Error")
/// );
/// # assert_eq!(result.unwrap(), "Test1");
///
/// // A simple error
/// let error_str = "Simple Error";
/// let result: Result<&str> = ok_or!(
///   None,
///   anyhow!(error_str)
/// );
/// # test_error(result, error_str);
///
///
/// // An error with additional context
/// let error_str = "The Error";
/// let result: Result<&str> = ok_or!(
///   None,
///   anyhow!(error_str),
///   "A context block with extra info - Ctx1: {}, Ctx2: {}",
///   "Value1",
///   "Value2"
/// );
/// # test_error(result, "A context block with extra info - Ctx1: Value1, Ctx2: Value2\n\nCaused by:\n    The Error");
/// ```
#[macro_export]
macro_rules! ok_or {
  ($code:expr, $err:expr) => {
    $code.ok_or($err)
  };
  ($code:expr, $err:expr, $($ctx:expr),+) => {
    $code.ok_or($err).context(format!($($ctx, )*))
  };
}

/// Make an anyhow::Error instance with a string context
///
/// # Examples
///
///
/// ```rust
/// # use subpar::err_ctx;
/// use anyhow::{anyhow, Result, Context};
///
/// // An error with a simple string context
/// let error: Result<()> = err_ctx!(
///   anyhow!("The Error"),
///   "This is a context"
/// );
///
/// // Adding formatting for the string
/// let error: Result<()> = err_ctx!(
///   anyhow!("The Error"),
///   "Format: Term1: {}, Term2: {}",
///   "Value1",
///   "Value2"
/// );
/// ```
#[macro_export]
macro_rules! err_ctx {
  ($err:expr, $($terms:expr),+) => {
    Err($err).context(format!($($terms, )*))
  };
}
