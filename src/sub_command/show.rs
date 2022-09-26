use std::convert::TryFrom;
use std::path::PathBuf;

use toml_edit::Document;

use crate::config::Config;
use crate::error::{CargoMSRVError, IoErrorSource, TResult};

use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::reporter::event::ShowResult;
use crate::reporter::EventReporter;
use crate::SubCommand;

#[derive(Default)]
pub struct Show;

impl SubCommand for Show {
    type Output = ();

    fn run(&self, config: &Config, reporter: &impl EventReporter) -> TResult<Self::Output> {
        show_msrv(config, reporter)
    }
}

fn show_msrv(config: &Config, reporter: &impl EventReporter) -> TResult<()> {
    let cargo_toml = config.context().manifest_path()?;

    let contents = std::fs::read_to_string(cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.to_path_buf()),
    })?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let msrv = manifest
        .minimum_rust_version()
        .ok_or_else(|| Error::NoMSRVInCargoManifest(cargo_toml.to_path_buf()))?;

    reporter.report_event(ShowResult::new(msrv.clone(), cargo_toml.to_path_buf()))?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MSRV was not specified in Cargo manifest at '{}'", .0.display())]
    NoMSRVInCargoManifest(PathBuf),
}
