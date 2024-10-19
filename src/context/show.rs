use crate::cli::CargoMsrvOpts;
use crate::context::EnvironmentContext;
use crate::error::CargoMSRVError;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct ShowContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,
}

impl TryFrom<CargoMsrvOpts> for ShowContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts { shared_opts, .. } = opts;

        Ok(Self {
            environment: (&shared_opts).try_into()?,
        })
    }
}
