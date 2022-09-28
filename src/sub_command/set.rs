use std::io::Write;

use rust_releases::semver;

use toml_edit::{table, value, Document, Item, Value};

use crate::error::{IoErrorSource, SetMsrvError};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifestParser, TomlParser};
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, Destination, MsrvKind, SetResult,
};
use crate::reporter::Reporter;
use crate::{CargoMSRVError, Config, SubCommand, TResult};

const RUST_VERSION_SUPPORTED_SINCE: semver::Version = semver::Version::new(1, 56, 0);

#[derive(Default)]
pub struct Set;

impl SubCommand for Set {
    type Output = ();

    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<Self::Output> {
        set_msrv(config, reporter)
    }
}

fn set_msrv(config: &Config, reporter: &impl Reporter) -> TResult<()> {
    let cargo_toml = config.context().manifest_path()?;

    // Read the Cargo manifest to a String
    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.to_path_buf()),
    })?;

    // Parse the Cargo manifest contents, in particular the MSRV value
    let mut manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    check_workspace(&manifest)?;
    let msrv = &config.sub_command_config().set().msrv;

    // Set the MSRV
    set_or_override_msrv(&mut manifest, msrv)?;

    // Open the Cargo manifest file with write permissions and truncate the current its contents
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&cargo_toml)
        .map_err(|error| CargoMSRVError::Io {
            error,
            source: IoErrorSource::OpenFile(cargo_toml.to_path_buf()),
        })?;

    // Write the new manifest contents with the newly set MSRV value
    write!(&mut file, "{}", manifest).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::WriteFile(cargo_toml.to_path_buf()),
    })?;

    reporter.report_event(AuxiliaryOutput::new(
        Destination::file(cargo_toml.to_path_buf()),
        AuxiliaryOutputItem::msrv(MsrvKind::RustVersion),
    ))?;

    // Report that the MSRV was set
    reporter.report_event(SetResult::new(msrv.clone(), cargo_toml.to_path_buf()))?;

    Ok(())
}

fn check_workspace(manifest: &Document) -> TResult<()> {
    if manifest.as_table().get("package").is_none()
        && manifest.as_table().get("workspace").is_some()
    {
        Err(CargoMSRVError::WorkspaceFound)
    } else {
        Ok(())
    }
}

/// Override MSRV if it is already set, otherwise, simply set it
fn set_or_override_msrv(manifest: &mut Document, msrv: &BareVersion) -> TResult<()> {
    // NB: As a consequence of scrubbing the current MSRV, if the MSRV is the only value in the
    //     [package.metadata] table, and the table is an inline table, then the inline table will
    //     be removed and replaced with a regular table (normally we try to keep the same table type
    //     if a table already existed).
    //
    //     In a future refactor, we may want to handle all cases of modifying the manifest instead
    //     of discarding the current MSRV, considering the following cases:
    //     * MSRV is not set yet, or is
    //     * if set: is set as package.rust-version, as package.metadata.msrv or both
    //     * new MSRV is below package.rust-version Cargo support threshold, or above
    discard_current_msrv(manifest);
    insert_new_msrv(manifest, msrv)
}

