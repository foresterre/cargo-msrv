use crate::config::{Config, ModeIntent};
use crate::errors::{CargoMSRVError, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::paths::crate_root_folder;
use crate::reporter::Output;
use std::convert::TryFrom;
use toml_edit::Document;

pub fn run_show_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::Show);

    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(CargoMSRVError::Io)?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let msrv = manifest.minimum_rust_version();
    if msrv.is_some() {
        output.finish_success(
            ModeIntent::Show,
            msrv.map(BareVersion::to_semver_version).as_ref(),
        );
    } else {
        output.finish_failure(ModeIntent::Show, None);
    }

    Ok(())
}
