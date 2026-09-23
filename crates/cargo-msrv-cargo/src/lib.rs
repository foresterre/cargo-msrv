//! The cargo command builder of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]

mod cargo_command;

pub use cargo_command::CargoCommand;
