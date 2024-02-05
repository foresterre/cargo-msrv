use crate::manifest::bare_version::BareVersion;
use crate::manifest::CargoManifest;
use crate::semver;
use cargo_metadata::{MetadataCommand, Package};
use std::convert::TryFrom;
use std::path::Path;

pub fn package_msrv(package: &Package) -> Option<semver::Version> {
    package
        .rust_version
        .clone()
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
        MetadataCommand::new()
            .manifest_path(path)
            .exec()
            .ok()
            .and_then(|metadata| CargoManifest::try_from(metadata).ok())
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
