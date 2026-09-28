//! Estimates the minimum Rust version a crate requires, by scanning its source code.
//!
//! The scan looks for evidence which requires at least a certain Rust version:
//!
//! * the edition of the crate, e.g. edition 2021 requires Rust 1.56,
//! * language features, e.g. let-else statements require Rust 1.65, and
//! * items of the standard library, e.g. `Option::is_some_and` requires Rust 1.70.
//!
//! The paths used by the crate are resolved through the imports of each file, the
//! prelude, and the primitive types, to the public paths of the standard library. The
//! scan does not check types, so method calls can only be matched by name. Evidence which
//! relies on such a guess has a [`Confidence::Low`].
//!
//! The highest version among the evidence is a lower bound for the MSRV. Features which
//! the scan does not know about may leave it too low.

#![deny(clippy::all)]

mod language_feature;
mod visitor;

#[cfg(test)]
mod tests;

pub use language_feature::LanguageFeature;

use camino::{Utf8Path, Utf8PathBuf};
use cargo_msrv_std_since::StdSinceIndex;
use std::collections::HashSet;
use std::{fmt, fs, io};
use syn::visit::Visit;
use visitor::{Imports, LocalNames, Usage, Visitor};

/// The paths which start at one of these crates are items of the standard library.
const STD_CRATES: [&str; 3] = ["std", "core", "alloc"];

const PRIMITIVE_TYPES: [&str; 17] = [
    "bool", "char", "str", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16",
    "u32", "u64", "u128", "usize",
];

/// The number of times an import may refer to another import, e.g. `use std::io; use io::Read;`.
const MAX_IMPORT_CHAIN: usize = 4;

/// Scans the crate located at `root`, which is the directory containing its `Cargo.toml`.
///
/// Scanned are the `Cargo.toml`, the `build.rs`, and the Rust files in the `src` directory.
pub fn scan_crate(root: &Utf8Path, index: &StdSinceIndex<'_>) -> Result<Scan, ScanError> {
    let edition = manifest_edition(root)?;
    let mut evidence = Vec::from_iter(edition.as_deref().and_then(edition_evidence));
    let mut skipped_files = Vec::new();
    let mut findings = Vec::new();

    for file in source_files(root)? {
        let contents = fs::read_to_string(&file).map_err(|source| ScanError::ReadFile {
            path: file.clone(),
            source,
        })?;

        let relative = file.strip_prefix(root).unwrap_or(&file).to_path_buf();

        match syn::parse_file(&contents) {
            Ok(ast) => {
                let mut visitor = Visitor::default();
                visitor.visit_file(&ast);
                findings.push((relative, visitor.findings));
            }
            Err(error) => skipped_files.push(SkippedFile {
                path: relative,
                reason: format!("{} (on line {})", error, error.span().start().line),
            }),
        }
    }

    let local_names = findings
        .iter()
        .fold(LocalNames::default(), |mut names, (_, findings)| {
            names.extend(&findings.local_names);
            names
        });

    let prelude = prelude_module(edition.as_deref());

    for (file, findings) in findings {
        let resolver = Resolver {
            index,
            imports: &findings.imports,
            local_names: &local_names,
            prelude,
        };
        evidence.extend(resolver.evidence(&file, findings.features, findings.usages));
    }

    Ok(Scan::new(evidence, skipped_files))
}

/// How certain it is that the evidence requires the Rust version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Confidence {
    /// The evidence relies on a guess, like a method call which is matched by its name only,
    /// or a name which may be provided by a glob import.
    Low,
    /// The evidence is the edition, a language feature, or a path which resolves to an item
    /// of the standard library.
    High,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::High => f.write_str("high"),
        }
    }
}

/// The outcome of a scan.
#[derive(Debug)]
pub struct Scan {
    /// One piece of evidence per requirement, sorted from the highest version to the lowest.
    evidence: Vec<Evidence>,
    skipped_files: Vec<SkippedFile>,
}

impl Scan {
    fn new(mut evidence: Vec<Evidence>, skipped_files: Vec<SkippedFile>) -> Self {
        // Evidence with a high confidence is kept over the same requirement with a low one.
        evidence.sort_by_key(|e| std::cmp::Reverse(e.confidence));
        let mut seen = HashSet::new();
        evidence.retain(|e| seen.insert(e.requirement.clone()));
        evidence.sort_by(|a, b| b.version.cmp(&a.version));

        Self {
            evidence,
            skipped_files,
        }
    }

    /// The evidence with at least the given confidence, which requires the highest Rust
    /// version, i.e. the lower bound of the MSRV.
    ///
    /// Returns `None` if nothing which requires a Rust version above 1.0 was found.
    pub fn lower_bound(&self, confidence: Confidence) -> Option<&Evidence> {
        self.evidence.iter().find(|e| e.confidence >= confidence)
    }

    /// All evidence, one per requirement, sorted from the highest version to the lowest.
    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }

    /// The files which could not be parsed, and were thus not scanned.
    pub fn skipped_files(&self) -> &[SkippedFile] {
        &self.skipped_files
    }
}

