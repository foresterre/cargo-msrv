use crate::check_cmd::StaticCheckCommand;
use crate::cli::configurators::Configure;
use crate::cli::find_opts::FindOpts;
use crate::cli::{CargoMsrvOpts, SubCommand, VerifyOpts};
use crate::config::{ConfigBuilder, SelectedCheckCommand};
use crate::TResult;

pub(in crate::cli) struct CustomCheckCommand;

impl Configure for CustomCheckCommand {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        fn configure_from_verify(builder: ConfigBuilder, opts: &VerifyOpts) -> ConfigBuilder {
            if opts.custom_check.custom_check_command.is_empty() {
                return builder;
            }

            let cmd = opts.custom_check.custom_check_command.join(" ");
            let cmd = SelectedCheckCommand::Static(StaticCheckCommand::new(cmd));
            builder.check_command(cmd)
        }

        fn configure_from_find(builder: ConfigBuilder, opts: &FindOpts) -> ConfigBuilder {
            if opts.custom_check_opts.custom_check_command.is_empty() {
                return builder;
            }

            let cmd = opts.custom_check_opts.custom_check_command.join(" ");
            builder.check_command(SelectedCheckCommand::Static(StaticCheckCommand::new(cmd)))
        }

        let builder = match &opts.subcommand {
            Some(SubCommand::Verify(verify)) => configure_from_verify(builder, verify),
            None => configure_from_find(builder, &opts.find_opts),
            _ => builder,
        };

        Ok(builder)
    }
}
