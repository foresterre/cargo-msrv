use crate::common::reporter::EventTestDevice;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::compatibility::RustupToolchainCheck;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::reporter::{Message, SubcommandResult};
use cargo_msrv::{Context, Find, SubCommand};
use rust_releases::{semver, Release, ReleaseIndex};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::iter::FromIterator;

pub fn find_msrv<I: IntoIterator<Item = T>, T: Into<OsString> + Clone>(
    with_args: I,
) -> Result<Option<semver::Version>, CargoMSRVError> {
    find_msrv_with_releases(with_args, releases_one_thirty_four_to_one_thirty_eight())
        .map(|res| res.msrv().cloned())
}

pub fn run_cargo_version_which_doesnt_support_lockfile_v2<
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
>(
    with_args: I,
) -> Result<Option<semver::Version>, CargoMSRVError> {
    find_msrv_with_releases(with_args, releases_one_twenty_eight_to_one_thirty_nine())
        .map(|res| res.msrv().cloned())
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
) -> Result<TestResult, CargoMSRVError> {
    let matches = CargoCli::parse_args(with_args);
    let opts = matches.to_cargo_msrv_cli().to_opts();
    let ctx = Context::try_from(opts)?;
    let find_ctx = ctx.to_find_context();

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions = ReleaseIndex::from_iter(included_releases);

    let device = EventTestDevice::default();

    let ignore_toolchain = find_ctx.ignore_lockfile;
    let no_check_feedback = find_ctx.no_check_feedback;
    let env = &find_ctx.environment;

    let runner = RustupToolchainCheck::new(
        device.reporter(),
        ignore_toolchain,
        no_check_feedback,
        env,
        find_ctx.run_command(),
    );

    // Determine the MSRV from the index of available releases.
    let cmd = Find::new(&available_versions, runner);

    cmd.run(&find_ctx, device.reporter())?;

    let events = device.wait_for_events();
    let mut test_result = TestResult::default();

    for item in events {
        match item.message() {
            Message::CheckResult(res) if res.is_compatible() => {
                test_result.add_success(res.toolchain().version().clone());
            }
            Message::CheckResult(res) if !res.is_compatible() => {
                test_result.add_failure(res.toolchain().version().clone());
            }
            Message::SubcommandResult(SubcommandResult::Find(res)) => {
                test_result.set_msrv(res.msrv().cloned())
            }
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

    pub fn set_msrv(&mut self, version: Option<semver::Version>) {
        self.msrv = version;
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
