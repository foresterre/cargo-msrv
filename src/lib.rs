//! Documentation can be found on the project [README](https://github.com/foresterre/cargo-msrv/blob/main/README.md) page
//! and in the cargo-msrv [book](https://foresterre.github.io/cargo-msrv/). If you can't find an answer on your question,
//! feel free to open an [issue](https://github.com/foresterre/cargo-msrv/issues/new).
//!
//! Issues and ideas may be reported via the [issue tracker](https://github.com/foresterre/cargo-msrv/issues),
//! and questions can be asked on the [discussion forum](https://github.com/foresterre/cargo-msrv/discussions).
//!
//! The docs focus on how to use `cargo-msrv` from the command line. If you want to also use it as a library,
//! please feel free to open an [issue](https://github.com/foresterre/cargo-msrv/issues/new).

#![deny(clippy::all)]
#![allow(
    clippy::upper_case_acronyms,
    clippy::unnecessary_wraps,
    clippy::uninlined_format_args
)]

extern crate core;
#[macro_use]
extern crate tracing;

pub use crate::outcome::Outcome;
pub use crate::sub_command::{Find, List, Set, Show, SubCommand, Verify};

use crate::check::{Check, RustupToolchainCheck};
use crate::command::RustupCommand;
use crate::config::{Config, ReleaseSource, SubcommandId};
use crate::error::{CargoMSRVError, TResult};
use crate::reporter::event::{Meta, SubcommandInit};
use crate::reporter::{Event, Reporter};
use crate::toolchain::ToolchainSpec;
use rust_releases::semver;

pub mod check;
pub mod cli;
pub mod config;
pub mod error;
pub mod exit_code;
pub mod io;
pub mod reporter;
pub mod toolchain;

pub(crate) mod combinators;
pub(crate) mod command;
pub(crate) mod ctx;
pub(crate) mod default_target;
pub(crate) mod dependency_graph;
pub(crate) mod download;
pub(crate) mod filter_releases;
pub(crate) mod formatting;
pub(crate) mod lockfile;
pub(crate) mod log_level;
pub(crate) mod manifest;
pub(crate) mod msrv;
pub(crate) mod outcome;
mod release_index;
pub(crate) mod search_method;
pub(crate) mod sub_command;
pub(crate) mod typed_bool;
pub(crate) mod writer;

pub fn run_app(config: &Config, reporter: &impl Reporter) -> TResult<()> {
    reporter.report_event(Meta::default())?;

    let subcommand_id = config.subcommand_id();

    info!(
        subcommand_id = Into::<&'static str>::into(subcommand_id),
        "running subcommand"
    );

    reporter.report_event(SubcommandInit::new(subcommand_id))?;

    match subcommand_id {
        SubcommandId::Find => {
            let index = release_index::fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Find::new(&index, runner).run(config, reporter)?;
        }
        SubcommandId::Verify => {
            let index = release_index::fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Verify::new(&index, runner).run(config, reporter)?;
        }
        SubcommandId::List => {
            List::default().run(config, reporter)?;
        }
        SubcommandId::Set => {
            let index = release_index::fetch_index(config, reporter).ok();
            Set::new(index.as_ref()).run(config, reporter)?;
        }
        SubcommandId::Show => {
            Show::default().run(config, reporter)?;
        }
        SubcommandId::Forward(fwd) => {
            // Read the MSRV
            let msrv = todo!();

            // Run rustup run <toolchain> cargo (build | check | test) ...args

            let run_forwarded_subcommand = todo!();
        }
    }

    Ok(())
}

fn run(config: &Config, reporter: &impl Reporter) -> TResult<()> {
    let rustup_output = RustupCommand::new()
        .with_args(cmd.iter())
        .with_optional_dir(dir)
        .with_stderr()
        .run()
        .map_err(|_| CargoMSRVError::UnableToRunCheck)?;

    let status = rustup_output.exit_status();
}
