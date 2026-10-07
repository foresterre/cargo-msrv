//! Common types shared between the [`cargo-msrv`](https://github.com/foresterre/cargo-msrv) crates

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

pub mod io_error;
pub mod toolchain;

pub use io_error::{IoError, IoErrorSource};
pub use toolchain::Toolchain;
