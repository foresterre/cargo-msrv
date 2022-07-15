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

#[cfg(test)]
mod tests {
    use super::write_msrv;
    use crate::config::ConfigBuilder;
    use crate::reporter::FakeTestReporter;
    use crate::{semver, Action};
    use test_dir::{DirBuilder, FileType, TestDir};

    #[test]
    fn sample() {
        let tmp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);
        let manifest = tmp.path("Cargo.toml");

        std::fs::write(&manifest, "[package]").unwrap();

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(Action::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(2, 0, 5);

        write_msrv(&config, &fake_reporter, &version).unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }
}
