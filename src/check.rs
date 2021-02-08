use crate::command::command;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::RustStableVersion;
use std::path::Path;

pub enum CheckStatus {
    Success {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: RustStableVersion,
    },
    Failure {
        // toolchain specifier
        toolchain: String,
        // checked Rust version
        version: RustStableVersion,
    },
}

pub fn check_toolchain(version: RustStableVersion, config: &CmdMatches) -> TResult<CheckStatus> {
    let toolchain_specifier = version.as_toolchain_string(config.target());

    download_if_required(&toolchain_specifier)?;
    try_building(
        version,
        &toolchain_specifier,
        config.seek_path(),
        config.check_command(),
    )
}

fn download_if_required(toolchain_specifier: &str) -> TResult<()> {
    let mut child = command(
        &["install", "--profile", "minimal", toolchain_specifier],
        None,
    )?;

    info!(
        "attempting to install or locate toolchain '{}'",
        toolchain_specifier
    );

    let status = child.wait()?;

    if !status.success() {
        return Err(CargoMSRVError::RustupInstallFailed(
            toolchain_specifier.to_string(),
        ));
    }

    Ok(())
}

fn try_building(
    version: RustStableVersion,
    toolchain_specifier: &str,
    dir: Option<&Path>,
    check: &[&str],
) -> TResult<CheckStatus> {
    let mut cmd: Vec<&str> = vec!["run", toolchain_specifier];
    cmd.extend_from_slice(check);

    let mut child = command(&cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;
    info!("checking crate against toolchain '{}'", toolchain_specifier);
    let status = child.wait()?;

    if !status.success() {
        info!("check '{}' failed", cmd.join(" "));
        Ok(CheckStatus::Failure {
            version,
            toolchain: toolchain_specifier.to_string(),
        })
    } else {
        info!("check '{}' succeeded", cmd.join(" "));
        Ok(CheckStatus::Success {
            version,
            toolchain: toolchain_specifier.to_string(),
        })
    }
}
