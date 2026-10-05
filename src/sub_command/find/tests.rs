use super::*;
use crate::compatibility::TestRunner;
use crate::context::{
    CheckCommandContext, EnvironmentContext, Project, RustReleasesContext, ToolchainContext,
};
use crate::reporter::Event;
use crate::reporter::TestReporterWrapper;
use camino::{Utf8Path, Utf8PathBuf};
use std::iter::FromIterator;
use version_number::Version;

#[test]
fn bisect_find_only_last() {
    let index = ReleaseIndex::from_iter(vec![
        RustRelease::new(Stable::new(1, 56, 0), None, []),
        RustRelease::new(Stable::new(1, 55, 0), None, []),
        RustRelease::new(Stable::new(1, 54, 0), None, []),
        RustRelease::new(Stable::new(1, 53, 0), None, []),
        RustRelease::new(Stable::new(1, 52, 0), None, []),
        RustRelease::new(Stable::new(1, 51, 0), None, []),
        RustRelease::new(Stable::new(1, 50, 0), None, []),
        RustRelease::new(Stable::new(1, 49, 0), None, []),
        RustRelease::new(Stable::new(1, 48, 0), None, []),
        RustRelease::new(Stable::new(1, 47, 0), None, []),
        RustRelease::new(Stable::new(1, 46, 0), None, []),
        RustRelease::new(Stable::new(1, 45, 0), None, []),
        RustRelease::new(Stable::new(1, 44, 0), None, []),
        RustRelease::new(Stable::new(1, 43, 0), None, []),
        RustRelease::new(Stable::new(1, 42, 0), None, []),
        RustRelease::new(Stable::new(1, 41, 0), None, []),
        RustRelease::new(Stable::new(1, 40, 0), None, []),
        RustRelease::new(Stable::new(1, 39, 0), None, []),
        RustRelease::new(Stable::new(1, 38, 0), None, []),
        RustRelease::new(Stable::new(1, 37, 0), None, []),
    ]);

    let reporter = TestReporterWrapper::default();
    let runner = TestRunner::with_ok("x", &[semver::Version::new(1, 56, 0)]);

    let cmd = Find::new(&index, |_| &runner);
    let mut context = create_test_context();
    context.search_method = SearchMethod::Bisect;
    // necessary currently, otherwise our own cargo manifest edition is used (ugh),
    // which now is 2021, i.e. >= 1.56, and that breaks the tests
    context.rust_releases.minimum_rust_version = Some(Version::new_full_version(1, 37, 0));

    let found = cmd.run(&context, reporter.get()).unwrap();
    assert_eq!(found, semver::Version::new(1, 56, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![
        FindResult::new_msrv(
            semver::Version::new(1, 56, 0),
            "x",
            Version::new_full_version(1, 37, 0),
            Version::new_full_version(1, 56, 0),
            SearchMethod::Bisect,
        )
        .into(),
    ];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
}

#[test]
fn bisect_find_all_compatible() {
    let index = ReleaseIndex::from_iter(vec![
        RustRelease::new(Stable::new(1, 56, 0), None, []),
        RustRelease::new(Stable::new(1, 55, 0), None, []),
        RustRelease::new(Stable::new(1, 54, 0), None, []),
        RustRelease::new(Stable::new(1, 53, 0), None, []),
        RustRelease::new(Stable::new(1, 52, 0), None, []),
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

    let cmd = Find::new(&index, |_| &runner);
    let mut ctx = create_test_context();
    ctx.search_method = SearchMethod::Bisect;
    // necessary currently, otherwise our own cargo manifest edition is used (ugh),
    // which now is 2021, i.e. >= 1.56, and that breaks the tests
    ctx.rust_releases.minimum_rust_version = Some(Version::new_full_version(1, 52, 0));

    let found = cmd.run(&ctx, reporter.get()).unwrap();
    assert_eq!(found, semver::Version::new(1, 52, 0));

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![
        FindResult::new_msrv(
            semver::Version::new(1, 52, 0),
            "x",
            Version::new_full_version(1, 52, 0),
            Version::new_full_version(1, 56, 0),
            SearchMethod::Bisect,
        )
        .into(),
    ];

    phenomenon::contains_at_least_ordered(events, expected).assert_this();
}

#[test]
fn bisect_none_compatible() {
    let index = ReleaseIndex::from_iter(vec![
        RustRelease::new(Stable::new(1, 56, 0), None, []),
        RustRelease::new(Stable::new(1, 55, 0), None, []),
        RustRelease::new(Stable::new(1, 54, 0), None, []),
        RustRelease::new(Stable::new(1, 53, 0), None, []),
        RustRelease::new(Stable::new(1, 52, 0), None, []),
    ]);

    let reporter = TestReporterWrapper::default();
    let runner = TestRunner::with_ok("x", &[]);

    let cmd = Find::new(&index, |_| &runner);
    let mut ctx = create_test_context();
    ctx.search_method = SearchMethod::Bisect;
    // necessary currently, otherwise our own cargo manifest edition is used (ugh),
    // which now is 2021, i.e. >= 1.56, and that breaks the tests
    ctx.rust_releases.minimum_rust_version = Some(Version::new_full_version(1, 52, 0));

    let result = cmd.run(&ctx, reporter.get());
    assert!(result.is_err());

    let events = reporter.wait_for_events();
    let expected: Vec<Event> = vec![
        FindResult::none(
            "x",
            Version::new_full_version(1, 52, 0),
            Version::new_full_version(1, 56, 0),
            SearchMethod::Bisect,
        )
        .into(),
    ];

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
            RustRelease::new(Stable::new(1, 46, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 57, 0), None, []),
            RustRelease::new(Stable::new(1, 58, 1), None, []),
            RustRelease::new(Stable::new(1, 59, 0), None, []),
        ];

        let index = ReleaseIndex::from_iter(releases);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok("x", &[]);

        let cmd = Find::new(&index, |_| &runner);
        let mut ctx = create_test_context();

        // Create a negative search space, by setting min > max, effectively emptying it.
        ctx.rust_releases.minimum_rust_version = Some(Version::new_base_version(1, 56));
        ctx.rust_releases.maximum_rust_version = Some(Version::new_full_version(1, 54, 0));
        ctx.search_method = SearchMethod::Bisect;

        let result = cmd.run(&ctx, reporter.get());
        let err = result.unwrap_err();

        assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(ref inner) if inner.has_clues()));

        let message = format!("{}", err);
        assert_eq!(
            "No Rust releases to check: the filtered search space is empty. Search space limited by user to min Rust '1.56', and max Rust '1.54.0'",
            message
        );

        let events = reporter.wait_for_events();

        let unexpected_event: Event = FindResult::none(
            "x",
            Version::new_base_version(1, 56),
            Version::new_full_version(1, 54, 0),
            SearchMethod::Bisect,
        )
        .into();

        assert!(!events.contains(&unexpected_event));
    }

    #[test]
    fn linear() {
        let releases = vec![
            RustRelease::new(Stable::new(1, 46, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 57, 0), None, []),
            RustRelease::new(Stable::new(1, 58, 1), None, []),
            RustRelease::new(Stable::new(1, 59, 0), None, []),
        ];

        let index = ReleaseIndex::from_iter(releases);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok("x", &[]);

        let cmd = Find::new(&index, |_| &runner);
        let mut ctx = create_test_context();

        // Create a negative search space, by setting min > max, effectively emptying it.
        ctx.rust_releases.minimum_rust_version = Some(Version::new_base_version(1, 56));
        ctx.rust_releases.maximum_rust_version = Some(Version::new_full_version(1, 54, 0));
        ctx.search_method = SearchMethod::Linear;

        let result = cmd.run(&ctx, reporter.get());
        let err = result.unwrap_err();

        assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(ref inner) if inner.has_clues()));

        let message = format!("{}", err);
        assert_eq!(
            "No Rust releases to check: the filtered search space is empty. Search space limited by user to min Rust '1.56', and max Rust '1.54.0'",
            message
        );

        let events = reporter.wait_for_events();

        let unexpected_event: Event = FindResult::none(
            "x",
            Version::new_base_version(1, 56),
            Version::new_full_version(1, 54, 0),
            SearchMethod::Linear,
        )
        .into();

        assert!(!events.contains(&unexpected_event));
    }
}

