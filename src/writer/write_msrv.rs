use crate::config::set::SetCmdConfig;
use crate::config::{ConfigBuilder, SubCommandConfig};
use crate::reporter::Reporter;
use crate::{semver, Action, Config, Set, SubCommand, TResult};

/// Write the MSRV to the Cargo manifest
///
/// Repurposes the Set MSRV subcommand for this action.
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

    // Output is handled via Set
    Set::default().run(&config, reporter)?;

    Ok(())
}
