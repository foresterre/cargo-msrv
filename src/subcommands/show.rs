use std::convert::TryFrom;

use toml_edit::Document;

use crate::config::Config;
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};

use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::paths::crate_root_folder;
use crate::reporter::Reporter;
use crate::SubCommand;

#[derive(Default)]
pub struct Show;

impl SubCommand for Show {
    type Output = ();

    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<Self::Output> {
        show_msrv(config, reporter)
    }
}

fn show_msrv(config: &Config, _reporter: &impl Reporter) -> TResult<()> {
    // todo!
    // output.mode(ModeIntent::Show);

    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml),
    })?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let msrv = manifest.minimum_rust_version();
    #[allow(clippy::if_same_then_else)]
    if msrv.is_some() {
        // todo!
        // output.finish_success(
        //     ModeIntent::Show,
        //     msrv.map(BareVersion::to_semver_version).as_ref(),
        // );
    } else {
        // todo!
        // output.finish_failure(ModeIntent::Show, None);
    }

    Ok(())
}