mod unavailable_toolchains {
    use super::*;
    use cargo_msrv_rust_releases::release_toolchain::{Channel, Target, Toolchain};

    const HOST: &str = "x86_64-unknown-linux-gnu";
    const OTHER: &str = "aarch64-apple-darwin";

    fn release(minor: u64, host: &str) -> RustRelease<Stable> {
        let version = Stable::new(1, minor, 0);
        let target = Target::try_from_target_triple(host).unwrap();

        RustRelease::new(
            version.clone(),
            None,
            [Toolchain::new(
                Channel::Stable(version),
                None,
                target.clone(),
                Vec::new().into_iter().collect(),
                vec![target].into_iter().collect(),
            )],
        )
    }

    fn create_context() -> FindContext {
        let mut ctx = create_test_context();

        ctx.toolchain = ToolchainContext {
            host: HOST,
            target: HOST,
            components: &[],
        };
        ctx.rust_releases.minimum_rust_version = Some(Version::new_full_version(1, 57, 0));

        ctx
    }

    #[test]
    fn releases_without_the_requested_toolchain_are_not_searched() {
        let index = ReleaseIndex::from_iter(vec![release(58, HOST), release(57, OTHER)]);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok(HOST, &[semver::Version::new(1, 58, 0)]);

        let cmd = Find::new(&index, |_| &runner);
        let found = cmd.run(&create_context(), reporter.get()).unwrap();

        assert_eq!(found, semver::Version::new(1, 58, 0));
    }

