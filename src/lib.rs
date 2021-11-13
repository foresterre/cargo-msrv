#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

#[macro_use]
extern crate tracing;

use rust_releases::{
    semver, Channel, FetchResources, ReleaseIndex, RustChangelog, RustDist, Source,
};

use crate::config::{Config, ModeIntent, ReleaseSource};
use crate::errors::{CargoMSRVError, TResult};
use crate::reporter::{Output, ProgressAction};

use crate::subcommands::list::run_list_msrv;
use crate::subcommands::show::run_show_msrv;
pub use crate::{
    result::MinimalCompatibility, subcommands::determine_msrv::determine_msrv,
    subcommands::determine_msrv::run_determine_msrv_action,
    subcommands::verify_msrv::run_verify_msrv_action,
};

pub mod check;
pub mod cli;
pub(crate) mod command;
pub mod config;
pub(crate) mod dependencies;
pub mod errors;
pub(crate) mod fetch;
pub(crate) mod lockfile;
pub(crate) mod manifest;
pub(crate) mod paths;
pub mod reporter;
pub(crate) mod result;
pub(crate) mod subcommands;

pub fn run_app<R: Output>(config: &Config, reporter: &R) -> TResult<()> {
    reporter.progress(ProgressAction::FetchingIndex);

    let index = fetch_index(config)?;
    run_action(config, &index, reporter)
}

fn fetch_index(config: &Config) -> TResult<ReleaseIndex> {
    let source = config.release_source();

    info!(
        source = Into::<&'static str>::into(source),
        "fetching index"
    );

    let index = match config.release_source() {
        ReleaseSource::RustChangelog => {
            RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
        }
        ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
    };

    Ok(index)
}

fn run_action<R: Output>(config: &Config, index: &ReleaseIndex, reporter: &R) -> TResult<()> {
    let action = config.action_intent();

    info!(
        action = Into::<&'static str>::into(action),
        "running action"
    );

    match action {
        ModeIntent::DetermineMSRV => run_determine_msrv_action(config, reporter, index),
        ModeIntent::VerifyMSRV => run_verify_msrv_action(config, reporter, index),
        ModeIntent::List => run_list_msrv(config, reporter),
        ModeIntent::Show => run_show_msrv(config, reporter),
    }
}
