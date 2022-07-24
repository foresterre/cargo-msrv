use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct ManifestPathConfig;

impl Configure for ManifestPathConfig {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        let path = opts.shared_opts.manifest_path.as_ref();

        Ok(builder.manifest_path(path))
    }
}