    #[test]
    fn an_empty_search_space_reports_the_excluded_releases() {
        let index = ReleaseIndex::from_iter(vec![release(58, OTHER), release(57, OTHER)]);

        let reporter = TestReporterWrapper::default();
        let runner = TestRunner::with_ok(HOST, &[]);

        let cmd = Find::new(&index, |_| &runner);
        let err = cmd.run(&create_context(), reporter.get()).unwrap_err();

        assert!(matches!(err, CargoMSRVError::NoToolchainsToTry(ref inner) if inner.has_clues()));

        assert_eq!(
            format!("{}", err),
            "No Rust releases to check: the filtered search space is empty. \
             Search space limited by user to min Rust '1.57.0', and max Rust '<not overridden>' \
             Excluded 2 release(s) which do not provide the requested toolchain \
             (target 'x86_64-unknown-linux-gnu'): \
             Rust 1.58.0 (no toolchain for host 'x86_64-unknown-linux-gnu'), \
             Rust 1.57.0 (no toolchain for host 'x86_64-unknown-linux-gnu')"
        );
    }
}

mod workspace {
    use super::*;
    use crate::reporter::Message;
    use crate::reporter::event::SubcommandResult;
    use assert_fs::prelude::*;
    use cargo_msrv_context::types::Edition;
    use cargo_msrv_context::{CargoProject, Package};

    fn index() -> ReleaseIndex {
        ReleaseIndex::from_iter(vec![
            RustRelease::new(Stable::new(1, 58, 0), None, []),
            RustRelease::new(Stable::new(1, 57, 0), None, []),
            RustRelease::new(Stable::new(1, 56, 0), None, []),
        ])
    }

    fn package(root: &Utf8Path, name: &str) -> Package {
        Package {
            name: name.to_string(),
            manifest_path: root.join(name).join("Cargo.toml"),
            rust_version: None,
            edition: Edition::Edition2021,
        }
    }

    fn workspace(tmp: &assert_fs::TempDir) -> FindContext {
        for name in ["a", "b"] {
            tmp.child(name)
                .child("Cargo.toml")
                .write_str(&format!("[package]\nname = \"{name}\"\n"))
                .unwrap();
        }

        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let project = CargoProject::new(
            root.to_path_buf(),
            vec![package(root, "a"), package(root, "b")],
        )
        .unwrap();

        let mut ctx = create_test_context();
        ctx.environment = EnvironmentContext {
            root_crate_path: root.to_path_buf(),
            project: Project::Cargo(project),
        };
        ctx
    }

    fn runner_for(target: CheckTarget) -> TestRunner {
        let command = target.run_command.components().join(" ");
        let accepted = if command.contains("--package a") {
            vec![
                semver::Version::new(1, 56, 0),
                semver::Version::new(1, 57, 0),
                semver::Version::new(1, 58, 0),
            ]
        } else if command.contains("--package b") {
            vec![semver::Version::new(1, 58, 0)]
        } else {
            vec![]
        };

        TestRunner::with_ok("x", &accepted)
    }

