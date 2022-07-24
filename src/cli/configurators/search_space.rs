use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct IncludeAllPatchReleases;

impl Configure for IncludeAllPatchReleases {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.include_all_patch_releases(
            opts.find_opts.rust_releases_opts.include_all_patch_releases,
        ))
    }
}