/// Something found in the crate, which requires at least a certain Rust version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub requirement: Requirement,
    pub version: semver::Version,
    pub confidence: Confidence,
    pub location: Location,
}

impl fmt::Display for Evidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} requires Rust {} ({}, {} confidence)",
            self.requirement, self.version, self.location, self.confidence
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    /// The `package.edition` of the Cargo manifest.
    Edition(String),
    LanguageFeature(LanguageFeature),
    /// An item of the standard library, as written in the source code, e.g. `fs::chown`.
    StdItem(String),
    /// A method call which matches a method of the standard library by name.
    StdMethod(String),
    StdMacro(String),
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edition(edition) => write!(f, "edition {edition}"),
            Self::LanguageFeature(feature) => write!(f, "{feature}"),
            Self::StdItem(path) => write!(f, "`{path}`"),
            Self::StdMethod(name) => write!(f, "method `.{name}()`"),
            Self::StdMacro(name) => write!(f, "macro `{name}!`"),
        }
    }
}

/// A location relative to the root of the scanned crate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    pub file: Utf8PathBuf,
    pub line: Option<usize>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{}", self.file, line),
            None => write!(f, "{}", self.file),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedFile {
    pub path: Utf8PathBuf,
    pub reason: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("unable to read the directory '{path}': {source}")]
    ReadDir {
        path: Utf8PathBuf,
        source: io::Error,
    },

    #[error("unable to read the file '{path}': {source}")]
    ReadFile {
        path: Utf8PathBuf,
        source: io::Error,
    },

    #[error("the path '{0}' is not valid UTF-8")]
    NonUtf8Path(std::path::PathBuf),

    #[error("unable to parse the Cargo manifest '{path}': {source}")]
    ParseManifest {
        path: Utf8PathBuf,
        // Boxed, since the error is large compared to the other variants.
        source: Box<toml_edit::TomlError>,
    },
}

/// Resolves the usages of a file to items of the standard library.
struct Resolver<'a> {
    index: &'a StdSinceIndex<'a>,
    imports: &'a Imports,
    local_names: &'a LocalNames,
    /// The module of the prelude of the edition, e.g. `rust_2021`.
    prelude: &'a str,
}

impl Resolver<'_> {
    fn evidence(
        &self,
        file: &Utf8Path,
        features: Vec<(LanguageFeature, usize)>,
        usages: Vec<(Usage, usize)>,
    ) -> Vec<Evidence> {
        let at = |line: usize| Location {
            file: file.to_path_buf(),
            line: Some(line),
        };

        let features = features.into_iter().map(|(feature, line)| Evidence {
            requirement: Requirement::LanguageFeature(feature),
            version: feature.since(),
            confidence: Confidence::High,
            location: at(line),
        });

        let std_items = usages.into_iter().filter_map(|(usage, line)| {
            let (requirement, version, confidence) = self.usage(usage)?;

            Some(Evidence {
                requirement,
                version,
                confidence,
                location: at(line),
            })
        });

        features
            .chain(std_items)
            // Anything available since 1.0.0 is not evidence of a higher MSRV.
            .filter(|evidence| evidence.version > semver::Version::new(1, 0, 0))
            .collect()
    }

    fn usage(&self, usage: Usage) -> Option<(Requirement, semver::Version, Confidence)> {
        match usage {
            Usage::Path(segments) => {
                let (version, confidence) = self.path(&segments)?;
                Some((
                    Requirement::StdItem(segments.join("::")),
                    version,
                    confidence,
                ))
            }
            Usage::Macro(segments) => {
                let (version, confidence) = self.macro_path(&segments)?;
                let name = segments.last()?.clone();
                Some((Requirement::StdMacro(name), version, confidence))
            }
            Usage::Method(name) => {
                if self.local_names.functions.contains(&name) {
                    return None;
                }
                let version = self.index.method(&name)?;
                Some((Requirement::StdMethod(name), version, Confidence::Low))
            }
        }
    }

    fn path(&self, segments: &[String]) -> Option<(semver::Version, Confidence)> {
        let (first, rest) = segments.split_first()?;
        let high = |version| Some((version, Confidence::High));

        if STD_CRATES.contains(&first.as_str()) {
            return high(self.index.resolve_path(segments)?);
        }

        if matches!(first.as_str(), "crate" | "self" | "super" | "Self") {
            return None;
        }

        // An imported name refers to the standard library only if its import does, e.g.
        // `fs::chown` after `use std::os::unix::fs;`, but not after `use tokio::fs;`.
        if self.imports.names.contains_key(first) {
            let import = self.import(first)?;
            let full: Vec<&str> = import.iter().chain(rest).map(String::as_str).collect();
            return high(self.index.resolve_path(&full)?);
        }

        if self.local_names.parents.contains(first) || self.local_names.functions.contains(first) {
            return None;
        }

        if PRIMITIVE_TYPES.contains(&first.as_str()) {
            return high(
                self.index
                    .resolve_from_item(self.index.path(first)?.item, rest)?,
            );
        }

        if let Some(version) = self.prelude_item(first, rest) {
            return high(version);
        }

        // A glob import may provide the name, but a local variable or another glob may
        // shadow it.
        self.imports
            .globs
            .iter()
            .filter_map(|glob| self.expand(glob, 0))
            .find_map(|glob| {
                let full: Vec<&str> = glob.iter().chain(segments).map(String::as_str).collect();
                self.index.resolve_path(&full)
            })
            .map(|version| (version, Confidence::Low))
    }

    fn macro_path(&self, segments: &[String]) -> Option<(semver::Version, Confidence)> {
        let [name] = segments else {
            return self.path(segments);
        };

        if self.local_names.macros.contains(name) {
            return None;
        }

        if self.imports.names.contains_key(name) {
            return self.path(segments);
        }

        // The macros of the standard library are in scope everywhere, so like for the
        // prelude, the version of the macro is used, not the version of its path. E.g.
        // `format_args!` is usable since 1.0, while `std::format_args` says 1.38.
        let entry = self.index.path(&format!("std::{name}"))?;
        let version = self.index.resolve_from_item(entry.item, &[] as &[&str])?;
        Some((version, Confidence::High))
    }

    /// The item of the prelude with the given name, or one of its associated items.
    fn prelude_item(&self, name: &str, rest: &[String]) -> Option<semver::Version> {
        let entry = [self.prelude, "v1"]
            .into_iter()
            .find_map(|module| self.index.path(&format!("std::prelude::{module}::{name}")))?;

        // An item of the prelude is in scope without naming the prelude module, so the
        // version of the item is used, not the version of the path through the prelude.
        self.index.resolve_from_item(entry.item, rest)
    }

    /// The path an imported name refers to, if the name is imported from the standard library.
    fn import(&self, name: &str) -> Option<Vec<String>> {
        let import = self.imports.names.get(name)?;
        self.expand(import, 0)
    }

    /// Expands a path which starts with an imported name, e.g. `io::Read` after
    /// `use std::io;`. Returns `None` if the path does not start at the standard library.
    fn expand(&self, path: &[String], depth: usize) -> Option<Vec<String>> {
        let (first, rest) = path.split_first()?;

        if STD_CRATES.contains(&first.as_str()) {
            return Some(path.to_vec());
        }

        if depth >= MAX_IMPORT_CHAIN {
            return None;
        }

        let import = self
            .imports
            .names
            .get(first)
            .filter(|import| *import != path)?;
        let expanded = self.expand(import, depth + 1)?;

        Some(expanded.into_iter().chain(rest.iter().cloned()).collect())
    }
}

