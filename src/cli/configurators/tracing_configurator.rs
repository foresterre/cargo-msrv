use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::{ConfigBuilder, TracingOptions};
use crate::TResult;

pub(in crate::cli) struct Tracing;

impl Configure for Tracing {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        if opts.shared_opts.debug_output_opts.no_log {
            return Ok(builder);
        }

        let tracing_opts = TracingOptions::new(
            opts.shared_opts.debug_output_opts.log_target,
            opts.shared_opts.debug_output_opts.log_level,
        );

        Ok(builder.tracing_config(tracing_opts))
    }
}