    fn found_msrvs(events: &[Event]) -> Vec<(String, Option<semver::Version>)> {
        events
            .iter()
            .filter_map(|event| match event.message() {
                Message::SubcommandResult(SubcommandResult::Find(result)) => Some((
                    result.package().unwrap().name.clone(),
                    result.msrv().cloned(),
                )),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn each_package_has_its_own_msrv() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let ctx = workspace(&tmp);
        let index = index();
        let reporter = TestReporterWrapper::default();

        let found = Find::new(&index, runner_for)
            .run(&ctx, reporter.get())
            .unwrap();

        assert_eq!(found, semver::Version::new(1, 58, 0));

        let events = reporter.wait_for_events();
        assert_eq!(
            found_msrvs(&events),
            [
                ("a".to_string(), Some(semver::Version::new(1, 56, 0))),
                ("b".to_string(), Some(semver::Version::new(1, 58, 0))),
            ]
        );

        let package_scopes = events
            .iter()
            .filter(|event| matches!(event.message(), Message::CheckPackage(_)))
            .count();
        assert_eq!(
            package_scopes, 4,
            "expected a start and end event for each package"
        );
    }

    #[test]
    fn write_msrv_per_package_and_highest_toolchain_file() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut ctx = workspace(&tmp);
        ctx.write_msrv = true;
        ctx.write_toolchain_file = true;
        let index = index();
        let reporter = TestReporterWrapper::default();

        Find::new(&index, runner_for)
            .run(&ctx, reporter.get())
            .unwrap();

        tmp.child("a/Cargo.toml")
            .assert("[package]\nname = \"a\"\nrust-version = \"1.56\"\n");
        tmp.child("b/Cargo.toml")
            .assert("[package]\nname = \"b\"\nrust-version = \"1.58\"\n");
        tmp.child("rust-toolchain")
            .assert("[toolchain]\nchannel = \"1.58.0\"\n");
    }

    #[test]
    fn a_failing_package_does_not_stop_the_others() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut ctx = workspace(&tmp);
        let Project::Cargo(project) = &mut ctx.environment.project else {
            unreachable!()
        };
        project.first.name = "c".to_string();
        ctx.write_toolchain_file = true;
        let index = index();
        let reporter = TestReporterWrapper::default();

        let err = Find::new(&index, runner_for)
            .run(&ctx, reporter.get())
            .unwrap_err();

        assert!(matches!(
            err,
            CargoMSRVError::UnableToFindAnyGoodVersion { ref commands }
                if commands == &["cargo check --package c --target x"]
        ));
        assert_eq!(
            found_msrvs(&reporter.wait_for_events()),
            [
                ("c".to_string(), None),
                ("b".to_string(), Some(semver::Version::new(1, 58, 0))),
            ]
        );
        assert!(!tmp.child("rust-toolchain").path().exists());
    }

    #[test]
    fn custom_command_runs_once_for_the_workspace() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut ctx = workspace(&tmp);
        ctx.check_cmd.rustup_command = Some(vec!["cargo".into(), "check".into()]);
        let index = index();
        let reporter = TestReporterWrapper::default();

        let found = Find::new(&index, |_| {
            TestRunner::with_ok("x", &[semver::Version::new(1, 57, 0)])
        })
        .run(&ctx, reporter.get())
        .unwrap();

        assert_eq!(found, semver::Version::new(1, 57, 0));

        let results = reporter
            .wait_for_events()
            .into_iter()
            .filter_map(|event| match event.message() {
                Message::SubcommandResult(SubcommandResult::Find(result)) => {
                    Some(result.package().is_none())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(results, [true]);
    }
}

fn create_test_context() -> FindContext {
    FindContext {
        search_method: SearchMethod::Bisect,
        write_toolchain_file: false,
        ignore_lockfile: false,
        skip_unavailable_toolchains: true,
        no_check_feedback: false,
        write_msrv: false,
        rust_releases: RustReleasesContext::default(),
        toolchain: ToolchainContext {
            host: "x",
            target: "x",
            components: &[],
        },
        check_cmd: CheckCommandContext {
            cargo_features: None,
            cargo_all_features: false,
            cargo_no_default_features: false,
            rustup_command: None,
        },
        environment: EnvironmentContext {
            root_crate_path: Utf8PathBuf::new(),
            project: Project::Bare,
        },
    }
}
