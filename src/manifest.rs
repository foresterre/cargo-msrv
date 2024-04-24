use crate::manifest::bare_version::BareVersion;
use cargo_metadata::{semver, Metadata};
use std::convert::TryFrom;
use toml_edit::{DocumentMut, TomlError};

pub(crate) mod bare_version;

pub trait TomlParser {
    type Error;

    fn try_parse<T: TryFrom<DocumentMut, Error = Self::Error>>(
        &self,
        contents: &str,
    ) -> Result<T, Self::Error>;

    fn parse<T: From<DocumentMut>>(&self, contents: &str) -> Result<T, Self::Error>;
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

    fn try_parse<T: TryFrom<DocumentMut, Error = Self::Error>>(
        &self,
        contents: &str,
    ) -> Result<T, Self::Error> {
        contents.parse::<DocumentMut>().and_then(TryFrom::try_from)
    }

    fn parse<T: From<DocumentMut>>(&self, contents: &str) -> Result<T, Self::Error> {
        contents.parse().map(From::from)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The minimum rust version in your manifest file could not be parsed: {inner}")]
pub struct ManifestParseError {
    pub inner: bare_version::Error,
}

impl From<bare_version::Error> for ManifestParseError {
    fn from(bare: bare_version::Error) -> Self {
        Self { inner: bare }
    }
}

impl TryFrom<Metadata> for CargoManifest {
    type Error = ManifestParseError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        let minimum_rust_version = find_minimum_rust_version(&metadata)?;

        Ok(Self {
            minimum_rust_version,
        })
    }
}

/// Parse the minimum supported Rust version (MSRV) from `Cargo.toml` metadata.
fn find_minimum_rust_version(
    metadata: &Metadata,
) -> Result<Option<BareVersion>, bare_version::Error> {
    /// Parses the `MSRV` as supported by Cargo since Rust 1.56.0
    ///
    /// [`Cargo`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field
    fn find_rust_version(metadata: &Metadata) -> Option<&semver::Version> {
        metadata.root_package()?.rust_version.as_ref()
    }

    /// Parses the MSRV as supported by `cargo-msrv`, since prior to the release of Rust
    /// 1.56.0
    fn find_metadata_msrv(metadata: &Metadata) -> Option<&str> {
        metadata
            .root_package()?
            .metadata
            .as_object()?
            .get("msrv")?
            .as_str()
    }

    // Parse the MSRV from the `package.rust-version` key if it exists,
    // and try to fallback to our own `package.metadata.msrv` if it doesn't
    match find_rust_version(metadata) {
        Some(version) => Ok(Some(BareVersion::from(version))),
        None => find_metadata_msrv(metadata)
            .map(BareVersion::try_from)
            .transpose(),
    }
}

#[cfg(test)]
mod minimal_version_tests {
    use crate::manifest::{BareVersion, CargoManifest};
    use cargo_metadata::Metadata;
    use std::convert::TryFrom;

    fn metadata_json(rust_version: Option<&str>, metadata: Option<&str>) -> String {
        let rust_version = match rust_version {
            Some(rust_version) => format!(r#""rust_version": "{}","#, rust_version),
            None => "".to_string(),
        };
        let metadata = match metadata {
            Some(metadata) => format!(r#""metadata": {},"#, metadata),
            None => "".to_string(),
        };
        format!(
            r#"{{
  "packages": [
    {{
      "name": "some",
      "version": "0.1.0",
      "id": "some 0.1.0 (path+file:///some)",
      "manifest_path": "/some/Cargo.toml",
      {}
      {}
      "dependencies": [],
      "targets": [],
      "features": {{}},
      "edition": "2018"
    }}
  ],
  "workspace_members": [
    "some 0.1.0 (path+file:///some)"
  ],
  "target_directory": "/some/target",
  "version": 1,
  "workspace_root": "/some"
}}"#,
            rust_version, metadata
        )
    }

    #[test]
    fn parse_no_minimum_rust_version() {
        let metadata = metadata_json(None, None);
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();

        assert!(manifest.minimum_rust_version.is_none());
    }

    #[test]
    fn parse_rust_version_three_components() {
        let metadata = metadata_json(Some("1.56.0"), None);
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::ThreeComponents(1, 56, 0));
    }

    #[test]
    fn parse_rust_version_three_components_with_pre_release() {
        let metadata = metadata_json(Some("1.56.0-nightly"), None);
        // cargo_metadata will check this is invalid for us
        let err = serde_json::from_str::<Metadata>(&metadata).unwrap_err();
        assert_eq!(
            err.to_string(),
            "pre-release identifiers are not supported in rust-version at line 8 column 38"
        );
    }

    #[test]
    fn parse_rust_version_two_components() {
        let metadata = metadata_json(Some("1.56"), None);
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        // going through cargo_metadata means it gets turned into 3 components
        assert_eq!(version, BareVersion::ThreeComponents(1, 56, 0));
    }

    #[test]
    fn parse_metadata_msrv_three_components() {
        let metadata = metadata_json(None, Some(r#"{"msrv": "1.51.0"}"#));
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, BareVersion::ThreeComponents(1, 51, 0));
    }

    #[test]
    fn parse_metadata_msrv_two_components() {
        let metadata = metadata_json(None, Some(r#"{"msrv": "1.51"}"#));
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
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
        let msrv = format!(r#"{{"msrv": "{}"}}"#, version);
        let metadata = metadata_json(None, Some(&msrv));
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata);

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
        one_fifty_four_p1 = {  BareVersion::ThreeComponents(1, 54, 1), semver::Version::new(1, 54, 1) },
        one = {  BareVersion::ThreeComponents(1, 0, 0), semver::Version::new(1, 0, 0) },
    )]
    fn three_components_to_semver(version: BareVersion, expected: semver::Version) {
        let index = release_indices();
        let available = index.releases().iter().map(Release::version);

        let v = version.try_to_semver(available).unwrap();

        assert_eq!(v, &expected);
    }
}
