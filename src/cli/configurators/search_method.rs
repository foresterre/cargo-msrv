use crate::cli::configurators::Configure;
use crate::cli::CargoMsrvOpts;
use crate::config::{ConfigBuilder, SearchMethod};
use crate::TResult;

pub(in crate::cli) struct SearchMethodConfig;

impl Configure for SearchMethodConfig {
    fn configure(builder: ConfigBuilder, opts: &CargoMsrvOpts) -> TResult<ConfigBuilder> {
        let method = match (opts.find_opts.linear, opts.find_opts.bisect) {
            (true, false) => builder.search_method(SearchMethod::Linear),
            (false, true) => builder.search_method(SearchMethod::Bisect),
            _ => builder.search_method(SearchMethod::default()),
        };

        Ok(method)
    }
}
