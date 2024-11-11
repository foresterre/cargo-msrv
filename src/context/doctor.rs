use crate::cli::{CargoMsrvOpts, SubCommand};
use crate::context::EnvironmentContext;
use crate::error::CargoMSRVError;

#[derive(Debug)]
pub struct DoctorContext {
    /// Try and fix the issues found!
    pub fix: bool,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

impl TryFrom<CargoMsrvOpts> for DoctorContext {
    type Error = CargoMSRVError;

    fn try_from(opts: CargoMsrvOpts) -> Result<Self, Self::Error> {
        let CargoMsrvOpts {
            shared_opts,
            subcommand,
            ..
        } = opts;

        let doctor_opts = match subcommand {
            SubCommand::Doctor(opts) => opts,
            _ => unreachable!("This should never happen. The subcommand is not `doctor`!"),
        };

        Ok(Self {
            fix: doctor_opts.fix,
            environment: (&shared_opts).try_into()?,
        })
    }
}
