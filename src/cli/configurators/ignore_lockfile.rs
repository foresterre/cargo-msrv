use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct IgnoreLockfile;

impl Configure for IgnoreLockfile {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.ignore_lockfile(opts.find_opts.ignore_lockfile))
    }
}
