use crate::config::list::ListVariant;
use crate::config::OutputFormat;
use crate::manifest::{bare_version::BareVersion, CargoManifest, CargoManifestParser, TomlParser};
use cargo_metadata::Package;
use rust_releases::semver::Version;
use std::convert::TryFrom;
use std::path::Path;
use toml_edit::Document;

use super::DependencyGraph;

mod direct_deps;
mod ordered_by_msrv;

pub(crate) fn format(
    graph: &DependencyGraph,
    variant: ListVariant,
    format: OutputFormat,
) -> Option<String> {
    match variant {
        ListVariant::DirectDeps => direct_deps::format(graph, format),
        ListVariant::OrderedByMSRV => ordered_by_msrv::format(graph, format),
    }
}

#[allow(unused)]
pub(super) fn format_version_req(version_req: Option<&crate::semver::VersionReq>) -> String {
    if let Some(req) = version_req {
        format!("{}", req)
    } else {
        "".to_string()
    }
}

pub(super) fn format_version(version_req: Option<&crate::semver::Version>) -> String {
    if let Some(req) = version_req {
        format!("{}", req)
    } else {
        "".to_string()
    }
}

// Workaround: manual parsing since current (1.56) version of cargo-metadata doesn't yet output the
//  rust-version
pub(super) fn parse_manifest_workaround<P: AsRef<Path>>(path: P) -> Option<crate::semver::Version> {
    fn parse(path: &Path) -> Option<Version> {
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

pub(super) fn get_package_metadata_msrv(package: &Package) -> Option<crate::semver::Version> {
    package
        .metadata
        .get("msrv")
        .and_then(|v| v.as_str())
        .and_then(|v| crate::semver::Version::parse(v).ok())
}

fn msrv(package: &Package) -> Option<Version> {
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
