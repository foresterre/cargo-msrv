use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli_new) struct CheckFeedback;

impl Configure for CheckFeedback {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        Ok(builder.no_check_feedback(opts.find_opts.no_check_feedback))
    }
}
