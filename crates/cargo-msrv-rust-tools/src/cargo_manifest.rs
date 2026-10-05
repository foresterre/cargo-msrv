use cargo_metadata::{Metadata, Package, semver};
use std::convert::TryFrom;
use toml_edit::{DocumentMut, TomlError};
use version_number::{FromSemverError, FullVersion, Version};

pub trait TomlParser {
    type Error;

    fn parse<T: From<DocumentMut>>(&self, contents: &str) -> Result<T, Self::Error>;
}

/// A structure for owning the values in a `Cargo.toml` manifest relevant for `cargo-msrv`.
#[derive(Debug)]
pub struct CargoManifest {
    minimum_rust_version: Option<Version>,
}

impl CargoManifest {
    pub fn minimum_rust_version(&self) -> Option<&Version> {
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

    fn parse<T: From<DocumentMut>>(&self, contents: &str) -> Result<T, Self::Error> {
        contents.parse().map(From::from)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestParseError {
    #[error("The minimum rust version in your manifest file could not be parsed: {0}")]
    MetadataMsrv(#[from] version_number::Error),

    #[error("The rust-version in your manifest file is not supported: {0}")]
    RustVersion(#[from] FromSemverError),
}

impl TryFrom<Metadata> for CargoManifest {
    type Error = ManifestParseError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        match metadata.root_package() {
            Some(package) => Self::for_package(package),
            None => Ok(Self {
                minimum_rust_version: None,
            }),
        }
    }
}

impl CargoManifest {
    pub fn for_package(package: &Package) -> Result<Self, ManifestParseError> {
        let minimum_rust_version = find_minimum_rust_version(package)?;

        Ok(Self {
            minimum_rust_version,
        })
    }
}

/// Parse the minimum supported Rust version (MSRV) from `Cargo.toml` metadata.
fn find_minimum_rust_version(package: &Package) -> Result<Option<Version>, ManifestParseError> {
    /// Parses the `MSRV` as supported by Cargo since Rust 1.56.0
    ///
    /// [`Cargo`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field
    fn find_rust_version(package: &Package) -> Option<&semver::Version> {
        package.rust_version.as_ref()
    }

    /// Parses the MSRV as supported by `cargo-msrv`, since prior to the release of Rust
    /// 1.56.0
    fn find_metadata_msrv(package: &Package) -> Option<&str> {
        package.metadata.as_object()?.get("msrv")?.as_str()
    }

    // Parse the MSRV from the `package.rust-version` key if it exists,
    // and try to fallback to our own `package.metadata.msrv` if it doesn't
    match find_rust_version(package) {
        Some(version) => Ok(Some(Version::from(FullVersion::try_from(version)?))),
        None => Ok(find_metadata_msrv(package)
            .map(Version::parse)
            .transpose()?),
    }
}

#[cfg(test)]
mod minimal_version_tests {
    use crate::cargo_manifest::{CargoManifest, Version};
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

        assert_eq!(version, Version::new_full_version(1, 56, 0));
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
        assert_eq!(version, Version::new_full_version(1, 56, 0));
    }

    #[test]
    fn parse_metadata_msrv_three_components() {
        let metadata = metadata_json(None, Some(r#"{"msrv": "1.51.0"}"#));
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, Version::new_full_version(1, 51, 0));
    }

    #[test]
    fn parse_metadata_msrv_two_components() {
        let metadata = metadata_json(None, Some(r#"{"msrv": "1.51"}"#));
        let metadata: Metadata = serde_json::from_str(&metadata).unwrap();

        let manifest = CargoManifest::try_from(metadata).unwrap();
        let version = manifest.minimum_rust_version.unwrap();

        assert_eq!(version, Version::new_base_version(1, 51));
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
