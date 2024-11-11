use crate::cli::CargoMsrvOpts;
use crate::context::EnvironmentContext;
use crate::error::CargoMSRVError;

#[derive(Debug)]
pub struct DoctorContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,
}

impl TryFrom<CargoMsrvOpts> for DoctorContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts { shared_opts, .. } = opts;

        Ok(Self {
            environment: (&shared_opts).try_into()?,
        })
    }
}
