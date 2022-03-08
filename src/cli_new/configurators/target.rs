use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli_new) struct Target;

impl Configure for Target {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        // TODO{foresterre}: maybe also for `verify`, not just `find`?
        if let Some(target) = &opts.find_opts.toolchain_opts.target {
            Ok(builder.target(target.as_str()))
        } else {
            Ok(builder)
        }
    }
}
