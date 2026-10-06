use crate::types::ParseEditionError;
use camino::Utf8PathBuf;
use cargo_msrv_rust_tools::ManifestParseError;
use std::path::PathBuf;

pub use cargo_msrv_types::{IoError, IoErrorSource};

pub type TResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to read the Cargo workspace at '{path}': {source}")]
    CargoMetadata {
        path: Utf8PathBuf,
        source: cargo_metadata::Error,
    },

    #[error("The '{flag}' flag requires a Cargo project, but no Cargo.toml was found at '{root}'")]
    CargoFlagWithoutCargoProject {
        flag: &'static str,
        root: Utf8PathBuf,
    },

    #[error("Shell completions do not have an analysis context.")]
    CompletionsHaveNoContext,

    #[error("The default host triple (target) could not be found.")]
    DefaultHostTripleNotFound,

    #[error("No packages selected. Did `--exclude` remove every workspace member?")]
    EmptySelection,

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    ManifestParseError(#[from] ManifestParseError),

    #[error(
        "No Cargo.toml manifest found at '{0}'. Use '--rust-version <VERSION>' to verify projects which don't use cargo."
    )]
    NoCargoManifest(Utf8PathBuf),

    #[error("{}", format_without_msrv(.0))]
    NoMSRVKeyInCargoToml(Vec<PackageWithoutMsrv>),

    #[error(
        "The selected packages have different MSRVs, but a custom check command runs once for all of them. Use '--rust-version <VERSION>', or select a single package with '--package <NAME>'."
    )]
    MixedMSRVs,

    #[error(transparent)]
    ParseEdition(#[from] ParseEditionError),

    #[error(transparent)]
    Path(#[from] PathError),

    #[error("Package '{0}' is not a member of this workspace")]
    UnknownPackage(String),
}

#[derive(Debug)]
pub struct PackageWithoutMsrv {
    pub name: String,
    pub manifest_path: Utf8PathBuf,
}

fn format_without_msrv(packages: &[PackageWithoutMsrv]) -> String {
    match packages {
        [package] => format!(
            "Unable to find key 'package.rust-version' (or 'package.metadata.msrv') in '{}'",
            package.manifest_path
        ),
        packages => format!(
            "Unable to find key 'package.rust-version' (or 'package.metadata.msrv') for {} packages: {}. \
             Use '--rust-version <VERSION>' to verify them against a single version, \
             or leave them out with '--exclude <NAME>'.",
            packages.len(),
            packages
                .iter()
                .map(|p| format!("'{}' ('{}')", p.name, p.manifest_path))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("'{}' does not exist", .0.display())]
    DoesNotExist(PathBuf),

    #[error("No parent directory for '{}'", .0.display())]
    NoParent(PathBuf),

    #[error(transparent)]
    InvalidUtf8(#[from] InvalidUtf8Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InvalidUtf8Error {
    error: Utf8PathErrorInner,
}

impl From<camino::FromPathError> for InvalidUtf8Error {
    fn from(value: camino::FromPathError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPath(value),
        }
    }
}

impl From<camino::FromPathBufError> for InvalidUtf8Error {
    fn from(value: camino::FromPathBufError) -> Self {
        Self {
            error: Utf8PathErrorInner::FromPathBuf(value),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Utf8PathErrorInner {
    #[error("Path contains non UTF-8 characters")]
    FromPath(camino::FromPathError),
    #[error("Path contains non UTF-8 characters (path: '{}')", .0.as_path().display())]
    FromPathBuf(camino::FromPathBufError),
}
