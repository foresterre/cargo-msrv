use thiserror::private::PathAsDisplay;

use crate::config::set::SetCmdConfig;
use crate::config::{ConfigBuilder, SubCommandConfig};
use crate::reporter::event::{AuxiliaryOutput, Destination, Item};
use crate::reporter::Reporter;
use crate::{semver, Action, Config, OutputFormat, Set, SubCommand, TResult};

/// Write the MSRV to the Cargo manifest
///
/// Repurposes the Set MSRV subcommand for this action.
// FIXME: Currently we do not completely wire up the reporter, but it should be. The after the storyteller
//  integration it will be much easier to write appropriate output messages. In this case, we should
//  not write the complete startup message and wind down message, just an update that the MSRV
//  has been written.
pub fn write_msrv(
    config: &Config,
    reporter: &impl Reporter,
    version: &semver::Version,
) -> TResult<()> {
    let config = ConfigBuilder::from_config(config)
        .mode_intent(Action::Set)
        .sub_command_config(SubCommandConfig::SetConfig(SetCmdConfig {
            msrv: version.into(),
        }))
        .build();

    Set::default().run(&config, reporter)?;

    Ok(())
}
