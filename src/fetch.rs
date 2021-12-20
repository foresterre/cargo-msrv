use std::ffi::OsString;

use crate::{
    command::RustupCommand,
    errors::{CargoMSRVError, TResult},
};

pub type ToolchainSpecifier = String;

/// Check if the given target is available.
/// with `rustup target list`
pub fn is_target_available<S: AsRef<str>>(name: S) -> TResult<()> {
    let toolchain = name.as_ref();
    let output = RustupCommand::new()
        .with_stdout()
        .with_args(&["list"])
        .execute(OsString::from("target"))?;

    let stdout = output.stdout();

    // Each target is listed on a single line.
    // If a target is installed, it is listed as <target> (installed).
    // If a target is the default, it is listed as <target> (default).
    for line in stdout.lines() {
        if let Some(it) = line.split_ascii_whitespace().next() {
            if it == toolchain {
                return Ok(());
            }
        }
    }

    Err(CargoMSRVError::UnknownTarget)
}

/// Uses the `.rustup/settings.toml` file to determine the default target (aka
/// the `default_host_triple`) if not set by a user.
pub fn default_target() -> TResult<String> {
    let output = RustupCommand::new().with_stdout().show()?;

    let stdout = output.stdout();

    stdout
        .lines()
        .next()
        .ok_or(CargoMSRVError::DefaultHostTripleNotFound)
        .and_then(|line| {
            line.split_ascii_whitespace()
                .nth(2)
                .ok_or(CargoMSRVError::DefaultHostTripleNotFound)
                .map(String::from)
        })
}
