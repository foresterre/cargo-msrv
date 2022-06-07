use thiserror::private::PathAsDisplay;

use crate::config::set::SetCmdConfig;
use crate::config::{ConfigBuilder, SubCommandConfig};
use crate::reporter::Reporter;
use crate::{semver, Config, ModeIntent, OutputFormat, Set, SubCommand, TResult};

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
        .mode_intent(ModeIntent::Set)
        .sub_command_config(SubCommandConfig::SetConfig(SetCmdConfig {
            msrv: version.into(),
        }))
        .build();

    Set::default().run(&config, reporter)?;

    // FIXME: report for other output formats as well
    if let OutputFormat::Human = config.output_format() {
        let manifest_path = config.ctx().manifest_path(&config)?;
        let _message = format!(
            "Written MSRV '{}' to '{}'",
            version,
            manifest_path.as_display()
        );

        // todo!
        // reporter.write_line(&message);
    }
    Ok(())
}
