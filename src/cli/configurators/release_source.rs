use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct ReleaseSource;

impl Configure for ReleaseSource {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.release_source(opts.find_opts.rust_releases_opts.release_source))
    }
}
