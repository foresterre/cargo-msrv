use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct MaxVersion;

impl Configure for MaxVersion {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        if let Some(max) = &opts.find_opts.rust_releases_opts.max {
            Ok(builder.maximum_version(max.clone()))
        } else {
            Ok(builder)
        }
    }
}
