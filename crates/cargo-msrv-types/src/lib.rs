//! Common types shared between the [`cargo-msrv`](https://github.com/foresterre/cargo-msrv) crates

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

pub mod io_error;
pub mod toolchain;
pub mod version_match;

pub use io_error::{IoError, IoErrorSource};
pub use toolchain::Toolchain;
pub use version_match::{NoVersionMatchesManifestMsrvError, find_matching_version};
