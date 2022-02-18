use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::{ConfigBuilder, SearchMethod};
use crate::TResult;

pub(in crate::cli_new) struct SearchMethodConfig;

impl Configure for SearchMethodConfig {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        let method = match (opts.find_opts.linear, opts.find_opts.bisect) {
            (true, false) => builder.search_method(SearchMethod::Linear),
            (false, true) => builder.search_method(SearchMethod::Bisect),
            _ => builder.search_method(SearchMethod::default()),
        };

        Ok(method)
    }
}
