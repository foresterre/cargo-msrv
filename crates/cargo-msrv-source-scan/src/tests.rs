use super::*;
use assert_fs::prelude::*;
use cargo_msrv_std_since::source::Archive;
use std::sync::LazyLock;

static ARCHIVE: LazyLock<Archive> = LazyLock::new(cargo_msrv_std_since::source::bundled);
static INDEX: LazyLock<StdSinceIndex<'static>> = LazyLock::new(|| ARCHIVE.index().unwrap());

fn scan_source(manifest: &str, lib: &str) -> Scan {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("Cargo.toml").write_str(manifest).unwrap();
    dir.child("src/lib.rs").write_str(lib).unwrap();

    let root = Utf8Path::from_path(dir.path()).unwrap();
    scan_crate(root, &INDEX).unwrap()
}

fn manifest(edition: &str) -> String {
    format!("[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n")
}

fn lower_bound(scan: &Scan, confidence: Confidence) -> Option<(String, semver::Version)> {
    scan.lower_bound(confidence)
        .map(|e| (e.requirement.to_string(), e.version.clone()))
}

fn expected(requirement: &str, minor: u64) -> Option<(String, semver::Version)> {
    Some((requirement.to_string(), semver::Version::new(1, minor, 0)))
}

#[yare::parameterized(
    let_else = { "fn f(x: Option<u8>) { let Some(_) = x else { return }; }", "let-else statements", 65 },
    let_chains = { "fn f(x: Option<u8>) { if let Some(y) = x && y > 1 {} }", "let chains", 88 },
    async_closure = { "fn f() { let _ = async || {}; }", "async closures", 85 },
    inline_const = { "fn f() { let _ = const { 1 }; }", "inline const expressions", 79 },
    implicit_format_arguments = { "fn f(x: u8) { println!(\"{x}\"); }", "implicit format arguments", 58 },
    exclusive_range_pattern = { "fn f(x: u8) { match x { 0..5 => {}, _ => {} } }", "exclusive range patterns", 80 },
    expect_attribute = { "#[expect(dead_code)] fn f() {}", "the `#[expect]` lint attribute", 81 },
    gat = { "trait T { type A<'a>; }", "generic associated types", 65 },
    std_path = { "fn f() { std::hint::black_box(1); }", "`std::hint::black_box`", 66 },
    use_item = { "use core::hint::black_box; fn f() { black_box(1); }", "`core::hint::black_box`", 66 },
    reexport_newer_than_item = { "use core::ffi::CStr;", "`core::ffi::CStr`", 64 },
    imported_module = { "use std::os::unix::fs; fn f() { fs::chown(\"a\", None, None); }", "`fs::chown`", 73 },
    imported_through_import = { "use std::os::unix; use unix::fs::chown;", "`unix::fs::chown`", 73 },
    renamed_import = { "use std::sync::OnceLock as Once; fn f() { Once::<u8>::new(); }", "`std::sync::OnceLock`", 70 },
    prelude_associated_item = { "fn f(x: Option<u8>) -> bool { Option::is_some_and(x, |v| v > 1) }", "`Option::is_some_and`", 70 },
    primitive_associated_item = { "fn f(x: u32) -> u32 { u32::div_ceil(x, 2) }", "`u32::div_ceil`", 73 },
    std_macro = { "fn f() { todo!() }", "macro `todo!`", 40 },
    std_macro_path = { "fn f() { std::todo!() }", "macro `todo!`", 40 },
    builtin_macro_path = { "fn f() { let _ = std::format_args!(\"a\"); }", "macro `format_args!`", 38 },
)]
fn finds_lower_bound_with_high_confidence(lib: &str, requirement: &str, minor: u64) {
    let scan = scan_source(&manifest("2015"), lib);

    assert_eq!(
        lower_bound(&scan, Confidence::High),
        expected(requirement, minor)
    );
}

#[yare::parameterized(
    method = { "fn f(x: Option<u8>) -> bool { x.is_some_and(|v| v > 1) }", "method `.is_some_and()`", 70 },
    integer_method = { "fn f(x: u32) -> u32 { x.div_ceil(2) }", "method `.div_ceil()`", 73 },
    glob_import = { "use std::sync::*; fn f() { OnceLock::<u8>::new(); }", "`OnceLock::new`", 70 },
)]
fn finds_lower_bound_with_low_confidence_only(lib: &str, requirement: &str, minor: u64) {
    let scan = scan_source(&manifest("2015"), lib);

    assert_eq!(lower_bound(&scan, Confidence::High), None);
    assert_eq!(
        lower_bound(&scan, Confidence::Low),
        expected(requirement, minor)
    );
}

