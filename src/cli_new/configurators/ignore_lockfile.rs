use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli_new) struct IgnoreLockfile;

impl Configure for IgnoreLockfile {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        Ok(builder.ignore_lockfile(opts.find_opts.ignore_lockfile))
    }
}
