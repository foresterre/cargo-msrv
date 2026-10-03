use crate::common::Fixture;
use crate::common::reporter::EventTestDevice;
use crate::common::sub_cmd_find::find_msrv_with_releases;
use crate::common::sub_cmd_verify::run_verify;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::context::error::Error as ContextError;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::reporter::{Message, SubcommandResult};
use cargo_msrv::{Context, Set, Show, SubCommand};
use cargo_msrv_context::types::Edition;
use cargo_msrv_context::{Package, Project};
use cargo_msrv_rust_releases::{RustRelease, Stable};
use std::convert::TryFrom;

mod common;

fn context(args: &[&str]) -> Result<Context, ContextError> {
    let args = ["cargo", "msrv"].iter().chain(args);
    let opts = CargoCli::parse_args(args).to_cargo_msrv_cli().to_opts();

    Context::try_from(opts)
}

fn selected(ctx: &Context) -> Vec<Package> {
    match &ctx.environment_context().project {
        Project::Cargo(project) => project.packages().cloned().collect(),
        Project::Bare => panic!("expected a Cargo project"),
    }
}

fn names(packages: &[Package]) -> Vec<&str> {
    packages.iter().map(|p| p.name.as_str()).collect()
}

fn show(args: &[&str]) -> (Vec<(String, String)>, Result<(), CargoMSRVError>) {
    let ctx = context(args).unwrap();
    let Context::Show(show_ctx) = ctx else {
        panic!("expected show")
    };

    let device = EventTestDevice::default();
    let result = Show.run(&show_ctx, device.reporter());

    let shown = device
        .wait_for_events()
        .into_iter()
        .filter_map(|event| match event.message() {
            Message::SubcommandResult(SubcommandResult::Show(result)) => Some((
                result.package().unwrap().name.clone(),
                result.version().to_string(),
            )),
            _ => None,
        })
        .collect();

    (shown, result)
}

fn set(args: &[&str]) -> Result<(), CargoMSRVError> {
    let ctx = context(args).unwrap();
    let Context::Set(set_ctx) = ctx else {
        panic!("expected set")
    };

    let device = EventTestDevice::default();
    Set::new(None).run(&set_ctx, device.reporter())
}

#[test]
fn virtual_workspace_selects_every_member() {
    let fixture = Fixture::new("virtual-workspace");

    let ctx = context(&["--path", fixture.to_str(), "show"]).unwrap();

    assert_eq!(names(&selected(&ctx)), ["a", "b"]);
}

#[test]
fn member_directory_selects_only_that_member() {
    let fixture = Fixture::new("virtual-workspace");
    let member = fixture.tmp_path("a");

    let ctx = context(&["--path", member.to_str().unwrap(), "show"]).unwrap();
    let packages = selected(&ctx);

    assert_eq!(names(&packages), ["a"]);
    assert_eq!(
        ctx.environment_context().lock().unwrap(),
        fixture.tmp_path("Cargo.lock")
    );
}

#[test]
fn root_package_is_the_default_selection() {
    let fixture = Fixture::new("workspace-root-package");

    let ctx = context(&["--path", fixture.to_str(), "show"]).unwrap();
    assert_eq!(names(&selected(&ctx)), ["root"]);

    let ctx = context(&["--path", fixture.to_str(), "show", "--workspace"]).unwrap();
    assert_eq!(names(&selected(&ctx)), ["member", "root"]);
}

#[test]
fn package_flag_selects_a_member() {
    let fixture = Fixture::new("workspace-root-package");

    let ctx = context(&["--path", fixture.to_str(), "show", "-p", "member"]).unwrap();

    assert_eq!(names(&selected(&ctx)), ["member"]);
}

