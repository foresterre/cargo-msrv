use crate::{
    config::{list::ListVariant, Config, ModeIntent, OutputFormat},
    dependencies::resolver::{CargoMetadataResolver, DependencyResolver},
    errors::TResult,
    reporter::Output,
};

pub fn run_list_msrv<R: Output>(config: &Config, output: &R) -> TResult<()> {
    use crate::dependencies::formatter;

    output.mode(ModeIntent::List);

    let resolver = CargoMetadataResolver::try_from_config(config)?;
    let graph = resolver.resolve()?;

    match config.sub_command_config().list().variant {
        ListVariant::DirectDeps => match config.output_format() {
            OutputFormat::Human => {
                use crate::reporter::ui::HumanPrinter;
                let formatter = formatter::DirectDependenciesFormatter::<HumanPrinter>::new(graph);
                output.write_line(&format!("{}", formatter));
            }
            OutputFormat::Json => {
                use crate::reporter::json::JsonPrinter;
                let formatter = formatter::DirectDependenciesFormatter::<JsonPrinter>::new(graph);
                output.write_line(&format!("{}", formatter));
            }
            OutputFormat::None | OutputFormat::TestSuccesses => {}
        },
        ListVariant::OrderedByMSRV => match config.output_format() {
            OutputFormat::Human => {
                use crate::reporter::ui::HumanPrinter;
                let formatter = formatter::ByMSRVFormatter::<HumanPrinter>::new(graph);
                output.write_line(&format!("{}", formatter));
            }
            OutputFormat::Json => {
                use crate::reporter::json::JsonPrinter;
                let formatter = formatter::ByMSRVFormatter::<JsonPrinter>::new(graph);
                output.write_line(&format!("{}", formatter));
            }
            OutputFormat::None | OutputFormat::TestSuccesses => {}
        },
    }

    output.finish_success(ModeIntent::List, None);

    Ok(())
}
