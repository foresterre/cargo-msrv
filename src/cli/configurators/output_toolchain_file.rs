use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct OutputToolchainFile;

impl Configure for OutputToolchainFile {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.output_toolchain_file(opts.find_opts.write_toolchain_file))
    }
}
