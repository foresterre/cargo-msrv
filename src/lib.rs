#![deny(clippy::all)]
#![allow(clippy::upper_case_acronyms, clippy::unnecessary_wraps)]

use rust_releases::{semver, Channel, FetchResources, RustChangelog, RustDist, Source};

use crate::config::{Config, ModeIntent, ReleaseSource};
use crate::errors::{CargoMSRVError, TResult};
use crate::reporter::{Output, ProgressAction};

pub use crate::{
    result::MinimalCompatibility, subcommands::determine_msrv::determine_msrv,
    subcommands::determine_msrv::run_determine_msrv_action,
    subcommands::verify_msrv::run_verify_msrv_action,
};

pub mod check;
pub mod cli;
pub(crate) mod command;
pub mod config;
pub mod errors;
pub(crate) mod fetch;
pub(crate) mod lockfile;
pub(crate) mod manifest;
pub(crate) mod packages;
pub(crate) mod paths;
pub mod reporter;
pub(crate) mod result;
pub(crate) mod subcommands;

pub fn run_app<R: Output>(config: &Config, reporter: &R) -> TResult<()> {
    reporter.progress(ProgressAction::FetchingIndex);

    let index = match config.release_source() {
        ReleaseSource::RustChangelog => {
            RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
        }
        ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
    };

    match config.action_intent() {
        ModeIntent::DetermineMSRV => run_determine_msrv_action(config, reporter, &index),
        ModeIntent::VerifyMSRV => run_verify_msrv_action(config, reporter, &index),
        ModeIntent::List => todo!(),
    }
}
