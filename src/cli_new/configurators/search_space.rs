use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli_new) struct IncludeAllPatchReleases;

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
