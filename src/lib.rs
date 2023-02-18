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

pub use crate::outcome::Outcome;
pub use crate::sub_command::{Find, List, Set, Show, SubCommand, Verify};

#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::{semver, Channel, FetchResources, ReleaseIndex, RustChangelog, Source};

use crate::check::RustupToolchainCheck;
use crate::config::{Config, ReleaseSource, SubcommandId};
use crate::error::{CargoMSRVError, TResult};
use crate::reporter::event::{FetchIndex, Meta, SubcommandInit};
use crate::reporter::{Event, Reporter};

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
pub(crate) mod manifest;
pub(crate) mod msrv;
pub(crate) mod outcome;
pub(crate) mod search_method;
pub(crate) mod sub_command;
pub(crate) mod typed_bool;
pub(crate) mod writer;

pub fn run_app(config: &Config, reporter: &impl Reporter) -> TResult<()> {
    reporter.report_event(Meta::default())?;

    let subcommand_id = config.subcommand_id();

    reporter.report_event(SubcommandInit::new(subcommand_id))?;

    match subcommand_id {
        SubcommandId::Find => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Find::new(&index, runner).run(config, reporter)?;
        }
        SubcommandId::Verify => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Verify::new(&index, runner).run(config, reporter)?;
        }
        SubcommandId::List => {
            List::default().run(config, reporter)?;
        }
        SubcommandId::Set => {
            let index = fetch_index(config, reporter).ok();
            Set::new(index.as_ref()).run(config, reporter)?;
        }
        SubcommandId::Show => {
            Show::default().run(config, reporter)?;
        }
    }

    Ok(())
}

fn fetch_index(config: &Config, reporter: &impl Reporter) -> TResult<ReleaseIndex> {
    reporter.run_scoped_event(FetchIndex::new(config.release_source()), || {
        let index = match config.release_source() {
            ReleaseSource::RustChangelog => {
                RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
            }
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
        };

        Ok(index)
    })
}
