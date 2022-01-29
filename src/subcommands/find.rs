use rust_releases::{semver, Release, ReleaseIndex};

use crate::check::RunCheck;
use crate::config::{Config, ModeIntent, SearchMethod};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::paths::crate_root_folder;
use crate::releases::filter_releases;
use crate::reporter::Output;
use crate::result::MinimalCompatibility;
use crate::search_methods::{Bisect, FindMinimalCapableToolchain, Linear};

pub fn run_find_msrv_action<R: Output>(
    config: &Config,
    reporter: &R,
    release_index: &ReleaseIndex,
) -> TResult<()> {
    match find_msrv(config, reporter, release_index)? {
        MinimalCompatibility::NoCompatibleToolchains => {
            info!("no minimal-compatible toolchain found");

            Err(CargoMSRVError::UnableToFindAnyGoodVersion {
                command: config.check_command().join(" "),
            })
        }
        MinimalCompatibility::CapableToolchain {
            toolchain,
            ref version,
            last_error: _,
        } if config.output_toolchain_file() => {
            let version_formatted = version.to_string();
            info!(
                toolchain = toolchain.as_str(),
                version = version_formatted.as_str(),
                "found minimal-compatible toolchain"
            );

            output_toolchain_file(config, version)
        }
        MinimalCompatibility::CapableToolchain {
            toolchain,
            version,
            last_error: _,
        } => {
            let version = version.to_string();

            info!(
                toolchain = toolchain.as_str(),
                version = version.as_str(),
                "found minimal-compatible toolchain"
            );

            Ok(())
        }
    }
}

pub fn find_msrv<R: Output>(
    config: &Config,
    reporter: &R,
    index: &rust_releases::ReleaseIndex,
) -> TResult<MinimalCompatibility> {
    let releases = index.releases();
    let included_releases = filter_releases(config, releases);

    reporter.mode(ModeIntent::Find);
    reporter.set_steps(included_releases.len() as u64);
    run_with_search_method(config, &included_releases, reporter)
}

fn run_with_search_method(
    config: &Config,
    included_releases: &[Release],
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    output.set_steps(included_releases.len() as u64);

    let search_method = config.search_method();
    info!(?search_method);

    let runner = RunCheck::new(output);

    // Run a linear or binary search depending on the configuration
    match search_method {
        SearchMethod::Linear => {
            run_searcher(Linear::new(runner), included_releases, config, output)
        }
        SearchMethod::Bisect => {
            run_searcher(Bisect::new(runner), included_releases, config, output)
        }
    }
}

fn run_searcher(
    method: impl FindMinimalCapableToolchain,
    releases: &[Release],
    config: &Config,
    output: &impl Output,
) -> TResult<MinimalCompatibility> {
    let minimum_capable = method.find_toolchain(releases, config, output)?;

    let cmd = config.check_command_string();
    report_outcome(&minimum_capable, &cmd, output);

    Ok(minimum_capable)
}

fn report_outcome(minimum_capable: &MinimalCompatibility, cmd: &str, output: &impl Output) {
    match minimum_capable {
        MinimalCompatibility::CapableToolchain {
            toolchain: _,
            version,
            last_error: _,
        } => {
            output.finish_success(ModeIntent::Find, Some(version));
        }
        MinimalCompatibility::NoCompatibleToolchains => {
            output.finish_failure(ModeIntent::Find, Some(cmd));
        }
    }
}

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

fn output_toolchain_file(config: &Config, stable_version: &semver::Version) -> TResult<()> {
    let path_prefix = crate_root_folder(config)?;

    // check if the rust-toolchain(.toml) file already exists
    if path_prefix.join(TOOLCHAIN_FILE).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE
        );
        return Ok(());
    } else if path_prefix.join(TOOLCHAIN_FILE_TOML).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE_TOML
        );
        return Ok(());
    }

    let path = path_prefix.join(TOOLCHAIN_FILE);
    let content = format!(
        r#"[toolchain]
channel = "{}"
"#,
        stable_version
    );

    std::fs::write(&path, content).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::WriteFile(path.clone()),
    })?;
    eprintln!("Written toolchain file to '{}'", &path.display());

    Ok(())
}
