use crate::cli::CargoMsrvOpts;
use crate::context::{EnvironmentContext, UserOutputContext};
use crate::error::CargoMSRVError;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct ShowContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,
}

impl TryFrom<CargoMsrvOpts> for ShowContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts { shared_opts, .. } = opts;

        Ok(Self {
            environment: (&shared_opts).try_into().unwrap(), // todo!
            user_output: shared_opts.user_output_opts.into(),
        })
    }
}
