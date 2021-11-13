#![allow(unused)] // allowed since we do use these functions in the actual test files

use std::ffi::OsString;
use std::iter::FromIterator;

use rust_releases::semver::Version;
use rust_releases::{semver, Release, ReleaseIndex};

use cargo_msrv::config::{test_config_from_matches, Config, OutputFormat};
use cargo_msrv::errors::TResult;
use cargo_msrv::reporter::__private::SuccessOutput;
use cargo_msrv::reporter::json::JsonPrinter;
use cargo_msrv::reporter::no_output::NoOutput;
use cargo_msrv::reporter::ui::HumanPrinter;
use cargo_msrv::reporter::Output;
use cargo_msrv::{reporter, MinimalCompatibility};

pub fn run_msrv<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(
    with_args: I,
) -> MinimalCompatibility {
    run(
        with_args,
        vec![
            Release::new_stable(semver::Version::new(1, 38, 0)),
            Release::new_stable(semver::Version::new(1, 37, 0)),
            Release::new_stable(semver::Version::new(1, 36, 0)),
            Release::new_stable(semver::Version::new(1, 35, 0)),
            Release::new_stable(semver::Version::new(1, 34, 0)),
        ],
        &fake_reporter(),
        cargo_msrv::determine_msrv,
    )
    .unwrap()
}

pub fn run_msrv_with_releases<I, T, S>(
    with_args: I,
    releases: S,
) -> (MinimalCompatibility, SuccessOutput)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    S: IntoIterator<Item = Release>,
{
    let reporter = test_reporter();
    let compatibility = run(with_args, releases, &reporter, cargo_msrv::determine_msrv).unwrap();
    (compatibility, reporter)
}

pub fn run_verify<I, T, S>(with_args: I, releases: S) -> TResult<()>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = Release>,
{
    run(
        with_args,
        releases,
        &fake_reporter(),
        cargo_msrv::run_verify_msrv_action,
    )
}

fn run<T, I, S, F, R, Reporter>(
    with_args: I,
    releases: S,
    reporter: &Reporter,
    action: F,
) -> TResult<R>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = Release>,
    Reporter: Output,
    F: FnOnce(&Config, &Reporter, &ReleaseIndex) -> TResult<R>,
{
    let matches = cargo_msrv::cli::cli().get_matches_from(with_args);
    let config = test_config_from_matches(&matches).expect("Unable to parse cli arguments");

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions: ReleaseIndex = FromIterator::from_iter(releases);

    // Determine the MSRV from the index of available releases.
    action(&config, reporter, &available_versions)
}

pub fn run_cargo_version_which_doesnt_support_lockfile_v2<
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
>(
    with_args: I,
) -> MinimalCompatibility {
    let matches = cargo_msrv::cli::cli().get_matches_from(with_args);
    let matches = test_config_from_matches(&matches).expect("Unable to parse cli arguments");

    let reporter = fake_reporter();

    // Limit the available versions: this ensures we don't want to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions: ReleaseIndex = FromIterator::from_iter(vec![
        Release::new_stable(semver::Version::new(1, 39, 0)),
        Release::new_stable(semver::Version::new(1, 38, 0)),
        Release::new_stable(semver::Version::new(1, 37, 0)),
        Release::new_stable(semver::Version::new(1, 30, 1)),
        Release::new_stable(semver::Version::new(1, 29, 2)),
        Release::new_stable(semver::Version::new(1, 28, 0)),
    ]);

    // Determine the MSRV from the index of available releases.
    cargo_msrv::determine_msrv(&matches, &reporter, &available_versions)
        .expect("Unable to run MSRV process")
}

pub fn fake_reporter() -> reporter::no_output::NoOutput {
    reporter::no_output::NoOutput
}

pub fn test_reporter() -> reporter::__private::SuccessOutput {
    reporter::__private::SuccessOutput::default()
}
