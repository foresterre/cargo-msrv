use crate::command::command;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::RustStableVersion;
use std::path::Path;

pub fn check_with_rust_version(version: &RustStableVersion, config: &CmdMatches) -> TResult<()> {
    let mut toolchain_specifier = version.as_string().clone();
    toolchain_specifier.push('-');
    toolchain_specifier.push_str(config.target());

    download_if_required(&toolchain_specifier)?;
    try_building(
        &toolchain_specifier,
        config.seek_path(),
        config.custom_check(),
    )?;

    Ok(())
}

fn download_if_required(toolchain_specifier: &str) -> TResult<()> {
    let mut child = command(&["install", toolchain_specifier], None)?;
    let status = child.wait()?;

    if !status.success() {
        return Err(CargoMSRVError::RustupInstallFailed);
    }

    Ok(())
}

fn try_building(toolchain_specifier: &str, dir: Option<&Path>, check: &[&str]) -> TResult<()> {
    let mut cmd: Vec<&str> = Vec::new();
    cmd.push("run");
    cmd.push(toolchain_specifier);

    for element in check {
        cmd.push(element);
    }

    let mut child = command(cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;

    let status = child.wait()?;

    if !status.success() {
        return Err(CargoMSRVError::RustupRunWithCommandFailed);
    }

    Ok(())
}
