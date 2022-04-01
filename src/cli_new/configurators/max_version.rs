use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli_new) struct MaxVersion;

impl Configure for MaxVersion {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        if let Some(max) = &opts.find_opts.rust_releases_opts.max {
            Ok(builder.maximum_version(max.clone()))
        } else {
            Ok(builder)
        }
    }
}
