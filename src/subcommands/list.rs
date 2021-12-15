use crate::config::list::ListVariant;
use crate::config::{Config, ModeIntent, OutputFormat};
use crate::dependencies::formatter::direct_deps;
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::errors::TResult;
use crate::reporter::Output;

pub fn run_list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    use crate::dependencies::formatter;

    output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    let format = config.output_format();
    let variant = config.sub_command_config().list().variant;

    if let Some(s) = match variant {
        ListVariant::DirectDeps => direct_deps::format(format, &graph),
        ListVariant::OrderedByMSRV => match format {
            OutputFormat::Human => {
                use crate::reporter::ui::HumanPrinter;
                let formatter = formatter::ByMSRVFormatter::<HumanPrinter>::new(graph);
                Some(formatter.to_string())
            }
            OutputFormat::Json => {
                use crate::reporter::json::JsonPrinter;
                let formatter = formatter::ByMSRVFormatter::<JsonPrinter>::new(graph);
                Some(formatter.to_string())
            }
            OutputFormat::None | OutputFormat::TestSuccesses => None,
        },
    } {
        output.write_line(&s)
    }

    output.finish_success(ModeIntent::List, None);

    Ok(())
}
