#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

#[macro_use]
extern crate tracing;
extern crate core;

use crate::check::RustupToolchainCheck;
#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::{semver, Channel, FetchResources, ReleaseIndex, RustChangelog, Source};

use crate::config::{Config, ModeIntent, OutputFormat, ReleaseSource};
use crate::errors::{CargoMSRVError, TResult};
use crate::reporter::{Output, ProgressAction};

pub use crate::outcome::Outcome;
pub use crate::subcommands::{Find, List, Set, Show, SubCommand, Verify};

pub mod check;
pub mod cli;
pub mod config;
pub mod errors;
pub mod exit_code;
pub mod reporter;
pub mod toolchain;

pub(crate) mod command;
pub(crate) mod ctx;
pub(crate) mod dependencies;
pub(crate) mod download;
pub(crate) mod fetch;
pub(crate) mod formatter;
pub(crate) mod lockfile;
pub(crate) mod log_level;
pub(crate) mod manifest;
pub(crate) mod outcome;
pub(crate) mod paths;
pub(crate) mod releases;
pub(crate) mod result;
pub(crate) mod search_methods;
pub(crate) mod storyteller;
pub(crate) mod subcommands;
pub(crate) mod writers;

#[cfg(test)]
pub(crate) mod testing;

pub fn run_app<R: Output>(config: &Config, reporter: &R) -> TResult<()> {
    let action = config.action_intent();

    info!(
        action = Into::<&'static str>::into(action),
        "running action"
    );

    let result = match action {
        ModeIntent::Find => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Find::new(&index, runner).run(config, reporter)
        }
        ModeIntent::Verify => {
            let index = fetch_index(config, reporter)?;
            let runner = RustupToolchainCheck::new(reporter);
            Verify::new(&index, runner).run(config, reporter)
        }
        ModeIntent::List => List::default().run(config, reporter),
        ModeIntent::Set => Set::default().run(config, reporter),
        ModeIntent::Show => Show::default().run(config, reporter),
    };

    if let Err(ref err) = result {
        if let OutputFormat::Human = config.output_format() {
            // Can't use reporter here because the ProgressBar in HumanReporter is already set to
            // finished. Adding a line on top, will redraw the bar, instead of updating it, producing
            // two bars with the text in between.
            eprintln!("{}", err);
        }

        // FIXME: re-enable reporting errors in json, but first format them as json!
    }

    result
}

fn fetch_index<R: Output>(config: &Config, reporter: &R) -> TResult<ReleaseIndex> {
    reporter.progress(ProgressAction::FetchingIndex);

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
}
