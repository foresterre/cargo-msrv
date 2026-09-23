//! The rustup command runner of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]

#[macro_use]
extern crate tracing;

mod rustup_command;

pub use rustup_command::{RustupCommand, RustupOutput};
