/*
for future use when processing each specific radar information
*/

pub mod error;
pub mod radar;

// re-export usage of Error and Results from this module
pub use crate::model::error::{Error, Result};