#[test]
fn inherited_rust_version_and_edition() {
    let fixture = Fixture::new("workspace-package-inheritance");

    let ctx = context(&["--path", fixture.to_str(), "find"]).unwrap();
    let packages = selected(&ctx);

    assert_eq!(names(&packages), ["a"]);
    assert_eq!(packages[0].edition, Edition::Edition2021);
    assert_eq!(
        packages[0].rust_version.as_ref().unwrap().to_string(),
        "1.66.0"
    );

    let find_ctx = ctx.to_find_context().unwrap();
    let min = find_ctx
        .rust_releases
        .resolve_minimum_version(Some(&packages[0]));
    assert_eq!(min.unwrap().to_string(), "1.56.0");
}

#[test]
fn excluding_every_member_is_an_empty_selection() {
    let fixture = Fixture::new("virtual-workspace");

    let result = context(&[
        "--path",
        fixture.to_str(),
        "--workspace",
        "--exclude",
        "a",
        "--exclude",
        "b",
        "show",
    ]);

    assert!(matches!(result, Err(ContextError::EmptySelection)));
}

#[test]
fn unknown_package() {
    let fixture = Fixture::new("virtual-workspace");

    let result = context(&["--path", fixture.to_str(), "-p", "does-not-exist", "show"]);

    assert!(matches!(result, Err(ContextError::UnknownPackage(name)) if name == "does-not-exist"));
}

#[yare::parameterized(
    workspace = { &["--workspace"], "--workspace" },
    package = { &["--package", "a"], "--package" },
    exclude = { &["--exclude", "a"], "--exclude" },
)]
fn cargo_flag_without_cargo_project(flags: &[&str], expected: &str) {
    let dir = assert_fs::TempDir::new().unwrap();

    let mut args = vec!["--path", dir.to_str().unwrap()];
    args.extend_from_slice(flags);
    args.push("show");

    let result = context(&args);

    assert!(matches!(
        result,
        Err(ContextError::CargoFlagWithoutCargoProject { flag, .. }) if flag == expected
    ));
}

#[test]
fn broken_manifest_is_not_a_bare_project() {
    let dir = assert_fs::TempDir::new().unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package\n").unwrap();

    let result = context(&["--path", dir.to_str().unwrap(), "show"]);

    assert!(matches!(result, Err(ContextError::CargoMetadata { .. })));
}

#[test]
fn show_every_member() {
    let fixture = Fixture::new("virtual-workspace");

    let (shown, result) = show(&["--path", fixture.to_str(), "show"]);

    result.unwrap();
    assert_eq!(
        shown,
        [
            ("a".to_string(), "1.56.0".to_string()),
            ("b".to_string(), "1.58.0".to_string())
        ]
    );
}

