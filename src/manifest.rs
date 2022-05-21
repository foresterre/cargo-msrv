use crate::manifest::bare_version::BareVersion;
use std::convert::TryFrom;
use toml_edit::{Document, Item, TomlError};

pub(crate) mod bare_version;

pub trait TomlParser {
    type Error;

    fn try_parse<T: TryFrom<Document, Error = Self::Error>>(
        &self,
        contents: &str,
    ) -> Result<T, Self::Error>;

    fn parse<T: From<Document>>(&self, contents: &str) -> Result<T, Self::Error>;
}

/// A structure for owning the values in a `Cargo.toml` manifest relevant for `cargo-msrv`.
#[derive(Debug)]
pub struct CargoManifest {
    minimum_rust_version: Option<BareVersion>,
}

impl CargoManifest {
    pub fn minimum_rust_version(&self) -> Option<&BareVersion> {
        self.minimum_rust_version.as_ref()
    }
}

/// A parser for `Cargo.toml` files. Only handles the parts necessary for `cargo-msrv`.
#[derive(Debug)]
pub struct CargoManifestParser;

impl Default for CargoManifestParser {
    fn default() -> Self {
        Self
    }
}

impl TomlParser for CargoManifestParser {
    type Error = TomlError;

    fn try_parse<T: TryFrom<Document, Error = Self::Error>>(
        &self,
        contents: &str,
    ) -> Result<T, Self::Error> {
        contents.parse::<Document>().and_then(TryFrom::try_from)
    }

    fn parse<T: From<Document>>(&self, contents: &str) -> Result<T, Self::Error> {
        contents.parse().map(From::from)
    }
}

impl TryFrom<Document> for CargoManifest {
    type Error = crate::CargoMSRVError;

    fn try_from(map: Document) -> Result<Self, Self::Error> {
        let minimum_rust_version = minimum_rust_version(&map)?;

        Ok(Self {
            minimum_rust_version,
        })
    }
}

fn minimum_rust_version(value: &Document) -> Result<Option<BareVersion>, crate::CargoMSRVError> {
    let version = match find_minimum_rust_version(value) {
        Some(version) => version,
        None => return Ok(None),
    };

    Ok(Some(version.parse()?))
}

/// Parse the minimum supported Rust version (MSRV) from `Cargo.toml` manifest data.
fn find_minimum_rust_version(document: &Document) -> Option<&str> {
    /// Parses the `MSRV` as supported by Cargo since Rust 1.56.0
    ///
    /// [`Cargo`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field
    fn find_rust_version(document: &Document) -> Option<&str> {
        document
            .as_table()
            .get("package")
            .and_then(Item::as_table)
            .and_then(|package| package.get("rust-version"))
            .and_then(Item::as_str)
    }

    /// Parses the MSRV as supported by `cargo-msrv`, since prior to the release of Rust
    /// 1.56.0
    fn find_metadata_msrv(document: &Document) -> Option<&str> {
        document
            .as_table()
            .get("package")
            .and_then(Item::as_table)
            .and_then(|package| package.get("metadata"))
            .and_then(Item::as_table_like)
            .and_then(|metadata| metadata.get("msrv"))
            .and_then(Item::as_str)
    }

    // Parse the MSRV from the `package.rust-version` key if it exists,
    // and try to fallback to our own `package.metadata.msrv` if it doesn't
    find_rust_version(document).or_else(|| find_metadata_msrv(document))
}

#[cfg(test)]
mod minimal_version_tests {
    use crate::errors::CargoMSRVError;
    use crate::manifest::bare_version::Error;
    use crate::manifest::{BareVersion, CargoManifest, CargoManifestParser, TomlParser};
    use std::convert::TryFrom;
    use toml_edit::Document;

    #[test]
    fn parse_toml() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"

[dependencies]
"#;

        assert!(CargoManifestParser::default()
            .parse::<Document>(contents)
            .is_ok());
    }

    #[test]
    fn parse_invalid_toml() {
        let contents = r#"-[package]
name = "some"
version = "0.1.0"
edition = "2018"

[dependencies]
"#;

        assert!(CargoManifestParser::default()
            .parse::<Document>(contents)
            .is_err());
    }

    #[test]
    fn parse_no_minimum_rust_version() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();

        assert!(manifest.minimum_rust_version.is_none());
    }

    #[test]
    fn parse_rust_version_three_components() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"
rust-version = "1.56.0"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::ThreeComponents(1, 56, 0));
    }

    #[test]
    fn parse_rust_version_three_components_with_pre_release() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"
rust-version = "1.56.0-nightly"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let parse_err = CargoManifest::try_from(manifest).unwrap_err();

        if let CargoMSRVError::BareVersionParse(err) = parse_err {
            assert_eq!(err, Error::PreReleaseModifierNotAllowed);
        } else {
            panic!("Incorrect cargo-msrv error type");
        }
    }

    #[test]
    fn parse_rust_version_two_components() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"
