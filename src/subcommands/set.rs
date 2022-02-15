use crate::errors::IoErrorSource;
use crate::manifest::bare_version::BareVersion;
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

    set_msrv(&mut manifest, msrv);

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

fn set_msrv(manifest: &mut Document, msrv: &BareVersion) {
    discard_current_msrv(manifest);
    insert_new_msrv(manifest, msrv);
}

fn insert_new_msrv(manifest: &mut Document, msrv: &BareVersion) {
    if msrv.to_semver_version() >= RUST_VERSION_SUPPORTED_SINCE {
        manifest["package"]["rust-version"] = value(msrv.to_string());
    } else {
        manifest["package"]["metadata"]["msrv"] = value(msrv.to_string());
    }
}

/// Removes the minimum supported Rust version (MSRV) from `Cargo.toml` manifest, if it exists
fn discard_current_msrv(document: &mut Document) {
    fn get_package(document: &mut Document) -> Option<&mut Item> {
        document.as_table_mut().get_mut("package")
    }

    fn get_metadata(document: &Document) -> Option<&Item> {
        document
            .as_table()
            .get("package")
            .and_then(|package| package.get("metadata"))
    }

    /// Removes the `MSRV` as supported by Cargo since Rust 1.56.0
    ///
    /// [`Cargo`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field
    fn remove_rust_version(document: &mut Document) {
        get_package(document)
            .and_then(Item::as_table_like_mut)
            .and_then(|package| package.remove("rust-version"));
    }

    /// Removes the MSRV as supported by `cargo-msrv`, since prior to the release of Rust
    /// 1.56.0
    fn remove_metadata_msrv(document: &mut Document) {
        get_package(document)
            .and_then(|package| package.get_mut("metadata"))
            .and_then(Item::as_table_like_mut)
            .and_then(|metadata| metadata.remove("msrv"));

        // remove residual metadata table if now empty
        if let Some(true) = get_metadata(document)
            .and_then(Item::as_table_like)
            .map(|metadata| metadata.is_empty())
        {
            get_package(document)
                .and_then(Item::as_table_like_mut)
                .map(|package| package.remove("metadata"));
        }
    }

    // First remove the rust-version
    remove_rust_version(document);

    // Then remove the metadata.msrv, if it exists
    remove_metadata_msrv(document);
}

#[cfg(test)]
mod set_msrv_tests {
    use crate::manifest::bare_version::BareVersion;
    use crate::manifest::{CargoManifestParser, TomlParser};
    use crate::subcommands::set::set_msrv;
    use toml_edit::Document;

    #[test]
    fn set_rust_version_in_empty_two_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56"
        );
    }

    #[test]
    fn set_metadata_msrv_in_empty_two_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 10));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.10"
        );
    }

    #[test]
    fn set_rust_version_with_existing() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
rust-version = "1.58.0"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56"
        );
    }

    #[test]
    fn set_rust_version_with_existing_metadata_msrv() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.58.0"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56"
        );

        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata");

        assert!(metadata.is_none());
    }

    #[test]
    fn set_rust_version_with_existing_metadata_msrv_and_other() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.58.0"
other = 1

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56"
        );

        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata").unwrap();
        let msrv = metadata.get("msrv");

        assert!(msrv.is_none());

        let other = metadata.get("other");

        assert!(other.is_some());
    }

    #[test]
    fn set_metadata_msrv_with_existing() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.11.0"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.11.0"
        );

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.17"
        );
    }

    #[test]
    fn set_metadata_msrv_with_existing_rust_version() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
rust-version = "1.58"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.58"
        );

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.17"
        );

        assert!(manifest
            .get("package")
            .and_then(|p| p.get("rust-version"))
            .is_none());
    }

    #[test]
    fn set_metadata_msrv_with_existing_and_other() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.58"
other = 1

[dependencies]

"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.58"
        );

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.17"
        );

        assert_eq!(
            manifest["package"]["metadata"]["other"]
                .as_integer()
                .unwrap(),
            1
        );
    }

    #[test]
    fn set_rust_version_with_inline_table() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
metadata = { msrv = "1.15" }

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.15"
        );

        set_msrv(&mut manifest, &BareVersion::TwoComponents(1, 57));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.57"
        );

        assert!(manifest
            .get("package")
            .and_then(|p| p.get("metadata"))
            .is_none());
    }
}

#[cfg(test)]
mod discard_current_msrv_tests {
    use crate::manifest::{CargoManifestParser, TomlParser};
    use crate::subcommands::set::discard_current_msrv;
    use toml_edit::Document;

    #[test]
    fn discard_none() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        let expected = manifest.clone();

        discard_current_msrv(&mut manifest);

        assert_eq!(manifest.to_string(), expected.to_string());
    }

    #[test]
    fn discard_rust_version() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
rust-version = "1.56.0"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        // pre
        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56.0"
        );

        discard_current_msrv(&mut manifest);

        let package = manifest.get("package").unwrap();
        let rust_version = package.get("rust-version");

        assert!(rust_version.is_none());
    }

    #[test]
    fn discard_package_metadata_msrv_with_empty_metadata() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.56.0"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        // pre
        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.56.0"
        );

        // perform action
        discard_current_msrv(&mut manifest);

        // check result
        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata");

        // The metadata table has been completely emptied
        assert!(metadata.is_none());
    }

    #[test]
    fn discard_package_metadata_msrv_with_other_metadata_item() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.56.0"
other = 7

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        // pre
        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.56.0"
        );

        // perform action
        discard_current_msrv(&mut manifest);

        // check result
        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata").unwrap();
        let msrv = metadata.get("msrv");

        // The MSRV has been removed
        assert!(msrv.is_none());

        // .. but the other field is still there
        assert_eq!(
            manifest["package"]["metadata"]["other"]
                .as_integer()
                .unwrap(),
            7,
        );
    }

    #[test]
    fn discard_package_metadata_inline_table() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
metadata = { msrv = "1.15" }

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        // pre
        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.15"
        );

        // perform action
        discard_current_msrv(&mut manifest);

        // check result
        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata");

        assert!(metadata.is_none());
    }

    #[test]
    fn discard_package_metadata_inline_table_and_other() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
metadata = { msrv = "1.15", other = 1 }

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        // pre
        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.15"
        );

        // perform action
        discard_current_msrv(&mut manifest);

        // check result
        let package = manifest.get("package").unwrap();
        let metadata = package.get("metadata").unwrap();

        let msrv = metadata.get("msrv");
        assert!(msrv.is_none());

        let other = metadata.get("other");
        assert!(other.is_some());
    }
}

#[cfg(test)]
mod insert_new_msrv_tests {
    use crate::manifest::bare_version::BareVersion;
    use crate::manifest::{CargoManifestParser, TomlParser};
    use crate::subcommands::set::insert_new_msrv;
    use toml_edit::Document;

    #[test]
    fn insert_rust_version_in_empty_two_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        insert_new_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56"
        );
    }

    #[test]
    fn insert_rust_version_in_empty_three_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        insert_new_msrv(&mut manifest, &BareVersion::ThreeComponents(1, 56, 1));

        assert_eq!(
            manifest["package"]["rust-version"].as_str().unwrap(),
            "1.56.1"
        );
    }

    #[test]
    fn insert_metadata_msrv_in_empty_two_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        insert_new_msrv(&mut manifest, &BareVersion::TwoComponents(1, 10));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.10"
        );
    }

    #[test]
    fn insert_metadata_msrv_in_empty_three_component() {
        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        insert_new_msrv(&mut manifest, &BareVersion::ThreeComponents(1, 10, 1));

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.10.1"
        );
    }
}
