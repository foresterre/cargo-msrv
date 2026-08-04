//! The command line interface of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv).
//!
//! This crate defines the `clap` front-end: the options as they are presented to the user,
//! and their resolution into the context defined by the
//! [`cargo-msrv-context`](cargo_msrv_context) crate.

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

pub mod cli;
pub mod context;
pub mod values;
