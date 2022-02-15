use crate::manifest::bare_version::BareVersion;
use clap::ArgMatches;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub struct SetCmdConfig {
    pub msrv: BareVersion,
}

impl<'a> TryFrom<&'a ArgMatches> for SetCmdConfig {
    type Error = crate::CargoMSRVError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        use crate::cli::id;

        let msrv: BareVersion = args.value_of_t_or_exit(id::SUB_COMMAND_SET_VALUE);

        Ok(Self { msrv })
    }
}
