use crate::config::list::ListVariant;
use crate::config::{Config, ModeIntent};
use crate::dependencies::formatter::{direct_deps, ordered_by_msrv};
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::errors::TResult;
use crate::reporter::Output;

pub fn run_list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    let format = config.output_format();
    let variant = config.sub_command_config().list().variant;

    if let Some(s) = match variant {
        ListVariant::DirectDeps => direct_deps::format(format, &graph),
        ListVariant::OrderedByMSRV => ordered_by_msrv::format(format, &graph),
    } {
        output.write_line(&s)
    }

    output.finish_success(ModeIntent::List, None);

    Ok(())
}
