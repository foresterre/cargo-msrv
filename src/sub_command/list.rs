use crate::context::ListContext;
use crate::dependency_graph::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::error::TResult;
use crate::reporter::event::ListResult;
use crate::reporter::Reporter;
use crate::SubCommand;

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
    let resolver = CargoMetadataResolver::from_manifest_path(&ctx.environment.manifest());
    let graph = resolver.resolve()?;
    let variant = ctx.variant;

    reporter.report_event(ListResult::new(variant, graph))?;

    Ok(())
}
