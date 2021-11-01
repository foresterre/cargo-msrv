use crate::config::list::ListVariant;
use crate::config::Config;
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::errors::TResult;
use crate::reporter::Output;

pub fn run_list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    use crate::dependencies::formatter;

    let resolver = CargoMetadataResolver::try_from_config(config)?;

    let graph = resolver.resolve()?;

    match config.sub_command_config().as_list_cmd_config().variant {
        ListVariant::DirectDeps => {
            let formatter = formatter::DirectDependenciesFormatter::new(graph);
            output.write_line(&format!("{}", formatter));
        }
        ListVariant::OrderedByMSRV => {
            let formatter = formatter::ByMSRVFormatter::new(graph);
            output.write_line(&format!("{}", formatter));
        }
    }

    Ok(())
}
