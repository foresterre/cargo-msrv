use crate::cli::configurators::Configure;
use crate::cli::find_opts::FindOpts;
use crate::cli::{CargoMsrvOpts, SubCommand, VerifyOpts};
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct CustomCheckCommand;

impl Configure for CustomCheckCommand {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        fn configure_from_verify<'c>(
            builder: ConfigBuilder<'c>,
            opts: &'c VerifyOpts,
        ) -> ConfigBuilder<'c> {
            if opts.custom_check.custom_check_command.is_empty() {
                return builder;
            }

            let cmd = opts
                .custom_check
                .custom_check_command
                .iter()
                .map(|s| s.as_str())
                .collect();

            builder.check_command(cmd)
        }

        fn configure_from_find<'c>(
            builder: ConfigBuilder<'c>,
            opts: &'c FindOpts,
        ) -> ConfigBuilder<'c> {
            if opts.custom_check_opts.custom_check_command.is_empty() {
                return builder;
            }

            let cmd = opts
                .custom_check_opts
                .custom_check_command
                .iter()
                .map(|s| s.as_str())
                .collect();

            builder.check_command(cmd)
        }

        let builder = match &opts.subcommand {
            Some(SubCommand::Verify(verify)) => configure_from_verify(builder, verify),
            None => configure_from_find(builder, &opts.find_opts),
            _ => builder,
        };

        Ok(builder)
    }
}
