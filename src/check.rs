use crate::command::command;
use crate::config::Config;
use crate::crate_root_folder;
use crate::errors::{CargoMSRVError, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::reporter::{Output, ProgressAction};
use rust_releases::semver;
use std::path::Path;

#[derive(Clone, Debug)]
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
    },
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

    let mut child = command(&cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;
    output.progress(ProgressAction::Checking, version);

    let status = child.wait()?;

    output.complete_step(version, status.success());

    if !status.success() {
        Ok(CheckStatus::Failure {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    } else {
        Ok(CheckStatus::Success {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    }
}
