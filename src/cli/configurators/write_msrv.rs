use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct WriteMsrv;

impl Configure for WriteMsrv {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.write_msrv(opts.find_opts.write_msrv))
    }
}