#[yare::parameterized(
    local_module = { "mod fs { pub fn chown() {} } fn f() { fs::chown() }" },
    module_of_other_crate = { "use tokio::fs; fn f() { fs::chown(\"a\", None, None); }" },
    item_of_other_crate = { "use tokio::fs::chown; fn f() { chown(\"a\", None, None); }" },
    local_method = { "struct S; impl S { fn is_some_and(&self) -> bool { true } } fn f(s: S) -> bool { s.is_some_and() }" },
    local_macro = { "macro_rules! todo { () => {} } fn f() { todo!() }" },
    imported_macro = { "use other::todo; fn f() { todo!() }" },
    local_type = { "struct Option; impl Option { fn is_some_and() {} } fn f() { Option::is_some_and() }" },
    self_path = { "struct S; impl S { fn is_some_and() {} fn f() { Self::is_some_and() } }" },
    reexport_older_than_item = { "use std::ffi::CStr;" },
    explicit_named_format_argument = { "fn f() { println!(\"{x}\", x = 1); }" },
    builtin_macro = { "fn f() { assert!(true); let _ = format_args!(\"a\"); }" },
)]
fn no_evidence(lib: &str) {
    let scan = scan_source(&manifest("2015"), lib);

    assert_eq!(lower_bound(&scan, Confidence::Low), None);
}

#[yare::parameterized(
    edition_2018 = { "2018", 31 },
    edition_2021 = { "2021", 56 },
    edition_2024 = { "2024", 85 },
)]
fn edition_is_evidence(edition: &str, minor: u64) {
    let scan = scan_source(&manifest(edition), "");

    assert_eq!(
        lower_bound(&scan, Confidence::High),
        expected(&format!("edition {edition}"), minor)
    );
}

#[yare::parameterized(
    edition_2015 = { &manifest("2015") },
    no_edition = { "[package]\nname = \"a\"\nversion = \"0.1.0\"\n" },
    inherited_edition = { "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition.workspace = true\n" },
)]
fn no_edition_evidence(manifest: &str) {
    let scan = scan_source(manifest, "fn f(x: Vec<u8>) -> usize { x.len() }");

    assert_eq!(lower_bound(&scan, Confidence::Low), None);
}

#[test]
fn evidence_is_sorted_and_deduplicated() {
    let lib = "\
use std::sync::OnceLock;
fn f(x: Option<u8>) -> bool {
    let Some(_) = x else { return false };
    let Some(_) = x else { return false };
    x.is_some_and(|v| v > 1)
}
";
    let scan = scan_source(&manifest("2021"), lib);

    let evidence: Vec<String> = scan.evidence().iter().map(ToString::to_string).collect();
    let lib = Utf8PathBuf::from("src").join("lib.rs");

    assert_eq!(
        evidence,
        [
            format!("`std::sync::OnceLock` requires Rust 1.70.0 ({lib}:1, high confidence)"),
            format!("method `.is_some_and()` requires Rust 1.70.0 ({lib}:5, low confidence)"),
            format!("let-else statements requires Rust 1.65.0 ({lib}:3, high confidence)"),
            "edition 2021 requires Rust 1.56.0 (Cargo.toml, high confidence)".to_string(),
        ]
    );
}

#[test]
fn unparsable_files_are_skipped() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("Cargo.toml")
        .write_str(&manifest("2015"))
        .unwrap();
    dir.child("src/lib.rs").write_str("mod a;").unwrap();
    dir.child("src/a.rs").write_str("fn f( {").unwrap();
    dir.child("build.rs")
        .write_str("fn main() { let Some(_) = None::<u8> else { return }; }")
        .unwrap();

    let root = Utf8Path::from_path(dir.path()).unwrap();
    let scan = scan_crate(root, &INDEX).unwrap();

    assert_eq!(scan.skipped_files().len(), 1);
    assert_eq!(scan.skipped_files()[0].path, Utf8PathBuf::from("src/a.rs"));
    assert_eq!(
        lower_bound(&scan, Confidence::High),
        expected("let-else statements", 65)
    );
}

#[test]
fn missing_manifest_is_an_error() {
    let dir = assert_fs::TempDir::new().unwrap();
    let root = Utf8Path::from_path(dir.path()).unwrap();

    assert!(matches!(
        scan_crate(root, &INDEX),
        Err(ScanError::ReadFile { .. })
    ));
}
