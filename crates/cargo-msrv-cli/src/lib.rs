//! The command line interface of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv).
//!
//! This crate defines the `clap` front-end: the options as they are presented to the user.
//! It intentionally does not resolve these options into a runnable configuration; that is the
//! job of the `context` module of the `cargo-msrv` crate, which consumes the opts defined here.

#![deny(clippy::all)]
#![allow(clippy::uninlined_format_args)]

pub mod cli;
pub mod types;
