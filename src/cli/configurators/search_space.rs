use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct IncludeAllPatchReleases;

impl Configure for IncludeAllPatchReleases {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        Ok(builder.include_all_patch_releases(
            opts.find_opts.rust_releases_opts.include_all_patch_releases,
        ))
    }
}
