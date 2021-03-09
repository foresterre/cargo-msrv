use crate::fetch::ToolchainSpecifier;
use rust_releases::RustReleasesError;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io;
use std::string::FromUtf8Error;

pub type TResult<T> = Result<T, CargoMSRVError>;

#[derive(Debug)]
pub enum CargoMSRVError {
    DefaultHostTripleNotFound,
    Env(env::VarError),
    GenericMessage(String),
    Io(io::Error),
    InvalidRustVersionNumber(std::num::ParseIntError),
    InvalidUTF8(FromUtf8Error),
    Log(log::ParseLevelError),
    RustupInstallFailed(ToolchainSpecifier),
    RustReleasesError(RustReleasesError),
    RustupRunWithCommandFailed,
    SystemTime(std::time::SystemTimeError),
    ToolchainNotInstalled,
    UnknownTarget,
    UnableToCacheChannelManifest,
    UnableToFindAnyGoodVersion { command: String },
    UnableToParseCliArgs,
    UnableToParseRustVersion,
    UnableToRunCheck,
}

impl fmt::Display for CargoMSRVError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CargoMSRVError::DefaultHostTripleNotFound => write!(f, "The default host triple (target) could not be found."),
            CargoMSRVError::Env(err) => err.fmt(f),
            CargoMSRVError::GenericMessage(msg) => write!(f, "{}", msg.as_str()),
            CargoMSRVError::Io(err) => err.fmt(f),
            CargoMSRVError::InvalidRustVersionNumber(err) => err.fmt(f),
            CargoMSRVError::InvalidUTF8(err) => err.fmt(f),
            CargoMSRVError::Log(err) => err.fmt(f),
            CargoMSRVError::RustupInstallFailed(toolchain) => f.write_fmt(format_args!("Unable to install toolchain with `rustup install {}`.", toolchain)),
            CargoMSRVError::RustReleasesError(err) => err.fmt(f),
            CargoMSRVError::RustupRunWithCommandFailed => write!(f, "Check toolchain (with `rustup run <toolchain> <command>`) failed."),
            CargoMSRVError::SystemTime(err) => err.fmt(f),
            CargoMSRVError::ToolchainNotInstalled => write!(f, "The given toolchain could not be found. Run `rustup toolchain list` for an overview of installed toolchains."),
            CargoMSRVError::UnknownTarget => write!(f, "The given target could not be found. Run `rustup target list` for an overview of available toolchains."),
            CargoMSRVError::UnableToCacheChannelManifest => write!(f, "Unable to get or store the channel manifest on disk."),
            CargoMSRVError::UnableToFindAnyGoodVersion { command } => write!(f, r#"Unable to find a Minimum Supported Rust Version (MSRV).

If you think this result is erroneous, please run: `{}` manually.

If the above does succeed, or you think cargo-msrv errored in another way, please feel free to
report the issue at: https://github.com/foresterre/cargo-msrv/issues

Thank you in advance!"#, command.as_str()),
            CargoMSRVError::UnableToParseCliArgs => write!(f, "Unable to parse the CLI arguments. Use `cargo msrv help` for more info."),
            CargoMSRVError::UnableToParseRustVersion => write!(f, "The Rust stable version could not be parsed from the stable channel manifest."),
            CargoMSRVError::UnableToRunCheck => write!(f, "Unable to run the checking command. If --check <cmd> is specified, you could try to verify if you can run the cmd manually." )
        }
    }
}

impl Error for CargoMSRVError {}

impl From<String> for CargoMSRVError {
    fn from(msg: String) -> Self {
        CargoMSRVError::GenericMessage(msg)
    }
}

impl From<env::VarError> for CargoMSRVError {
    fn from(err: env::VarError) -> Self {
        CargoMSRVError::Env(err)
    }
}

impl From<io::Error> for CargoMSRVError {
    fn from(err: io::Error) -> Self {
        CargoMSRVError::Io(err)
    }
}

impl From<FromUtf8Error> for CargoMSRVError {
    fn from(err: FromUtf8Error) -> Self {
        CargoMSRVError::InvalidUTF8(err)
    }
}

impl From<std::num::ParseIntError> for CargoMSRVError {
    fn from(err: std::num::ParseIntError) -> Self {
        CargoMSRVError::InvalidRustVersionNumber(err)
    }
}

impl From<log::ParseLevelError> for CargoMSRVError {
    fn from(err: log::ParseLevelError) -> Self {
        CargoMSRVError::Log(err)
    }
}

impl From<std::time::SystemTimeError> for CargoMSRVError {
    fn from(err: std::time::SystemTimeError) -> Self {
        CargoMSRVError::SystemTime(err)
    }
}

impl From<rust_releases::RustReleasesError> for CargoMSRVError {
    fn from(err: rust_releases::RustReleasesError) -> Self {
        CargoMSRVError::RustReleasesError(err)
    }
}
