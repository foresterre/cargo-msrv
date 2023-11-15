use super::*;
use crate::check::TestRunner;
use crate::context::{
    CheckCommandContext, EnvironmentContext, ReleaseSource, RustReleasesContext, ToolchainContext,
    UserOutputContext,
};
use crate::manifest::bare_version::BareVersion;
use crate::reporter::TestReporterWrapper;
use crate::{Event, OutputFormat};
use camino::Utf8PathBuf;
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

    let reporter = TestReporterWrapper::default();
    let runner = TestRunner::with_ok("x", &[semver::Version::new(1, 56, 0)]);

    let cmd = Find::new(&index, runner);
    let mut context = create_test_context();
    context.search_method = SearchMethod::Bisect;

    let found = cmd.run(&context, reporter.get()).unwrap();
    assert_eq!(found, semver::Version::new(1, 56, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::new_msrv(
        semver::Version::new(1, 56, 0),
        "x",
        BareVersion::ThreeComponents(1, 37, 0),
        BareVersion::ThreeComponents(1, 56, 0),
        SearchMethod::Bisect,
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

    let reporter = TestReporterWrapper::default();
    let runner = TestRunner::with_ok(
        "x",
        &[
            semver::Version::new(1, 56, 0),
            semver::Version::new(1, 55, 0),
            semver::Version::new(1, 54, 0),
            semver::Version::new(1, 53, 0),
            semver::Version::new(1, 52, 0),
        ],
    );

    let cmd = Find::new(&index, runner);
    let mut ctx = create_test_context();
    ctx.search_method = SearchMethod::Bisect;

    let found = cmd.run(&ctx, reporter.get()).unwrap();
    assert_eq!(found, semver::Version::new(1, 52, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::new_msrv(
        semver::Version::new(1, 52, 0),
        "x",
        BareVersion::ThreeComponents(1, 52, 0),
        BareVersion::ThreeComponents(1, 56, 0),
        SearchMethod::Bisect,
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

    let reporter = TestReporterWrapper::default();
    let runner = TestRunner::with_ok("x", &[]);

    let cmd = Find::new(&index, runner);
    let mut ctx = create_test_context();
    ctx.search_method = SearchMethod::Bisect;

    let result = cmd.run(&ctx, reporter.get());
    assert!(result.is_err());

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![FindResult::none(
        "x",
        BareVersion::ThreeComponents(1, 52, 0),
        BareVersion::ThreeComponents(1, 56, 0),
        SearchMethod::Bisect,
    )
    .into()];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
}

// These test cases cover the case that the minimum is set to be a strictly more recent
// Rust release compared to the maximum set.
// https://github.com/foresterre/cargo-msrv/issues/369
#[cfg(test)]
mod issue_369_min_more_recent_than_max {
    use super::*;

    #[test]
    fn bisect() {
        let releases = vec![
            Release::new_stable(semver::Version::new(1, 46, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 57, 0)),
            Release::new_stable(semver::Version::new(1, 58, 1)),
            Release::new_stable(semver::Version::new(1, 59, 0)),
        ];

        let index = ReleaseIndex::from_iter(releases);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok("x", &[]);

        let cmd = Find::new(&index, runner);
        let mut ctx = create_test_context();

        // Create a negative search space, by setting min > max, effectively emptying it.
        ctx.rust_releases.minimum_rust_version = Some(BareVersion::TwoComponents(1, 56));
        ctx.rust_releases.maximum_rust_version = Some(BareVersion::ThreeComponents(1, 54, 0));
        ctx.search_method = SearchMethod::Bisect;

        let result = cmd.run(&ctx, reporter.get());
        let err = result.unwrap_err();

        assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(ref inner) if inner.has_clues()));

        let message = format!("{}", err);
        assert_eq!("No Rust releases to check: the filtered search space is empty. Search space limited by user to min Rust '1.56', and max Rust '1.54.0'", message);

        let events = reporter.wait_for_events();

        let unexpected_event: Event = FindResult::none(
            "x",
            BareVersion::TwoComponents(1, 56),
            BareVersion::ThreeComponents(1, 54, 0),
            SearchMethod::Bisect,
        )
        .into();

        assert!(!events.contains(&unexpected_event));
    }

    #[test]
    fn linear() {
        let releases = vec![
            Release::new_stable(semver::Version::new(1, 46, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 57, 0)),
            Release::new_stable(semver::Version::new(1, 58, 1)),
            Release::new_stable(semver::Version::new(1, 59, 0)),
        ];

        let index = ReleaseIndex::from_iter(releases);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok("x", &[]);

        let cmd = Find::new(&index, runner);
        let mut ctx = create_test_context();

        // Create a negative search space, by setting min > max, effectively emptying it.
        ctx.rust_releases.minimum_rust_version = Some(BareVersion::TwoComponents(1, 56));
        ctx.rust_releases.maximum_rust_version = Some(BareVersion::ThreeComponents(1, 54, 0));
        ctx.search_method = SearchMethod::Linear;

        let result = cmd.run(&ctx, reporter.get());
        let err = result.unwrap_err();

        assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(ref inner) if inner.has_clues()));

        let message = format!("{}", err);
        assert_eq!("No Rust releases to check: the filtered search space is empty. Search space limited by user to min Rust '1.56', and max Rust '1.54.0'", message);

        let events = reporter.wait_for_events();

        let unexpected_event: Event = FindResult::none(
            "x",
            BareVersion::TwoComponents(1, 56),
            BareVersion::ThreeComponents(1, 54, 0),
            SearchMethod::Linear,
        )
        .into();

        assert!(!events.contains(&unexpected_event));
    }
}

fn create_test_context() -> FindContext {
    FindContext {
        search_method: SearchMethod::Bisect,
        write_toolchain_file: false,
        ignore_lockfile: false,
        no_check_feedback: false,
        write_msrv: false,
        rust_releases: RustReleasesContext {
            minimum_rust_version: None,
            maximum_rust_version: None,
            consider_patch_releases: false,
            release_source: ReleaseSource::RustChangelog,
        },
        toolchain: ToolchainContext {
            target: "x".to_string(),
        },
        check_cmd: CheckCommandContext {
            cargo_features: None,
            cargo_all_features: false,
            cargo_no_default_features: false,
            rustup_command: None,
        },
        environment: EnvironmentContext {
            crate_path: Utf8PathBuf::new(),
        },
        user_output: UserOutputContext {
            output_format: OutputFormat::None,
        },
    }
}
