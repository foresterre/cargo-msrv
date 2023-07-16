use camino::Utf8PathBuf;
use std::convert::TryFrom;

use toml_edit::Document;

use crate::context::ShowContext;
use crate::error::{IoError, IoErrorSource, TResult};

use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::reporter::event::ShowResult;
use crate::reporter::Reporter;
use crate::SubCommand;

#[derive(Default)]
pub struct Show;

impl SubCommand for Show {
    type Context = ShowContext;
    type Output = ();

    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        show_msrv(ctx, reporter)
    }
}

fn show_msrv(ctx: &ShowContext, reporter: &impl Reporter) -> TResult<()> {
    let cargo_toml = ctx.environment.manifest();

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| IoError {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.to_path_buf()),
    })?;

    let manifest = CargoManifestParser.parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let msrv = manifest
        .minimum_rust_version()
        .ok_or_else(|| Error::NoMSRVInCargoManifest(cargo_toml.to_path_buf()))?;

    reporter.report_event(ShowResult::new(msrv.clone(), cargo_toml.clone()))?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MSRV was not specified in Cargo manifest at '{0}'")]
    NoMSRVInCargoManifest(Utf8PathBuf),
}