#[test]
fn show_reports_members_without_msrv() {
    let fixture = Fixture::new("virtual-workspace");
    std::fs::write(
        fixture.tmp_path("a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let (shown, result) = show(&["--path", fixture.to_str(), "show"]);

    assert_eq!(shown, [("b".to_string(), "1.58.0".to_string())]);
    let err = result.unwrap_err();
    assert!(matches!(err, CargoMSRVError::SubCommandShow(_)));
    assert!(err.to_string().contains("a/Cargo.toml"));
}

#[test]
fn set_every_selected_member() {
    let fixture = Fixture::new("virtual-workspace");

    set(&["--path", fixture.to_str(), "set", "1.70"]).unwrap();

    for member in ["a", "b"] {
        let manifest =
            std::fs::read_to_string(fixture.tmp_path(member).join("Cargo.toml")).unwrap();
        assert!(manifest.contains("rust-version = \"1.70\""), "{manifest}");
    }
}

#[test]
fn set_does_not_replace_inherited_msrv() {
    let fixture = Fixture::new("workspace-package-inheritance");
    let before = std::fs::read_to_string(fixture.tmp_path("a/Cargo.toml")).unwrap();

    let err = set(&["--path", fixture.to_str(), "set", "1.70"]).unwrap_err();

    assert!(matches!(err, CargoMSRVError::InheritedMsrv { ref package } if package == "a"));
    let after = std::fs::read_to_string(fixture.tmp_path("a/Cargo.toml")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn set_workspace_root() {
    let fixture = Fixture::new("workspace-package-inheritance");
    let member = fixture.tmp_path("a");

    set(&[
        "--path",
        member.to_str().unwrap(),
        "set",
        "1.70",
        "--workspace-root",
    ])
    .unwrap();

    let root = std::fs::read_to_string(fixture.tmp_path("Cargo.toml")).unwrap();
    assert!(root.contains("rust-version = \"1.70\""), "{root}");
}

#[test]
fn set_workspace_root_without_cargo_project() {
    let dir = assert_fs::TempDir::new().unwrap();

    let result = context(&[
        "--path",
        dir.to_str().unwrap(),
        "set",
        "1.70",
        "--workspace-root",
    ]);

    assert!(matches!(
        result,
        Err(ContextError::CargoFlagWithoutCargoProject {
            flag: "--workspace-root",
            ..
        })
    ));
}

#[test]
fn find_every_member_of_a_virtual_workspace() {
    let fixture = Fixture::new("virtual-workspace");
    let with_args = vec!["cargo", "msrv", "--path", fixture.to_str(), "find"];

    let versions = vec![
        RustRelease::new(Stable::new(1, 58, 1), None, []),
        RustRelease::new(Stable::new(1, 56, 1), None, []),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();

    assert_eq!(
        test_result.package_msrvs(),
        [
            (Some("a".to_string()), Some(semver::Version::new(1, 56, 1))),
            (Some("b".to_string()), Some(semver::Version::new(1, 58, 1))),
        ]
    );
}

#[test]
fn verify_every_member_of_a_virtual_workspace() {
    let fixture = Fixture::new("virtual-workspace");
    for (member, rust_version) in [("a", "1.56.1"), ("b", "1.58.1")] {
        std::fs::write(
            fixture.tmp_path(member).join("Cargo.toml"),
            format!(
                "[package]\nname = \"{member}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"{rust_version}\"\n"
            ),
        )
        .unwrap();
    }
    let with_args = vec!["cargo", "msrv", "--path", fixture.to_str(), "verify"];

    let result = run_verify(
        with_args,
        vec![
            RustRelease::new(Stable::new(1, 58, 1), None, []),
            RustRelease::new(Stable::new(1, 56, 1), None, []),
        ],
    );

    result.unwrap();
}

#[test]
fn verify_fails_when_a_member_is_incompatible() {
    let fixture = Fixture::new("virtual-workspace");
    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        fixture.to_str(),
        "verify",
        "--rust-version",
        "1.56.1",
    ];

    let err = run_verify(
        with_args,
        vec![
            RustRelease::new(Stable::new(1, 58, 1), None, []),
            RustRelease::new(Stable::new(1, 56, 1), None, []),
        ],
    )
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("Package 'b'"), "{message}");
    assert!(!message.contains("Package 'a'"), "{message}");
}

#[test]
fn bare_project_verify_with_custom_command() {
    let dir = assert_fs::TempDir::new().unwrap();
    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        dir.to_str().unwrap(),
        "verify",
        "--rust-version",
        "1.58.1",
        "--",
        "rustc",
        "--version",
    ];

    let result = run_verify(
        with_args,
        vec![RustRelease::new(Stable::new(1, 58, 1), None, [])],
    );

    result.unwrap();
}

#[test]
fn bare_project_find_with_custom_command() {
    let dir = assert_fs::TempDir::new().unwrap();
    let with_args = vec![
        "cargo",
        "msrv",
        "--path",
        dir.to_str().unwrap(),
        "find",
        "--",
        "rustc",
        "--version",
    ];

    let versions = vec![
        RustRelease::new(Stable::new(1, 58, 1), None, []),
        RustRelease::new(Stable::new(1, 56, 1), None, []),
    ];

    let test_result = find_msrv_with_releases(with_args, versions).unwrap();

    assert_eq!(
        test_result.package_msrvs(),
        [(None, Some(semver::Version::new(1, 56, 1)))]
    );
}
