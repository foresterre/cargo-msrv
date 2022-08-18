use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::semver;
use cargo_metadata::Package;
use std::convert::TryFrom;
use std::path::Path;
use toml_edit::Document;

pub fn package_msrv(package: &Package) -> Option<semver::Version> {
    package
        .rust_version
        .clone()
        .map(|req| {
            let comparator = &req.comparators[0];
            crate::semver::Version::new(
                comparator.major,
                comparator.minor.unwrap_or_default(),
                comparator.patch.unwrap_or_default(),
            )
        })
        .or_else(|| get_package_metadata_msrv(package))
        .or_else(|| parse_manifest_workaround(package.manifest_path.as_path())) // todo: add last one as option to config
}

pub fn format_version(version: Option<&semver::Version>) -> Option<String> {
    version.map(ToString::to_string)
}

// Workaround: manual parsing since current (1.56) version of cargo-metadata doesn't yet output the
//  rust-version
pub fn parse_manifest_workaround<P: AsRef<Path>>(path: P) -> Option<crate::semver::Version> {
    fn parse(path: &Path) -> Option<semver::Version> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|contents| {
                CargoManifestParser::default()
                    .parse::<Document>(&contents)
                    .ok()
            })
            .and_then(|map| CargoManifest::try_from(map).ok())
            .and_then(|manifest| manifest.minimum_rust_version().map(ToOwned::to_owned))
            .map(|version: BareVersion| version.to_semver_version())
    }

    parse(path.as_ref())
}

pub(in crate::reporter::event) fn get_package_metadata_msrv(
    package: &Package,
) -> Option<crate::semver::Version> {
    package
        .metadata
        .get("msrv")
        .and_then(|v| v.as_str())
        .and_then(|v| semver::Version::parse(v).ok())
}