fn manifest_edition(root: &Utf8Path) -> Result<Option<String>, ScanError> {
    let path = root.join("Cargo.toml");

    let contents = fs::read_to_string(&path).map_err(|source| ScanError::ReadFile {
        path: path.clone(),
        source,
    })?;

    let manifest = contents
        .parse::<toml_edit::DocumentMut>()
        .map_err(|source| ScanError::ParseManifest {
            path: path.clone(),
            source: Box::new(source),
        })?;

    // An edition inherited from the workspace (`edition.workspace = true`) is not a string,
    // and is not resolved.
    Ok(manifest
        .get("package")
        .and_then(|package| package.get("edition"))
        .and_then(|edition| edition.as_str())
        .map(ToString::to_string))
}

fn edition_evidence(edition: &str) -> Option<Evidence> {
    let minor = match edition {
        "2018" => 31,
        "2021" => 56,
        "2024" => 85,
        _ => return None,
    };

    Some(Evidence {
        requirement: Requirement::Edition(edition.to_string()),
        version: semver::Version::new(1, minor, 0),
        confidence: Confidence::High,
        location: Location {
            file: Utf8PathBuf::from("Cargo.toml"),
            line: None,
        },
    })
}

fn prelude_module(edition: Option<&str>) -> &'static str {
    match edition {
        Some("2018") => "rust_2018",
        Some("2021") => "rust_2021",
        Some("2024") => "rust_2024",
        _ => "rust_2015",
    }
}

fn source_files(root: &Utf8Path) -> Result<Vec<Utf8PathBuf>, ScanError> {
    let mut files = Vec::new();

    let build_script = root.join("build.rs");
    if build_script.is_file() {
        files.push(build_script);
    }

    let src = root.join("src");
    if src.is_dir() {
        collect_rust_files(&src, &mut files)?;
    }

    Ok(files)
}

fn collect_rust_files(dir: &Utf8Path, files: &mut Vec<Utf8PathBuf>) -> Result<(), ScanError> {
    let read_dir_error = |source| ScanError::ReadDir {
        path: dir.to_path_buf(),
        source,
    };

    let mut entries = fs::read_dir(dir)
        .map_err(read_dir_error)?
        .map(|entry| {
            let path = entry.map_err(read_dir_error)?.path();
            Utf8PathBuf::from_path_buf(path).map_err(ScanError::NonUtf8Path)
        })
        .collect::<Result<Vec<_>, _>>()?;

    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension() == Some("rs") {
            files.push(path);
        }
    }

    Ok(())
}
