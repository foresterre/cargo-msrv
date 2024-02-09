use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use std::convert::TryFrom;

use crate::context::ShowContext;
use crate::error::TResult;

use crate::manifest::CargoManifest;
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

    let metadata = MetadataCommand::new().manifest_path(&cargo_toml).exec()?;
    let manifest = CargoManifest::try_from(metadata)?;

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
