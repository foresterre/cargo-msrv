use crate::config::list::ListVariant;
use crate::config::{Config, ModeIntent, OutputFormat};
use crate::dependencies::formatter;
use crate::dependencies::resolver::{CargoMetadataResolver, DependencyResolver};
use crate::dependencies::DependencyGraph;
use crate::errors::TResult;
use crate::reporter::json::JsonPrinter;
use crate::reporter::ui::HumanPrinter;
use crate::reporter::Output;

pub fn run_list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    let output_format = config.output_format();
    let variant = config.sub_command_config().list().variant;

    if let Some(s) = format(output_format, variant, graph) {
        output.write_line(&s)
    }

    output.finish_success(ModeIntent::List, None);

    Ok(())
}

fn format(format: OutputFormat, variant: ListVariant, graph: DependencyGraph) -> Option<String> {
    match (format, variant) {
        (OutputFormat::Human, ListVariant::DirectDeps) => {
            Some(formatter::DirectDependenciesFormatter::<HumanPrinter>::new(graph).to_string())
        }
        (OutputFormat::Human, ListVariant::OrderedByMSRV) => {
            Some(formatter::ByMSRVFormatter::<HumanPrinter>::new(graph).to_string())
        }
        (OutputFormat::Json, ListVariant::DirectDeps) => {
            Some(formatter::DirectDependenciesFormatter::<JsonPrinter>::new(graph).to_string())
        }
        (OutputFormat::Json, ListVariant::OrderedByMSRV) => {
            Some(formatter::ByMSRVFormatter::<JsonPrinter>::new(graph).to_string())
        }
        (OutputFormat::None, _) | (OutputFormat::TestSuccesses, _) => None,
    }
}
