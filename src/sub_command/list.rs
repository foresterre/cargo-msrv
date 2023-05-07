use crate::config::Config;
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

    fn run(&self, config: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        list_msrv(config, reporter)
    }
}

fn list_msrv(config: &ListContext, reporter: &impl Reporter) -> TResult<()> {
    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;
    let variant = config.sub_command_config().list().variant;

    reporter.report_event(ListResult::new(variant, graph))?;

    Ok(())
}
