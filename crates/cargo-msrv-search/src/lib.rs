//! The MSRV search methods and toolchain compatibility check of
//! [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args, clippy::items_after_test_module)]

#[macro_use]
extern crate tracing;

pub mod compatibility;
pub mod error;
pub mod lockfile;
pub mod msrv;
pub mod outcome;
pub mod search_method;

pub(crate) mod setup_toolchain;

pub use error::{Error, TResult};
pub use outcome::Compatibility;
