#![allow(unused)] // allowed since we do use these functions in the actual test files

use cargo_msrv::config::{test_config_from_matches, Config, OutputFormat};
use cargo_msrv::errors::TResult;
use cargo_msrv::reporter::{Reporter, ReporterBuilder};
use cargo_msrv::MinimalCompatibility;
use rust_releases::{semver, Release, ReleaseIndex};
use std::ffi::OsString;
use std::iter::FromIterator;

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
        cargo_msrv::determine_msrv,
    )
    .unwrap()
}

pub fn run_verify<I, T, S>(with_args: I, releases: S) -> TResult<()>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = Release>,
{
    run(with_args, releases, cargo_msrv::run_verify_msrv_action)
}

fn run<T, I, S, F, R>(with_args: I, releases: S, action: F) -> TResult<R>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = Release>,
    F: FnOnce(&Config, &Reporter, &ReleaseIndex) -> TResult<R>,
{
    let matches = cargo_msrv::cli::cli().get_matches_from(with_args);
    let config = test_config_from_matches(&matches).expect("Unable to parse cli arguments");

    let reporter = fake_reporter();

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions: ReleaseIndex = FromIterator::from_iter(releases);

    // Determine the MSRV from the index of available releases.
    action(&config, &reporter, &available_versions)
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

pub fn fake_reporter() -> Reporter<'static> {
    ReporterBuilder::new("", "")
        .output_format(OutputFormat::None)
        .build()
}