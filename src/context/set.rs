use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::{EnvironmentContext, RustReleasesContext};
use crate::error::CargoMSRVError;
use crate::manifest::bare_version::BareVersion;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct SetContext {
    /// MSRV to set.
    pub msrv: BareVersion,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

impl TryFrom<CargoMsrvOpts> for SetContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let set_opts = match subcommand {
            SubCommand::Set(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `set`!"),
        };

        let environment = (&shared_opts).try_into()?;

        Ok(Self {
            msrv: set_opts.msrv,
            rust_releases: set_opts.rust_releases_opts.into(),
            environment,
        })
    }
}
