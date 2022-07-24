use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct PathConfig;

impl Configure for PathConfig {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        let path = opts.shared_opts.path.as_ref();

        Ok(builder.crate_path(path))
    }
}
