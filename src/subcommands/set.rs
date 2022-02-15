use crate::errors::IoErrorSource;
use crate::manifest::{CargoManifestParser, TomlParser};
use crate::paths::crate_root_folder;
use crate::{CargoMSRVError, Config, ModeIntent, Output, TResult};
use rust_releases::semver;
use std::io::Write;
use toml_edit::{value, Document, Item};

const RUST_VERSION_SUPPORTED_SINCE: semver::Version = semver::Version::new(1, 56, 0);

pub fn run_set_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::Show);

    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.clone()),
    })?;

    let mut manifest = CargoManifestParser::default().parse::<Document>(&contents)?;

    let msrv = &config.sub_command_config().set().msrv;

    remove_minimum_rust_version(&mut manifest);

    if msrv.to_semver_version() >= RUST_VERSION_SUPPORTED_SINCE {
        manifest["package"]["rust-version"] = value(msrv.to_string());
    } else {
        manifest["package"]["metadata"]["msrv"] = value(msrv.to_string());
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&cargo_toml)
        .map_err(|error| CargoMSRVError::Io {
            error,
            source: IoErrorSource::OpenFile(cargo_toml.clone()),
        })?;

    write!(&mut file, "{}", manifest).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::WriteFile(cargo_toml.clone()),
    })?;

    output.finish_success(ModeIntent::Set, None);

    Ok(())
}

/// Removes the minimum supported Rust version (MSRV) from `Cargo.toml` manifest, if it exists
fn remove_minimum_rust_version(document: &mut Document) {
    /// Removes the `MSRV` as supported by Cargo since Rust 1.56.0
    ///
    /// [`Cargo`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field
    fn remove_rust_version(document: &mut Document) {
        document
            .as_table_mut()
            .get_mut("package")
            .and_then(Item::as_table_mut)
            .and_then(|package| package.remove("rust-version"));
    }

    /// Removes the MSRV as supported by `cargo-msrv`, since prior to the release of Rust
    /// 1.56.0
    fn remove_metadata_msrv(document: &mut Document) {
        document
            .as_table_mut()
            .get_mut("package")
            .and_then(Item::as_table_mut)
            .and_then(|package| package.get_mut("metadata"))
            .and_then(Item::as_table_mut)
            .and_then(|metadata| metadata.remove("msrv")); // TODO: does this work?
    }

    // First remove the rust-version
    remove_rust_version(document);

    // Then remove the metadata.msrv, if it exists
    remove_metadata_msrv(document);
}