rust-version = "1.56"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::TwoComponents(1, 56));
    }

    #[yare::parameterized(
        empty = {""},
        one_component = {"1"},
        one_component_dot = {"1."},
        two_components_dot = {"1.1."},
        three_components_dot = {"1.1.1."},
        two_components_with_pre_release = {"1.1-nightly"},
        two_components_not_a_number = {"1.x"},
        three_components_not_a_number = {"1.1.x"},
        too_many_components = {"1.1.0.0"},
    )]
    fn parse_rust_version_faulty_versions(version: &str) {
        let contents = format!(
            r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"
rust-version = "{}"

[dependencies]
"#,
            version
        );

        let manifest = CargoManifestParser::default()
            .parse::<Document>(&contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest);

        assert!(manifest.is_err());
    }

    #[test]
    fn parse_metadata_msrv_three_components() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"

[package.metadata]
msrv = "1.51.0"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::ThreeComponents(1, 51, 0));
    }

    #[test]
    fn parse_metadata_msrv_two_components() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"

[package.metadata]
msrv = "1.51"

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::TwoComponents(1, 51));
    }

    // While uncommon, seems not to be against Cargo manifest spec; i.e. it just states Table,
    // not specifying whether only the regular or also the inline variant. This is similar to
    // [dependencies] which also accept 'Table' items for each dependency, and where inline tables
    // are more regular (and clearly supported!)
    //
    // NB: when using an inline metadata table, you won't be able to add another [metadata] table,
    //     you must use the same inline table
    #[test]
    fn parse_metadata_msrv_inline() {
        let contents = r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"
metadata = { msrv = "1.51" }

[dependencies]
"#;

        let manifest = CargoManifestParser::default()
            .parse::<Document>(contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::TwoComponents(1, 51));
    }

    #[yare::parameterized(
        empty = {""},
        one_component = {"1"},
        one_component_dot = {"1."},
        two_components_dot = {"1.1."},
        three_components_dot = {"1.1.1."},
        two_components_with_pre_release = {"1.1-nightly"},
        two_components_not_a_number = {"1.x"},
        three_components_not_a_number = {"1.1.x"},
        too_many_components = {"1.1.0.0"},
    )]
    fn parse_metadata_msrv_faulty_versions(version: &str) {
        let contents = format!(
            r#"[package]
name = "some"
version = "0.1.0"
edition = "2018"

[package.metadata]
msrv = "{}"

[dependencies]
"#,
            version
        );

        let manifest = CargoManifestParser::default()
            .parse::<Document>(&contents)
            .unwrap();

        let manifest = CargoManifest::try_from(manifest);

        assert!(manifest.is_err());
    }
}

#[cfg(test)]
mod bare_version_tests {
    use crate::manifest::BareVersion;
    use rust_releases::{semver, Release, ReleaseIndex};
    use std::iter::FromIterator;
    use yare::parameterized;

    fn release_indices() -> ReleaseIndex {
        FromIterator::from_iter(vec![
            Release::new_stable(semver::Version::new(2, 56, 0)),
            Release::new_stable(semver::Version::new(1, 56, 0)),
            Release::new_stable(semver::Version::new(1, 55, 0)),
            Release::new_stable(semver::Version::new(1, 54, 2)),
            Release::new_stable(semver::Version::new(1, 54, 1)),
            Release::new_stable(semver::Version::new(1, 0, 0)),
        ])
    }

