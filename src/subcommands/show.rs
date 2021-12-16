use std::convert::TryFrom;

use toml_edit::Document;

use crate::{
    config::{Config, ModeIntent},
    errors::{CargoMSRVError, IoErrorSource, TResult},
    manifest::{bare_version::BareVersion, CargoManifest, CargoManifestParser, TomlParser},
    paths::crate_root_folder,
    reporter::Output,
};

pub fn run_show_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::Show);

    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml),
    })?;

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
