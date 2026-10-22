//! Make Cargo metadata work with older versions of Cargo.

#![deny(clippy::all)]

mod error;
mod selection;

pub use error::Error;
pub use selection::Selection;