fn insert_new_msrv(manifest: &mut Document, msrv: &BareVersion) -> TResult<()> {
    fn insert_rust_version(manifest: &mut Document, msrv: &BareVersion) -> TResult<()> {
        manifest["package"]["rust-version"] = value(msrv.to_string());
        Ok(())
    }

    fn insert_package_metadata_msrv(manifest: &mut Document, msrv: &BareVersion) -> TResult<()> {
        let metadata_item = &mut manifest["package"]["metadata"];

        match metadata_item {
            Item::None => {
                // Explicitly create the table, otherwise it would default to an inline table instead
                *metadata_item = table();
                metadata_item["msrv"] = value(msrv.to_string());
            }
            Item::Value(Value::InlineTable(table)) => {
                // keep the inline table if it already exists
                table.insert("msrv", msrv.to_string().into());
            }
            Item::Table(table) => {
                table.insert("msrv", value(msrv.to_string()));
            }
            _ => return Err(CargoMSRVError::SetMsrv(SetMsrvError::NotATable)),
        }

        Ok(())
    }

    if msrv.to_semver_version() >= RUST_VERSION_SUPPORTED_SINCE {
        insert_rust_version(manifest, msrv)
    } else {
        insert_package_metadata_msrv(manifest, msrv)
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
mod set_or_override_msrv_tests {
    use toml_edit::Document;

    use crate::manifest::bare_version::BareVersion;
    use crate::manifest::{CargoManifestParser, TomlParser};
    use crate::sub_command::set::set_or_override_msrv;

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 10)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 17)).unwrap();

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

        set_or_override_msrv(&mut manifest, &BareVersion::TwoComponents(1, 57)).unwrap();

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
    use toml_edit::Document;

    use crate::manifest::{CargoManifestParser, TomlParser};
    use crate::sub_command::set::discard_current_msrv;

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
    use std::convert::TryInto;

    use toml_edit::Document;

    use crate::manifest::bare_version::BareVersion;
    use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
    use crate::sub_command::set::insert_new_msrv;

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

        insert_new_msrv(&mut manifest, &BareVersion::TwoComponents(1, 56)).unwrap();

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

        insert_new_msrv(&mut manifest, &BareVersion::ThreeComponents(1, 56, 1)).unwrap();

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

        insert_new_msrv(&mut manifest, &BareVersion::TwoComponents(1, 10)).unwrap();

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

        insert_new_msrv(&mut manifest, &BareVersion::ThreeComponents(1, 10, 1)).unwrap();

        assert_eq!(
            manifest["package"]["metadata"]["msrv"].as_str().unwrap(),
            "1.10.1"
        );
    }

    // In this module we check whether the correct formatting is used, i.e. whether the correct
    // TOML items are used, such as inline tables and regular tables.
    // Only applicable to the [package.manifest] msrv = "..." fallback variant MSRV
    mod insert_package_manifest_msrv_correct_table_type {
        use toml_edit::{Document, Item, Value};

        use crate::manifest::bare_version::BareVersion;
        use crate::manifest::{CargoManifestParser, TomlParser};
        use crate::sub_command::set::insert_new_msrv;

        const METADATA_MSRV: BareVersion = BareVersion::TwoComponents(1, 55);

        #[test]
        fn insert_without_preexisting_table() {
            let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

            let mut manifest = CargoManifestParser::default()
                .parse::<Document>(input)
                .unwrap();

            insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

            let metadata = &manifest["package"]["metadata"];
            assert!(matches!(metadata, Item::Table(_)));

            let msrv = &metadata["msrv"];
            assert!(matches!(msrv, Item::Value(Value::String(s)) if s.value() == "1.55"))
        }

        #[test]
        fn insert_with_preexisting_rust_version() {
            let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
rust-version = "1.56"

[dependencies]
"#;

            let mut manifest = CargoManifestParser::default()
                .parse::<Document>(input)
                .unwrap();

            insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

            let metadata = &manifest["package"]["metadata"];
            assert!(matches!(metadata, Item::Table(_)));

            let msrv = &metadata["msrv"];
            assert!(matches!(msrv, Item::Value(Value::String(s)) if s.value() == "1.55"))
        }

        #[test]
        fn insert_with_preexisting_package_metadata_msrv_regular_table() {
            let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[package.metadata]
msrv = "1.54"

[dependencies]
"#;

            let mut manifest = CargoManifestParser::default()
                .parse::<Document>(input)
                .unwrap();

            insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

            let metadata = &manifest["package"]["metadata"];
            assert!(matches!(metadata, Item::Table(_)));

            let msrv = &metadata["msrv"];
            assert!(matches!(msrv, Item::Value(Value::String(s)) if s.value() == "1.55"))
        }

        #[test]
        fn insert_with_preexisting_package_metadata_msrv_inline_table() {
            let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
metadata = { msrv = "1.54" }

[dependencies]
"#;

            let mut manifest = CargoManifestParser::default()
                .parse::<Document>(input)
                .unwrap();

            insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

            let metadata = &manifest["package"]["metadata"];
            assert!(matches!(metadata, Item::Value(Value::InlineTable(_))));

            let msrv = &metadata["msrv"];
            assert!(matches!(msrv, Item::Value(Value::String(s)) if s.value() == "1.55"))
        }

        #[test]
        fn insert_with_preexisting_package_metadata_but_no_msrv() {
            let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"
metadata = { k = "1.54" }

[dependencies]
"#;

            let mut manifest = CargoManifestParser::default()
                .parse::<Document>(input)
                .unwrap();

            insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

            let metadata = &manifest["package"]["metadata"];
            assert!(matches!(metadata, Item::Value(Value::InlineTable(_))));

            let msrv = &metadata["msrv"];
            assert!(matches!(msrv, Item::Value(Value::String(s)) if s.value() == "1.55"));

            let k = &metadata["k"];
            assert!(matches!(k, Item::Value(Value::String(s)) if s.value() == "1.54"));
        }
    }

    #[test]
    fn set_and_reparse() {
        const METADATA_MSRV: BareVersion = BareVersion::TwoComponents(1, 55);

        let input = r#"[package]
name = "package_name"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        let mut manifest = CargoManifestParser::default()
            .parse::<Document>(input)
            .unwrap();

        insert_new_msrv(&mut manifest, &METADATA_MSRV).unwrap();

        let output = manifest.to_string();
        let new_manifest: CargoManifest = CargoManifestParser::default()
            .parse::<Document>(&output)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(new_manifest.minimum_rust_version().unwrap(), &METADATA_MSRV)
    }
}
