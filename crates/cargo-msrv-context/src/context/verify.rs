use crate::context::error::{Error, TResult};
use crate::context::{
    CheckCommandContext, EnvironmentContext, RustReleasesContext, ToolchainContext,
};
use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use cargo_msrv_manifest::CargoManifest;
use cargo_msrv_types::BareVersion;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct VerifyContext {
    /// The resolved Rust version, to check against for toolchain compatibility.
    pub rust_version: RustVersion,

    /// Ignore the lockfile for the MSRV verification
    pub ignore_lockfile: bool,

    /// Don't print the result of compatibility check
    pub no_check_feedback: bool,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for custom checks to be used with rustup
    pub check_cmd: CheckCommandContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

/// A combination of a bare (two- or three component) Rust version and the source which was used to
/// locate this version.
#[derive(Clone, Debug)]
pub struct RustVersion {
    rust_version: BareVersion,
    source: RustVersionSource,
}

impl RustVersion {
    pub fn from_arg(rust_version: BareVersion) -> Self {
        Self {
            rust_version,
            source: RustVersionSource::Arg,
        }
    }

    pub fn try_from_environment(env: &EnvironmentContext) -> TResult<Self> {
        let manifest_path = env.manifest();

        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()?;
        CargoManifest::try_from(metadata)?
            .minimum_rust_version()
            .ok_or_else(|| Error::NoMSRVKeyInCargoToml(manifest_path.clone()))
            .map(|v| RustVersion {
                rust_version: v.clone(),
                source: RustVersionSource::Manifest(manifest_path.clone()),
            })
    }

    /// Get the bare (two- or three component) version specifying the Rust version.
    pub fn version(&self) -> &BareVersion {
        &self.rust_version
    }

    /// Get the version and discard all else.
    pub fn into_version(self) -> BareVersion {
        self.rust_version
    }

    /// Get the version and the source which was used to locate it.
    pub fn into_parts(self) -> (BareVersion, RustVersionSource) {
        (self.rust_version, self.source)
    }
}

/// Source used to obtain a Rust version for the verifier.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RustVersionSource {
    #[error("as --rust-version argument")]
    Arg,

    #[error("as MSRV in the Cargo manifest located at '{0}'")]
    Manifest(Utf8PathBuf),
}