    #[parameterized(
        two_component_two_fifty_six = { "2.56", BareVersion::TwoComponents(2, 56) },
        three_component_two_fifty_six = { "2.56.0", BareVersion::ThreeComponents(2, 56, 0) },
        two_component_one_fifty_five = { "1.55", BareVersion::TwoComponents(1, 55) },
        three_component_one_fifty_five = { "1.55.0", BareVersion::ThreeComponents(1, 55, 0) },
        three_component_one_fifty_four = { "1.54.0", BareVersion::ThreeComponents(1, 54, 0) },
        three_component_one_fifty_four_p1 = { "1.54.1", BareVersion::ThreeComponents(1, 54, 1) },
        three_component_one_fifty_four_p10 = { "1.54.10", BareVersion::ThreeComponents(1, 54, 10) },
        two_component_zeros = { "0.0", BareVersion::TwoComponents(0, 0) },
        three_component_zeros = { "0.0.0", BareVersion::ThreeComponents(0, 0, 0) },
        two_component_large_major = { "18446744073709551615.0", BareVersion::TwoComponents(18_446_744_073_709_551_615, 0) },
        two_component_large_minor = { "0.18446744073709551615", BareVersion::TwoComponents(0, 18_446_744_073_709_551_615) },
        three_component_large_major = { "18446744073709551615.0.0", BareVersion::ThreeComponents(18_446_744_073_709_551_615, 0, 0) },
        three_component_large_minor = { "0.18446744073709551615.0", BareVersion::ThreeComponents(0, 18_446_744_073_709_551_615, 0) },
        three_component_large_patch = { "0.0.18446744073709551615", BareVersion::ThreeComponents(0, 0, 18_446_744_073_709_551_615) },
    )]
    fn try_from_ok(version: &str, expected: BareVersion) {
        use std::convert::TryFrom;

        let version = BareVersion::try_from(version).unwrap();

        assert_eq!(version, expected);
    }

    #[parameterized(
        empty = { "" }, // no first component
        no_components_space = { "1 36 0" },
        no_components_comma = { "1,36,0" },
        first_component_nan = { "x.0.0" },
        no_second_component = { "1." },
        second_component_nan = { "1.x" },
        no_third_component = { "1.0." },
        third_component_nan = { "1.36.x" },
        too_large_int_major_2c = { "18446744073709551616.0" },
        too_large_int_minor_2c = { "0.18446744073709551616" },
        too_large_int_major_3c = { "18446744073709551616.0.0" },
        too_large_int_minor_3c = { "0.18446744073709551616.0" },
        too_large_int_patch_3c = { "0.0.18446744073709551616" },
        neg_int_major = { "-1.0.0" },
        neg_int_minor = { "0.-1.0" },
        neg_int_patch = { "0.0.-1" },
        build_postfix_without_pre_release_id = { "0.0.0+some" },
        two_component_pre_release_id_variant_1 = { "0.0-nightly" },
        two_component_pre_release_id_variant_2 = { "0.0-beta.0" },
        two_component_pre_release_id_variant_3 = { "0.0-beta.1" },
        two_component_pre_release_id_variant_4 = { "0.0-anything", },
        two_component_pre_release_id_variant_5 = { "0.0-anything+build" },
        three_component_pre_release_id_variant_2 = { "0.0.0-beta.0" },
        three_component_pre_release_id_variant_3 = { "0.0.0-beta.1" },
        three_component_pre_release_id_variant_1 = { "0.0.0-nightly" },
        three_component_pre_release_id_variant_4 = { "0.0.0-anything" },
        three_component_pre_release_id_variant_5 = { "0.0.0-anything+build" },
    )]
    fn try_from_err(version: &str) {
        use std::convert::TryFrom;

        let res = BareVersion::try_from(version);

        assert!(res.is_err());
    }

    #[parameterized(
        two_fifty_six = {  BareVersion::TwoComponents(2, 56), semver::Version::new(2, 56, 0) },
        one_fifty_six = {  BareVersion::TwoComponents(1, 56), semver::Version::new(1, 56, 0) },
        one_fifty_five = {  BareVersion::TwoComponents(1, 55), semver::Version::new(1, 55, 0) },
        one_fifty_four_p2 = {  BareVersion::TwoComponents(1, 54), semver::Version::new(1, 54, 2) },
        one_fifty_four_p1 = {  BareVersion::TwoComponents(1, 54), semver::Version::new(1, 54, 2) },
        one_fifty_four_p0 = {  BareVersion::TwoComponents(1, 54), semver::Version::new(1, 54, 2) },
        one = {  BareVersion::TwoComponents(1, 0), semver::Version::new(1, 0, 0) },
    )]
    fn two_components_to_semver(version: BareVersion, expected: semver::Version) {
        let index = release_indices();
        let available = index.releases().iter().map(Release::version);

        let v = version.try_to_semver(available).unwrap();

        assert_eq!(v, &expected);
    }

    #[parameterized(
        two_fifty_six = {  BareVersion::ThreeComponents(2, 56, 0), semver::Version::new(2, 56, 0) },
        one_fifty_six = {  BareVersion::ThreeComponents(1, 56, 0), semver::Version::new(1, 56, 0) },
        one_fifty_five = {  BareVersion::ThreeComponents(1, 55, 0), semver::Version::new(1, 55, 0) },
        one_fifty_four_p2 = {  BareVersion::ThreeComponents(1, 54, 2), semver::Version::new(1, 54, 2) },
        one_fifty_four_p1 = {  BareVersion::ThreeComponents(1, 54, 1), semver::Version::new(1, 54, 2) },
        one_fifty_four_p0 = {  BareVersion::ThreeComponents(1, 54, 0), semver::Version::new(1, 54, 2) },
        one = {  BareVersion::ThreeComponents(1, 0, 0), semver::Version::new(1, 0, 0) },
    )]
    fn three_components_to_semver(version: BareVersion, expected: semver::Version) {
        let index = release_indices();
        let available = index.releases().iter().map(Release::version);

        let v = version.try_to_semver(available).unwrap();

        assert_eq!(v, &expected);
    }
}
