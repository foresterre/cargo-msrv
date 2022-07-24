//! A module which helps creating "check commands", the command given to the rust toolchain to
//! determine whether a Rust version is compatible.

use crate::manifest::bare_version::BareVersion;
use crate::semver;

/// A command ran by the Rustup to determine whether a toolchain is compatible or not.
///
/// ToString implementation must give human readable summary of available commands.
// TBD: keep or remove ToString requirement
pub trait CheckCommand: ToString {
    /// A specific command for a given version.
    fn for_version(&self, version: &semver::Version) -> Result<&str, Error>;
}

#[derive(Debug, Clone)]
pub struct StaticCheckCommand {
    cmd: Option<String>,
}

impl StaticCheckCommand {
    const DEFAULT: &'static str = "cargo check";

    pub fn new<T: Into<String>>(cmd: T) -> Self {
        Self {
            cmd: Some(cmd.into()),
        }
    }
}

impl Default for StaticCheckCommand {
    fn default() -> Self {
        Self { cmd: None }
    }
}

impl CheckCommand for StaticCheckCommand {
    fn for_version(&self, _version: &semver::Version) -> Result<&str, Error> {
        Ok(self
            .cmd
            .as_deref()
            .unwrap_or_else(|| StaticCheckCommand::DEFAULT))
    }
}

impl ToString for StaticCheckCommand {
    fn to_string(&self) -> String {
        self.cmd
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| String::from(StaticCheckCommand::DEFAULT))
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("Unable to create check command for '{given_version}': '{reason}'")]
pub struct Error {
    given_version: BareVersion,
    reason: Reason,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Reason {
    #[error("Version could not be found")]
    NotFound,
}
