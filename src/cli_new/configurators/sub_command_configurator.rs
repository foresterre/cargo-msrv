use crate::cli_new::configurators::Configure;
use crate::cli_new::{CargoMsrvOpts, ListOpts, SetOpts, SubCommand};
use crate::config::list::ListCmdConfig;
use crate::config::set::SetCmdConfig;
use crate::config::{ConfigBuilder, SubCommandConfig};
use crate::TResult;

pub(in crate::cli_new) struct SubCommandConfigurator;

impl Configure for SubCommandConfigurator {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        if let Some(cmd) = &opts.subcommand {
            match cmd {
                SubCommand::List(opts) => {
                    return configure_list(builder, opts);
                }
                SubCommand::Set(opts) => {
                    return configure_set(builder, opts);
                }
                _ => {}
            }
        }

        Ok(builder)
    }
}

fn configure_list<'c>(
    builder: ConfigBuilder<'c>,
    opts: &'c ListOpts,
) -> TResult<ConfigBuilder<'c>> {
    let config = ListCmdConfig {
        variant: opts.variant,
    };

    let config = SubCommandConfig::ListConfig(config);
    Ok(builder.sub_command_config(config))
}

fn configure_set<'c>(builder: ConfigBuilder<'c>, opts: &'c SetOpts) -> TResult<ConfigBuilder<'c>> {
    let config = SetCmdConfig {
        msrv: opts.msrv.clone(),
    };

    let config = SubCommandConfig::SetConfig(config);
    Ok(builder.sub_command_config(config))
}
