use crate::command::command_with_output;
use crate::errors::{CargoMSRVError, TResult};

pub type ToolchainSpecifier = String;

/// Verify that the given toolchain is installed.
/// with `rustup toolchain list`
pub fn is_toolchain_installed<S: AsRef<str>>(name: S) -> TResult<()> {
    let toolchain = name.as_ref();
    command_with_output(&["toolchain", "list"]).and_then(|child| {
        let output = child.wait_with_output()?;

        String::from_utf8(output.stdout)
            .map_err(From::from)
            .and_then(|string| {
                let mut lines = string.lines();

                // the default toolchain is formatted like so:
                // <toolchain> (default)
                if let Some(first) = lines.next() {
                    if let Some(default) = first.split_ascii_whitespace().next() {
                        if default == toolchain {
                            return Ok(());
                        }
                    }
                }

                // after the default toolchain, all other installed toolchains are listed
                // one per line
                for line in lines {
                    if line == toolchain {
                        return Ok(());
                    }
                }

                Err(CargoMSRVError::ToolchainNotInstalled)
            })
    })
}

/// Check if the given target is available.
/// with `rustup target list`
pub fn is_target_available<S: AsRef<str>>(name: S) -> TResult<()> {
    let toolchain = name.as_ref();
    command_with_output(&["target", "list"]).and_then(|child| {
        let output = child.wait_with_output()?;

        String::from_utf8(output.stdout)
            .map_err(From::from)
            .and_then(|string| {
                // Each target is listed on a single line.
                // If a target is installed, it is listed as <target> (installed).
                // If a target is the default, it is listed as <target> (default).
                for line in string.lines() {
                    if let Some(it) = line.split_ascii_whitespace().next() {
                        if it == toolchain {
                            return Ok(());
                        }
                    }
                }

                Err(CargoMSRVError::UnknownTarget)
            })
    })
}

/// Uses the `.rustup/settings.toml` file to determine the default target (aka the
/// `default_host_triple`) if not set by a user.
pub fn default_target() -> TResult<String> {
    command_with_output(&["show"]).and_then(|child| {
        let output = child.wait_with_output()?;

        String::from_utf8(output.stdout)
            .map_err(From::from)
            .and_then(|string| {
                // the first line contains the default target
                // e.g. `Default host: x86_64-unknown-linux-gnu`

                string
                    .lines()
                    .next()
                    .ok_or(CargoMSRVError::DefaultHostTripleNotFound)
                    .and_then(|line| {
                        line.split_ascii_whitespace()
                            .nth(2)
                            .ok_or(CargoMSRVError::DefaultHostTripleNotFound)
                            .map(|target| {
                                info!("default target determined to be '{}'", target);

                                String::from(target)
                            })
                    })
            })
    })
}
