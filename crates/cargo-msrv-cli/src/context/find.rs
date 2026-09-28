use crate::cli::{CargoMsrvOpts, SubCommand};
use cargo_msrv_context::context::error::{Error, TResult};
use cargo_msrv_context::{FindContext, SearchMethod};
use std::convert::{TryFrom, TryInto};

impl TryFrom<CargoMsrvOpts> for FindContext {
    type Error = Error;

    fn try_from(opts: CargoMsrvOpts) -> TResult<Self> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
        } = opts;

        let find_opts = match subcommand {
            SubCommand::Find(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `find`!"),
        };

        let toolchain = find_opts.toolchain_opts.try_into()?;
        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            search_method: if find_opts.linear {
                SearchMethod::Linear
            } else if find_opts.heuristic {
                SearchMethod::Heuristic
            } else {
                SearchMethod::Bisect
            },
            heuristic_source: find_opts.heuristic_source.unwrap_or_default(),
            write_toolchain_file: find_opts.write_toolchain_file,
            ignore_lockfile: find_opts.ignore_lockfile,
            skip_unavailable_toolchains: find_opts.skip_unavailable_toolchains,
            no_check_feedback: find_opts.no_check_feedback,
            write_msrv: find_opts.write_msrv,
            rust_releases: find_opts.rust_releases_opts.into(),
            toolchain,
            check_cmd: find_opts.custom_check_opts.into(),
            environment,
        })
    }
}
