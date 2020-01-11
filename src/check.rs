use crate::command::command;
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
use crate::fetch::RustStableVersion;
use std::path::Path;

pub fn check_with_rust_version(version: &RustStableVersion, config: &CmdMatches) -> TResult<()> {
    let mut toolchain_specifier = version.as_string();
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
    let mut child = command(
        &["install", "--profile", "minimal", toolchain_specifier],
        None,
    )?;

    let status = child.wait()?;

    if !status.success() {
        return Err(CargoMSRVError::RustupInstallFailed(
            toolchain_specifier.to_string(),
        ));
    }

    Ok(())
}

fn try_building(toolchain_specifier: &str, dir: Option<&Path>, check: &[&str]) -> TResult<()> {
    let mut cmd: Vec<&str> = vec!["run", toolchain_specifier];
    cmd.extend_from_slice(check);

    let mut child = command(cmd, dir).map_err(|_| CargoMSRVError::UnableToRunCheck)?;
    let status = child.wait()?;

    if !status.success() {
        return Err(CargoMSRVError::RustupRunWithCommandFailed);
    }

    Ok(())
}
