use std::path::Path;

use rust_releases::semver;

use crate::command::RustupCommand;
use crate::config::Config;
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::paths::crate_root_folder;
use crate::reporter::{Output, ProgressAction};

#[derive(Clone, Debug)]
pub struct Outcome {
    result: Status,
    // toolchain specifier
    toolchain: String,
    // checked Rust version
    version: semver::Version,
}

impl Outcome {
    pub(crate) fn is_success(&self) -> bool {
        match self.result {
            Status::Success => true,
            Status::Failure => false,
        }
    }

    pub(crate) fn version(&self) -> &semver::Version {
        &self.version
    }

    pub(crate) fn toolchain(&self) -> &str {
        &self.toolchain
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
    Success,
    Failure,
}

pub fn check_toolchain<'a>(
    version: &'a semver::Version,
    config: &'a Config,
    output: &'a impl Output,
) -> TResult<Outcome> {
    info!(ignore_lockfile_enabled = config.ignore_lockfile());

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
) -> TResult<Outcome> {
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

#[instrument(skip(output, toolchain_specifier, version))]
fn download_if_required(
    version: &semver::Version,
    toolchain_specifier: &str,
    output: &impl Output,
) -> TResult<()> {
    let toolchain = toolchain_specifier.to_owned();
    output.progress(ProgressAction::Installing(version));

    info!(
        toolchain = toolchain_specifier,
        version = format!("{}", version).as_str(),
        "installing toolchain"
    );

    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args(&["--profile", "minimal", &toolchain])
        .install()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain_specifier,
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to install toolchain"
        );

        return Err(CargoMSRVError::RustupInstallFailed(
            toolchain_specifier.to_string(),
        ));
    }

    Ok(())
}

fn remove_lockfile(config: &Config) -> TResult<()> {
    let lock_file = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;

    if lock_file.is_file() {
        std::fs::remove_file(&lock_file).map_err(|err| {
            CargoMSRVError::Io(
                err,
                IoErrorSource::RemoveFile {
                    path: lock_file.clone(),
                },
            )
        })?;
    }

    Ok(())
}

fn try_building(
    version: &semver::Version,
    toolchain_specifier: &str,
    dir: Option<&Path>,
    check: &[&str],
    output: &impl Output,
) -> TResult<Outcome> {
    let mut cmd: Vec<&str> = vec![toolchain_specifier];
    cmd.extend_from_slice(check);

    output.progress(ProgressAction::Checking(version));

    let rustup_output = RustupCommand::new()
        .with_args(cmd.iter())
        .with_optional_dir(dir)
        .with_stderr()
        .run()
        .map_err(|_| CargoMSRVError::UnableToRunCheck)?;

    let status = rustup_output.exit_status();

    output.complete_step(version, status.success());

    let toolchain = toolchain_specifier.to_owned();
    let version = version.clone();

    if status.success() {
        Ok(Outcome {
            result: Status::Success,
            toolchain,
            version,
        })
    } else {
        let stderr = rustup_output.stderr();
        let command = cmd.join(" ");

        info!(
            toolchain = toolchain_specifier,
            stderr,
            cmd = command.as_str(),
            "try_building run failed"
        );

        Ok(Outcome {
            result: Status::Failure,
            toolchain,
            version,
        })
    }
}
