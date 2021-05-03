use crate::command::command;
use crate::config::Config;
use crate::crate_root_folder;
use crate::errors::{CargoMSRVError, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::ui::Printer;
use console::style;
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
    ui: &'a Printer,
) -> TResult<CheckStatus> {
    // temporarily move the lockfile if the user opted to ignore it, and it exists
    let cargo_lock = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;
    let handle_wrap = if config.ignore_lockfile() && cargo_lock.is_file() {
        let handle = LockfileHandler::new(cargo_lock).move_lockfile()?;

        Some(handle)
    } else {
        None
    };

    let status = examine_toolchain(version, config, ui)?;

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
    ui: &Printer,
) -> TResult<CheckStatus> {
    let toolchain_specifier = as_toolchain_specifier(version, config.target());

    download_if_required(version, &toolchain_specifier, ui)?;

    if config.ignore_lockfile() {
        remove_lockfile(config)?;
    }

    try_building(
        version,
        &toolchain_specifier,
        config.crate_path(),
        config.check_command(),
        ui,
    )
}

fn download_if_required(
    version: &semver::Version,
    toolchain_specifier: &str,
    ui: &Printer,
) -> TResult<()> {
    let toolchain = toolchain_specifier.to_owned();
    ui.show_progress("Installing", version);

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
    ui: &Printer,
) -> TResult<CheckStatus> {
    let mut cmd: Vec<&str> = vec!["run", toolchain_specifier];
    cmd.extend_from_slice(check);

    let mut child = command(&cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;
    ui.show_progress("Checking", version);

    let status = child.wait()?;

    if !status.success() {
        ui.complete_step(format!(
            "{} Bad check for {}",
            style("Done").green().bold(),
            style(version).cyan()
        ));

        Ok(CheckStatus::Failure {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    } else {
        ui.complete_step(format!(
            "{} Good check for {}",
            style("Done").green().bold(),
            style(version).cyan()
        ));
        Ok(CheckStatus::Success {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    }
}
