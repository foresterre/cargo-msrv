use crate::command::command;
use crate::config::Config;
use crate::crate_root_folder;
use crate::errors::{CargoMSRVError, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::reporter::{Output, ProgressAction};
use rust_releases::semver;
use std::fmt::Formatter;
use std::path::Path;

#[derive(Debug)]
pub enum CheckStatus {
    Success {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: semver::Version,
    },
    Failure {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: semver::Version,
        // cause of failure
        cause: Cause,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct Cause {
    cause: String,
}

impl Cause {
    pub fn from_stderr(stderr: &[u8]) -> Self {
        let content = String::from_utf8_lossy(stderr);

        Self {
            cause: content.to_string(),
        }
    }
}

impl std::fmt::Display for Cause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nCaused by:")?;
        for line in self.cause.lines() {
            if line.starts_with("To learn more") {
                return Ok(());
            }
            writeln!(f, "|    {}", line)?;
        }
        writeln!(f)
    }
}

pub fn check_toolchain<'a>(
    version: &'a semver::Version,
    config: &'a Config,
    output: &'a impl Output,
) -> TResult<CheckStatus> {
    // temporarily move the lockfile if the user opted to ignore it, and it exists
    let cargo_lock = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;
    let handle_wrap = if config.ignore_lockfile() && cargo_lock.is_file() {
        let handle = LockfileHandler::new(cargo_lock).move_lockfile()?;

        Some(handle)
    } else {
        None
    };

    let status = examine_toolchain(version, config, output)?;

    // move the lockfile back
    if let Some(handle) = handle_wrap {
        handle.move_lockfile_back()?;
    }

    Ok(status)
}

pub fn as_toolchain_specifier(version: &semver::Version, target: &str) -> String {
    format!("{}-{}", version, target)
}

fn examine_toolchain(
    version: &semver::Version,
    config: &Config,
    output: &impl Output,
) -> TResult<CheckStatus> {
    let toolchain_specifier = as_toolchain_specifier(version, config.target());

    download_if_required(version, &toolchain_specifier, output)?;

    if config.ignore_lockfile() {
        remove_lockfile(config)?;
    }

    try_building(
        version,
        &toolchain_specifier,
        config.crate_path(),
        config.check_command(),
        output,
    )
}

fn download_if_required(
    version: &semver::Version,
    toolchain_specifier: &str,
    output: &impl Output,
) -> TResult<()> {
    let toolchain = toolchain_specifier.to_owned();
    output.progress(ProgressAction::Installing, version);

    let status = command(&["install", "--profile", "minimal", &toolchain], None)
        .and_then(|mut c| c.wait().map_err(CargoMSRVError::Io))?;

    if !status.success() {
        return Err(CargoMSRVError::RustupInstallFailed(
            toolchain_specifier.to_string(),
        ));
    }

    Ok(())
}

fn remove_lockfile(config: &Config) -> TResult<()> {
    let lock_file = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;

    if lock_file.is_file() {
        std::fs::remove_file(lock_file).map_err(CargoMSRVError::Io)?;
    }

    Ok(())
}

fn try_building(
    version: &semver::Version,
    toolchain_specifier: &str,
    dir: Option<&Path>,
    check: &[&str],
    output: &impl Output,
) -> TResult<CheckStatus> {
    let mut cmd: Vec<&str> = vec!["run", toolchain_specifier];
    cmd.extend_from_slice(check);

    let child = command(&cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;
    output.progress(ProgressAction::Checking, version);

    let process_output = child.wait_with_output()?;
    let exit_status = process_output.status;
    let stderr = &process_output.stderr;

    output.complete_step(version, exit_status.success());

    if !exit_status.success() {
        Ok(CheckStatus::Failure {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
            cause: Cause::from_stderr(stderr),
        })
    } else {
        Ok(CheckStatus::Success {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    }
}
