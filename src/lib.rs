#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

extern crate core;
#[macro_use]
extern crate tracing;

pub use crate::outcome::Outcome;
pub use crate::sub_command::{Find, List, Set, Show, SubCommand, Verify};

#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::{semver, Channel, FetchResources, ReleaseIndex, RustChangelog, Source};

use crate::check::RustupToolchainCheck;
use crate::config::{Action, Config, ReleaseSource};
use crate::error::{CargoMSRVError, TResult};
use crate::reporter::event::{ActionMessage, FetchIndex, Meta};
use crate::reporter::{Event, Reporter};

pub mod check;
pub mod cli;
pub mod config;
pub mod error;
pub mod exit_code;
pub mod reporter;
pub mod toolchain;

pub(crate) mod combinators;
pub(crate) mod command;
pub(crate) mod context;
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
pub(crate) mod search_method;
pub(crate) mod sub_command;
pub(crate) mod typed_bool;
pub(crate) mod writer;

pub fn run_app(config: &Config, reporter: &impl Reporter) -> TResult<()> {
    reporter.report_event(Meta::default())?;

    let action = config.action();

    info!(
        action = Into::<&'static str>::into(action),
        "running action"
    );

    reporter.report_event(ActionMessage::new(action))?;

    match action {
        Action::Find => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Find::new(&index, runner).run(config, reporter)?;
        }
        Action::Verify => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Verify::new(&index, runner).run(config, reporter)?;
        }
        Action::List => {
            List::default().run(config, reporter)?;
        }
        Action::Set => {
            Set::default().run(config, reporter)?;
        }
        Action::Show => {
            Show::default().run(config, reporter)?;
        }
    }

    Ok(())
}

fn fetch_index(config: &Config, reporter: &impl Reporter) -> TResult<ReleaseIndex> {
    reporter.run_scoped_event(FetchIndex::new(config.release_source()), || {
        let source = config.release_source();

        info!(
            source = Into::<&'static str>::into(source),
            "fetching index"
        );

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
