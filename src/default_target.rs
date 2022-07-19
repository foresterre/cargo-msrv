use crate::command::RustupCommand;
use crate::error::{CargoMSRVError, TResult};

/// Uses the `.rustup/settings.toml` file to determine the default target (aka the
/// `default_host_triple`) if not set by a user.
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
