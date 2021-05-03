use crate::command::{command, command_with_output};
use crate::config::CmdMatches;
use crate::errors::{CargoMSRVError, TResult};
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
    config: &'a CmdMatches,
    ui: &'a Printer,
) -> TResult<CheckStatus> {
    let toolchain_specifier = as_toolchain_specifier(version, config.target());

    download_if_required(version, &toolchain_specifier, ui)?;
    try_building(
        version,
        &toolchain_specifier,
        config.crate_path(),
        config.check_command(),
        ui,
    )
}

pub fn as_toolchain_specifier(version: &semver::Version, target: &str) -> String {
    format!("{}-{}", version, target)
}

fn download_if_required(
    version: &semver::Version,
    toolchain_specifier: &str,
    ui: &Printer,
) -> TResult<()> {
    let toolchain = toolchain_specifier.to_owned();
    ui.show_progress("Installing", version);

    let status = command_with_output(&["install", "--profile", "minimal", &toolchain])
        .and_then(|c| c.wait_with_output().map_err(CargoMSRVError::Io))?;

    let out = String::from_utf8_lossy(&status.stdout);
    let err = String::from_utf8_lossy(&status.stderr);

    println!("----");
    println!("DEBUG-start rustup stdout");
    println!("----");
    dbg!(out);
    println!("DEBUG-end rustup stdout");
    println!("DEBUG-start rustup stderr");
    dbg!(err);
    println!("DEBUG-end rustup stderr");
    println!("----");
    println!("----");

    if !status.status.success() {
        return Err(CargoMSRVError::RustupInstallFailed(
            toolchain_specifier.to_string(),
        ));
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
        ui.complete_step(
            format!(
                "{} Bad check for {}",
                style("Done").green().bold(),
                style(version).cyan()
            )
            .as_str(),
        );

        Ok(CheckStatus::Failure {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    } else {
        ui.complete_step(
            format!(
                "{} Good check for {}",
                style("Done").green().bold(),
                style(version).cyan()
            )
            .as_str(),
        );
        Ok(CheckStatus::Success {
            toolchain: toolchain_specifier.to_string(),
            version: version.clone(),
        })
    }
}
