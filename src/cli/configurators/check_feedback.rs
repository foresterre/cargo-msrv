use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use crate::TResult;

pub(in crate::cli) struct CheckFeedback;

impl Configure for CheckFeedback {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        Ok(builder.no_check_feedback(opts.find_opts.no_check_feedback))
    }
}
