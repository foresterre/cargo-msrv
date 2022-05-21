use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct OutputToolchainFile;

impl Configure for OutputToolchainFile {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        Ok(builder.output_toolchain_file(opts.find_opts.write_toolchain_file))
    }
}
