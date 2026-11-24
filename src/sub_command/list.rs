use crate::SubCommand;
use crate::context::ListContext;
use crate::error::{ContextError, TResult};
use crate::reporter::Reporter;
use crate::reporter::event::ListResult;
use camino::Utf8PathBuf;
use cargo_msrv_context::{Project, SelectedPackage};
use cargo_msrv_rust_tools::{CargoMetadataResolver, DependencyResolver};
use std::sync::Arc;

#[derive(Default)]
pub struct List;

impl SubCommand for List {
    type Context = ListContext;
    type Output = ();

    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        list_msrv(ctx, reporter)
    }
}

fn list_msrv(ctx: &ListContext, reporter: &impl Reporter) -> TResult<()> {
    let Project::Cargo(project) = &ctx.environment.project else {
        return Err(ContextError::NoCargoManifest(ctx.environment.manifest()).into());
    };

    let resolver = CargoMetadataResolver::from_manifest_path(&ctx.environment.manifest());
    let graph = Arc::new(resolver.resolve()?);

    for package in project.packages() {
        let root = graph
            .find_by_manifest_path(&package.manifest_path)
            .ok_or_else(|| Error::PackageNotInGraph(package.manifest_path.clone()))?;

        reporter.report_event(
            ListResult::new(ctx.variant, Arc::clone(&graph), root, ctx.highlight.clone())
                .with_package(Some(SelectedPackage::from(package))),
        )?;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to find the dependencies of the package with Cargo manifest at '{0}'")]
    PackageNotInGraph(Utf8PathBuf),
}
