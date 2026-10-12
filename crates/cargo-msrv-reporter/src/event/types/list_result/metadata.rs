use cargo_metadata::{MetadataCommand, Package};
use cargo_msrv_rust_tools::CargoManifest;
use msrv::MSRV;
use std::convert::TryFrom;
use std::path::Path;
use version_number::FullVersion;

pub fn package_msrv(package: &Package) -> Option<MSRV> {
    package
        .rust_version
        .as_ref()
        .and_then(semver_to_msrv)
        .or_else(|| get_package_metadata_msrv(package))
        .or_else(|| parse_manifest_workaround(package.manifest_path.as_path())) // todo: add last one as option to config
}

pub fn format_version(version: Option<&MSRV>) -> Option<String> {
    version.map(ToString::to_string)
}

// Workaround: manual parsing since current (1.56) version of cargo-metadata doesn't yet output the
//  rust-version
pub fn parse_manifest_workaround<P: AsRef<Path>>(path: P) -> Option<MSRV> {
    fn parse(path: &Path) -> Option<MSRV> {
        MetadataCommand::new()
            .manifest_path(path)
            .exec()
            .ok()
            .and_then(|metadata| CargoManifest::try_from(metadata).ok())
            .and_then(|manifest| {
                manifest
                    .minimum_rust_version()
                    .map(|msrv| MSRV::from(msrv.version().to_full_version_lossy()))
            })
    }

    parse(path.as_ref())
}

pub(in crate::event) fn get_package_metadata_msrv(package: &Package) -> Option<MSRV> {
    package
        .metadata
        .get("msrv")
        .and_then(|v| v.as_str())
        .and_then(|v| semver::Version::parse(v).ok())
        .as_ref()
        .and_then(semver_to_msrv)
}

fn semver_to_msrv(version: &semver::Version) -> Option<MSRV> {
    FullVersion::try_from(version).ok().map(MSRV::from)
}
