use crate::common::reporter::{Record, TestResultReporter};
use cargo_msrv::check::RustupToolchainCheck;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::config::test_config_from_cli;
use cargo_msrv::errors::TResult;
use cargo_msrv::{Find, SubCommand};
use rust_releases::{semver, Release, ReleaseIndex};
use std::ffi::OsString;
use std::iter::FromIterator;

pub fn find_msrv<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(
    with_args: I,
) -> TResult<Option<semver::Version>> {
    find_msrv_with_releases(with_args, releases_one_thirty_four_to_one_thirty_eight())
        .map(|res| res.msrv().map(Clone::clone))
}

pub fn run_cargo_version_which_doesnt_support_lockfile_v2<
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
>(
    with_args: I,
) -> TResult<Option<semver::Version>> {
    find_msrv_with_releases(with_args, releases_one_twenty_eight_to_one_thirty_nine())
        .map(|res| res.msrv().map(Clone::clone))
}

fn releases_one_thirty_four_to_one_thirty_eight() -> Vec<Release> {
    vec![
        Release::new_stable(semver::Version::new(1, 38, 0)),
        Release::new_stable(semver::Version::new(1, 37, 0)),
        Release::new_stable(semver::Version::new(1, 36, 0)),
        Release::new_stable(semver::Version::new(1, 35, 0)),
        Release::new_stable(semver::Version::new(1, 34, 0)),
    ]
}

fn releases_one_twenty_eight_to_one_thirty_nine() -> Vec<Release> {
    vec![
        Release::new_stable(semver::Version::new(1, 39, 0)),
        Release::new_stable(semver::Version::new(1, 38, 0)),
        Release::new_stable(semver::Version::new(1, 37, 0)),
        Release::new_stable(semver::Version::new(1, 30, 1)),
        Release::new_stable(semver::Version::new(1, 29, 2)),
        Release::new_stable(semver::Version::new(1, 28, 0)),
    ]
}

pub fn find_msrv_with_releases<
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    V: IntoIterator<Item = Release>,
>(
    with_args: I,
    included_releases: V,
) -> TResult<TestResult> {
    let matches = CargoCli::parse_args(with_args);
    let config = test_config_from_cli(&matches).expect("Unable to parse cli arguments");

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions = ReleaseIndex::from_iter(included_releases);

    let reporter = TestResultReporter::default();
    let runner = RustupToolchainCheck::new(&reporter);

    // Determine the MSRV from the index of available releases.
    let cmd = Find::new(&available_versions, runner);

    cmd.run(&config, &reporter)?;

    let mut test_result = TestResult::default();

    for item in reporter.log().iter() {
        match item {
            Record::StepComplete { version, success } if *success => {
                test_result.add_success(version.clone())
            }
            Record::StepComplete { version, success } if !(*success) => {
                test_result.add_failure(version.clone())
            }
            Record::CmdWasSuccessWithVersion(version) => test_result.set_msrv(version.clone()),
            _ => {}
        }
    }

    Ok(test_result)
}

#[derive(Default)]
pub struct TestResult {
    successful_checks: Vec<semver::Version>,
    failed_checks: Vec<semver::Version>,
    msrv: Option<semver::Version>,
}

impl TestResult {
    pub fn add_success(&mut self, version: semver::Version) {
        self.successful_checks.push(version);
    }

    pub fn add_failure(&mut self, version: semver::Version) {
        self.successful_checks.push(version);
    }

    pub fn set_msrv(&mut self, version: semver::Version) {
        self.msrv = Some(version);
    }

    pub fn successful_checks(&self) -> &[semver::Version] {
        &self.successful_checks
    }

    pub fn failed_checks(&self) -> &[semver::Version] {
        &self.failed_checks
    }

    pub fn msrv(&self) -> Option<&semver::Version> {
        self.msrv.as_ref()
    }
}
