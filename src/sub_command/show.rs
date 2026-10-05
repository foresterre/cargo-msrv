use crate::context::ShowContext;
use crate::error::TResult;
use camino::Utf8PathBuf;
use cargo_msrv_context::{Package, Project, SelectedPackage};
use std::fmt;

use crate::SubCommand;
use crate::reporter::Reporter;
use crate::reporter::event::ShowResult;

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
    let Project::Cargo(project) = &ctx.environment.project else {
        return Err(Error::NoCargoManifest(ctx.environment.manifest()).into());
    };

    let mut missing = Vec::new();

    for package in project.packages() {
        match &package.rust_version {
            Some(msrv) => reporter.report_event(
                ShowResult::new(msrv.clone(), package.manifest_path.clone())
                    .with_package(Some(SelectedPackage::from(package))),
            )?,
            None => missing.push(MissingMsrv::from(package)),
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::NoMSRVInCargoManifest(missing).into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No Cargo.toml manifest found at '{0}'")]
    NoCargoManifest(Utf8PathBuf),

    #[error("{}", format_missing(.0))]
    NoMSRVInCargoManifest(Vec<MissingMsrv>),
}

#[derive(Debug)]
pub struct MissingMsrv {
    package: String,
    manifest_path: Utf8PathBuf,
}

impl From<&Package> for MissingMsrv {
    fn from(package: &Package) -> Self {
        Self {
            package: package.name.clone(),
            manifest_path: package.manifest_path.clone(),
        }
    }
}

impl fmt::Display for MissingMsrv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' (Cargo manifest at '{}')",
            self.package, self.manifest_path
        )
    }
}

fn format_missing(missing: &[MissingMsrv]) -> String {
    match missing {
        [single] => format!(
            "MSRV was not specified in Cargo manifest at '{}'",
            single.manifest_path
        ),
        many => format!(
            "MSRV was not specified for {} packages: {}",
            many.len(),
            many.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
