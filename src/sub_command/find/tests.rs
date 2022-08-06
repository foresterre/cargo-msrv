use super::*;
use crate::check::TestRunner;
use crate::config::ConfigBuilder;
use crate::manifest::bare_version::BareVersion;
use crate::reporter::TestReporter;
use crate::{Event, SubcommandId};
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

    let config = Config::new(SubcommandId::Find, "".to_string());
    let reporter = TestReporter::default();
    let runner = TestRunner::with_ok(&[semver::Version::new(1, 56, 0)]);

    let cmd = Find::new(&index, runner);
    let found = cmd.run(&config, reporter.reporter()).unwrap();
    assert_eq!(found, semver::Version::new(1, 56, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::new_msrv(
        semver::Version::new(1, 56, 0),
        &config,
        BareVersion::ThreeComponents(1, 37, 0),
        BareVersion::ThreeComponents(1, 56, 0),
    )
    .into()];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
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

    let config = Config::new(SubcommandId::Find, "".to_string());
    let reporter = TestReporter::default();
    let runner = TestRunner::with_ok(&[
        semver::Version::new(1, 56, 0),
        semver::Version::new(1, 55, 0),
        semver::Version::new(1, 54, 0),
        semver::Version::new(1, 53, 0),
        semver::Version::new(1, 52, 0),
    ]);

    let cmd = Find::new(&index, runner);
    let found = cmd.run(&config, reporter.reporter()).unwrap();
    assert_eq!(found, semver::Version::new(1, 52, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::new_msrv(
        semver::Version::new(1, 52, 0),
        &config,
        BareVersion::ThreeComponents(1, 52, 0),
        BareVersion::ThreeComponents(1, 56, 0),
    )
    .into()];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
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

    let config = Config::new(SubcommandId::Find, "".to_string());
    let reporter = TestReporter::default();
    let runner = TestRunner::with_ok(&[]);

    let cmd = Find::new(&index, runner);
    let result = cmd.run(&config, reporter.reporter());
    assert!(result.is_err());

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::none(
        &config,
        BareVersion::ThreeComponents(1, 52, 0),
        BareVersion::ThreeComponents(1, 56, 0),
    )
    .into()];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
}

// https://github.com/foresterre/cargo-msrv/issues/369
#[test]
fn no_releases_available() {
    let releases = vec![
        Release::new_stable(semver::Version::new(1, 46, 0)),
        Release::new_stable(semver::Version::new(1, 55, 0)),
        Release::new_stable(semver::Version::new(1, 56, 0)),
        Release::new_stable(semver::Version::new(1, 57, 0)),
        Release::new_stable(semver::Version::new(1, 58, 1)),
        Release::new_stable(semver::Version::new(1, 59, 0)),
    ];

    let index = ReleaseIndex::from_iter(releases);

    let min = BareVersion::TwoComponents(1, 56);
    let max = BareVersion::ThreeComponents(1, 54, 0);

    // Make sure we end up with an empty releases set
    let config = ConfigBuilder::new(SubcommandId::Find, "")
        .minimum_version(min.clone()) // i.e. Rust edition = 2021
        .maximum_version(max.clone())
        .build();

    let reporter = TestReporter::default();
    let runner = TestRunner::with_ok(&[]);

    let cmd = Find::new(&index, runner);
    let result = cmd.run(&config, reporter.reporter());
    let err = result.unwrap_err();
    assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(_)));

    if let CargoMSRVError::NoToolchainsToTry(inner_err) = err {
        assert_eq!(inner_err.min.as_ref(), Some(&min));
        assert_eq!(inner_err.max.as_ref(), Some(&max));
        assert_eq!(&inner_err.search_space, &[]);
    }

    let events = reporter.wait_for_events();

    let unexpected_event: Event = FindResult::none(
        &config,
        BareVersion::TwoComponents(1, 56),
        BareVersion::ThreeComponents(1, 54, 0),
    )
    .into();

    assert!(!events.contains(&unexpected_event));
}
