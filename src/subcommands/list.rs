use crate::config::{Config, ModeIntent};
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::errors::TResult;
use crate::reporter::Output;
use crate::{dependencies, SubCommand};

#[derive(Default)]
pub struct List;

impl SubCommand for List {
    fn run<R: Output>(&self, config: &Config, reporter: &R) -> TResult<()> {
        list_msrv(config, reporter)
    }
}

fn list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    let format = config.output_format();
    let variant = config.sub_command_config().list().variant;

    if let Some(s) = dependencies::format(&graph, variant, format) {
        output.write_line(&s);
    }

    output.finish_success(ModeIntent::List, None);

    Ok(())
}
