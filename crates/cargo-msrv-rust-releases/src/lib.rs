//! The index of available Rust releases of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

#[macro_use]
extern crate tracing;

pub mod availability_filter;
pub mod error;
pub mod release;
pub mod release_index;
pub mod releases_filter;

pub use availability_filter::{
    AvailabilityFilter, AvailabilityOutcome, ExcludedRelease, ToolchainAvailability,
    ToolchainUnavailable,
};
pub use error::FetchIndexError;
pub use release::to_semver;
pub use release_index::{ReleaseIndex, fetch_index};
pub use releases_filter::ReleasesFilter;

pub use rust_releases::core::rust_release::toolchain as release_toolchain;
pub use rust_releases::core::{RustRelease, Stable, StableReleases};
