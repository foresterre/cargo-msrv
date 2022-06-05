use crate::config::{Config};
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::errors::TResult;
use crate::storyteller::Reporter;
use crate::{dependencies, SubCommand};

#[derive(Default)]
pub struct List;

impl SubCommand for List {
    fn run(&self, config: &Config, reporter: &impl Reporter) -> TResult<()> {
        list_msrv(config, reporter)
    }
}

fn list_msrv(config: &Config, _reporter: &impl Reporter) -> TResult<()> {
    // todo!
    // output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    let format = config.output_format();
    let variant = config.sub_command_config().list().variant;

    if let Some(_s) = dependencies::format(&graph, variant, format) {
        // todo!
        // output.write_line(&s);
    }

    // todo!
    // output.finish_success(ModeIntent::List, None);

    Ok(())
}
