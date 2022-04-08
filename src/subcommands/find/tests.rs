use super::*;
use crate::testing::{Record, TestResultReporter, TestRunner};
use rust_releases::semver;
use std::iter::FromIterator;

#[test]
fn bisect_find_only_last() {
    let index = ReleaseIndex::from_iter(vec![
        Release::new_stable(semver::Version::new(1, 56, 0)),
        Release::new_stable(semver::Version::new(1, 55, 0)),
        Release::new_stable(semver::Version::new(1, 54, 0)),
        Release::new_stable(semver::Version::new(1, 53, 0)),
        Release::new_stable(semver::Version::new(1, 52, 0)),
        Release::new_stable(semver::Version::new(1, 51, 0)),
        Release::new_stable(semver::Version::new(1, 50, 0)),
        Release::new_stable(semver::Version::new(1, 49, 0)),
        Release::new_stable(semver::Version::new(1, 48, 0)),
        Release::new_stable(semver::Version::new(1, 47, 0)),
        Release::new_stable(semver::Version::new(1, 46, 0)),
        Release::new_stable(semver::Version::new(1, 45, 0)),
        Release::new_stable(semver::Version::new(1, 44, 0)),
        Release::new_stable(semver::Version::new(1, 43, 0)),
        Release::new_stable(semver::Version::new(1, 42, 0)),
        Release::new_stable(semver::Version::new(1, 41, 0)),
        Release::new_stable(semver::Version::new(1, 40, 0)),
        Release::new_stable(semver::Version::new(1, 39, 0)),
        Release::new_stable(semver::Version::new(1, 38, 0)),
        Release::new_stable(semver::Version::new(1, 37, 0)),
    ]);

    let config = Config::new(ModeIntent::Find, "".to_string());
    let reporter = TestResultReporter::default();

    let runner = TestRunner::with_ok(&[semver::Version::new(1, 56, 0)]);

    let cmd = Find::new(&index, runner);
    let _ = cmd.run(&config, &reporter);

    let log = reporter.log();

    assert_eq!(
        log.as_slice(),
        &[
            Record::CheckToolchain(semver::Version::new(1, 47, 0)),
            Record::CheckToolchain(semver::Version::new(1, 52, 0)),
            Record::CheckToolchain(semver::Version::new(1, 54, 0)),
            Record::CheckToolchain(semver::Version::new(1, 55, 0)),
            Record::CheckToolchain(semver::Version::new(1, 56, 0)),
            Record::CmdWasSuccess,
        ]
    );
}

#[test]
fn bisect_find_all_compatible() {
    let index = ReleaseIndex::from_iter(vec![
        Release::new_stable(semver::Version::new(1, 56, 0)),
        Release::new_stable(semver::Version::new(1, 55, 0)),
        Release::new_stable(semver::Version::new(1, 54, 0)),
        Release::new_stable(semver::Version::new(1, 53, 0)),
        Release::new_stable(semver::Version::new(1, 52, 0)),
    ]);

    let config = Config::new(ModeIntent::Find, "".to_string());
    let reporter = TestResultReporter::default();

    let runner = TestRunner::with_ok(&[
        semver::Version::new(1, 56, 0),
        semver::Version::new(1, 55, 0),
        semver::Version::new(1, 54, 0),
        semver::Version::new(1, 53, 0),
        semver::Version::new(1, 52, 0),
    ]);

    let cmd = Find::new(&index, runner);
    let _ = cmd.run(&config, &reporter);

    let log = reporter.log();

    assert_eq!(
        log.as_slice(),
        &[
            Record::CheckToolchain(semver::Version::new(1, 54, 0)),
            Record::CheckToolchain(semver::Version::new(1, 53, 0)),
            Record::CheckToolchain(semver::Version::new(1, 52, 0)),
            Record::CmdWasSuccess,
        ]
    );
}

#[test]
fn bisect_none_compatible() {
    let index = ReleaseIndex::from_iter(vec![
        Release::new_stable(semver::Version::new(1, 56, 0)),
        Release::new_stable(semver::Version::new(1, 55, 0)),
        Release::new_stable(semver::Version::new(1, 54, 0)),
        Release::new_stable(semver::Version::new(1, 53, 0)),
        Release::new_stable(semver::Version::new(1, 52, 0)),
    ]);

    let config = Config::new(ModeIntent::Find, "".to_string());
    let reporter = TestResultReporter::default();

    let runner = TestRunner::with_ok(&[]);

    let cmd = Find::new(&index, runner);
    let _ = cmd.run(&config, &reporter);

    let log = reporter.log();

    assert_eq!(
        log.as_slice(),
        &[
            Record::CheckToolchain(semver::Version::new(1, 54, 0)),
            Record::CheckToolchain(semver::Version::new(1, 55, 0)),
            Record::CheckToolchain(semver::Version::new(1, 56, 0)),
            Record::CmdWasFailure,
        ]
    );
}
