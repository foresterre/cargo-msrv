//! Common types shared between the [`cargo-msrv`](https://github.com/foresterre/cargo-msrv) crates

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

pub mod bare_version;

pub use bare_version::{BareVersion, NoVersionMatchesManifestMsrvError};
