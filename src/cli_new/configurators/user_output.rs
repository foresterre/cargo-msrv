use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::{ConfigBuilder, OutputFormat};
use crate::TResult;

pub(in crate::cli_new) struct UserOutput;

impl Configure for UserOutput {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        if opts.shared_opts.user_output_opts.no_user_output {
            return Ok(builder.output_format(OutputFormat::None));
        }

        let format = opts.shared_opts.user_output_opts.output_format;
        Ok(builder.output_format(format))
    }
}
