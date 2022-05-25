use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct WriteMsrv;

impl Configure for WriteMsrv {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        Ok(builder.write_msrv(opts.find_opts.write_msrv))
    }
}
