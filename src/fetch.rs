use crate::command::command_with_output;
use crate::errors::{CargoMSRVError, TResult};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
                            .map(String::from)
                    })
            })
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustStableVersion {
    major: u16,
    minor: u16,
    patch: u16, // FIXME: since we currently just decrease the minor version, we require no patch version as it would break if the previous patch version is smaller than the current
}

impl RustStableVersion {
    pub fn new(major: u16, minor: u16, _patch: u16) -> Self {
        Self {
            major,
            minor,
            // FIXME: since we currently just decrease the minor version, we require no patch version as it would break if the previous patch version is smaller than the current
            // one way to solve this is by building a version cache first
            patch: 0,
        }
    }

    pub fn from_parts(parts: &[&str]) -> TResult<Self> {
        let major = parts
            .get(0)
            .ok_or(CargoMSRVError::UnableToParseRustVersion)?;
        let major = major.parse::<u16>()?;

        let minor = parts
            .get(1)
            .ok_or(CargoMSRVError::UnableToParseRustVersion)?;
        let minor = minor.parse::<u16>()?;

        let patch = parts
            .get(2)
            .ok_or(CargoMSRVError::UnableToParseRustVersion)?;
        let patch = patch.parse::<u16>()?;

        Ok(RustStableVersion::new(major, minor, patch))
    }

    pub fn major(&self) -> u16 {
        self.major
    }

    pub fn minor(&self) -> u16 {
        self.minor
    }

    pub fn patch(&self) -> u16 {
        self.patch
    }

    pub fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailableVersionPool {
    Exhausted,
    Ok(RustStableVersion),
}

const CHANNEL_MANIFEST_STABLE: &str = "https://static.rust-lang.org/dist/channel-rust-stable.toml";

pub fn latest_stable_version() -> TResult<RustStableVersion> {
    let manifest: (ManifestObtainedFrom, PathBuf) = get_stable_channel_manifest()?;

    parse_rust_version_from_stable_channel_manifest(&manifest.1)
}

#[derive(Debug)]
pub enum ManifestObtainedFrom {
    Cache,
    RustDistChannel,
}

/// Cached files are considered stale after one day
const STALENESS_TIMEOUT: Duration = Duration::from_secs(86_400);

/// Checks if the manifest file in the cache is stale.
/// A stale file should be redownloaded to get the latest changes.
fn is_stale<P: AsRef<Path>>(manifest: P) -> TResult<bool> {
    let metadata = fs::metadata(manifest)?;
    let modification = metadata.modified()?;

    let duration = modification.elapsed()?;

    Ok(STALENESS_TIMEOUT < duration)
}

/// Obtains the release channel manifest.
/// If the document doesn't exist in the cache, it is downloaded from the rust dist server
/// and cached locally on the client.
/// The path to the save location is returned on an Ok result.
fn get_stable_channel_manifest() -> TResult<(ManifestObtainedFrom, PathBuf)> {
    let cache = directories::ProjectDirs::from("com", "ilumeo", "cargo-msrv")
        .ok_or(CargoMSRVError::UnableToCacheChannelManifest)?;
    let cache = cache.cache_dir();
    let manifest = cache.join("channel-rust-stable.toml");

    if manifest.as_path().exists() && !is_stale(&manifest)? {
        return Ok((ManifestObtainedFrom::Cache, manifest));
    } else {
        std::fs::create_dir_all(cache)?;
    }

    let mut response = reqwest::get(CHANNEL_MANIFEST_STABLE)?;
    let mut file = File::create(manifest.as_path())?;
    response.copy_to(&mut file)?;

    Ok((ManifestObtainedFrom::RustDistChannel, manifest))
}

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    pkg: Pkg,
}
#[derive(Deserialize)]
struct Pkg {
    rust: Rust,
}

#[derive(Deserialize)]
struct Rust {
    version: String,
}

fn parse_rust_version_from_stable_channel_manifest<P: AsRef<Path>>(
    manifest_path: P,
) -> TResult<RustStableVersion> {
    let mut reader = BufReader::new(File::open(manifest_path)?);
    let mut buffer: Vec<u8> = Vec::new();
    let _ = reader.read_to_end(&mut buffer)?;
    let parsed: Manifest = toml::from_slice(&buffer)?;

    let version = parsed
        .pkg
        .rust
        .version
        .split_ascii_whitespace()
        .next()
        .ok_or(CargoMSRVError::UnableToParseRustVersion)?;

    let parts: Vec<&str> = version.split('.').collect::<Vec<_>>();

    RustStableVersion::from_parts(&parts)
}
