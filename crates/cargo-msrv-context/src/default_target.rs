use crate::context::error::{Error, IoError, IoErrorSource, TResult};
use std::ffi::OsStr;
use std::process::{Command, Stdio};

/// Uses the `.rustup/settings.toml` file to determine the default target (aka the
/// `default_host_triple`) if not set by a user.
pub fn default_target() -> TResult<String> {
    let stdout = rustup_show()?;

    stdout
        .lines()
        .next()
        .ok_or(Error::DefaultHostTripleNotFound)
        .and_then(|line| {
            line.split_ascii_whitespace()
                .nth(2)
                .ok_or(Error::DefaultHostTripleNotFound)
                .map(String::from)
        })
}

fn rustup_show() -> TResult<String> {
    let cmd = OsStr::new("show");

    let child = Command::new("rustup")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| IoError {
            error,
            source: IoErrorSource::SpawnProcess(cmd.to_owned()),
        })?;

    let output = child.wait_with_output().map_err(|error| IoError {
        error,
        source: IoErrorSource::WaitForProcessAndCollectOutput(cmd.to_owned()),
    })?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
