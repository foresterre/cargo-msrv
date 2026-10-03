//! The cargo and rustup tooling of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

#[macro_use]
extern crate tracing;

mod cargo_command;
mod cargo_manifest;
mod dependency_graph;
mod rustup_command;
mod writer;

pub use cargo_command::CargoCommand;
pub use cargo_manifest::{CargoManifest, CargoManifestParser, ManifestParseError, TomlParser};
pub use dependency_graph::{
    DependencyGraph,
    resolver::{CargoMetadataResolveError, CargoMetadataResolver, DependencyResolver},
};
pub use rustup_command::{RustupCommand, RustupOutput};
pub use writer::{
    manifest_msrv::{WriteManifestMsrvError, write_manifest_msrv, write_workspace_msrv},
    toolchain_file::write_toolchain_file,
};
