use crate::context::{EnvironmentContext, RustReleasesContext, SetContext, UserOutputContext};
use crate::manifest::bare_version::BareVersion;
use crate::reporter::Reporter;
use crate::{release_index, Set, SubCommand, TResult};

/// Write the MSRV to the Cargo manifest
///
/// Repurposes the Set MSRV subcommand for this action.
pub fn write_msrv(
    reporter: &impl Reporter,
    msrv: BareVersion,
    environment: EnvironmentContext,
    user_output: UserOutputContext,
    rust_releases: RustReleasesContext,
) -> TResult<()> {
    let release_source = rust_releases.release_source;

    let context = SetContext {
        msrv,
        environment,
        user_output,
        rust_releases,
    };

    let index = release_index::fetch_index(reporter, release_source).ok();
    Set::new(index.as_ref()).run(&context, reporter)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_msrv;
    use crate::config::ConfigBuilder;
    use crate::reporter::FakeTestReporter;
    use crate::{semver, SubcommandId};
    use test_dir::{DirBuilder, FileType, TestDir};

    // skip this test for now, as it became possibly flaky by adding the Set version validation.
    // The version is now checked against the release_index fetched from the config.release_source,
    // and since the release_index is now out of our control, this test is hard to fix.
    #[test]
    #[ignore]
    fn sample() {
        let tmp = TestDir::temp().create("Cargo.toml", FileType::EmptyFile);
        let manifest = tmp.path("Cargo.toml");

        std::fs::write(&manifest, "[package]").unwrap();

        let crate_path = tmp.root();
        let config = ConfigBuilder::new(SubcommandId::Find, "")
            .crate_path(Some(crate_path))
            .build();

        let fake_reporter = FakeTestReporter::default();
        let version = semver::Version::new(2, 0, 5);

        write_msrv(&config, &fake_reporter, &version).unwrap();

        let content = std::fs::read_to_string(&manifest).unwrap();
        assert_eq!(content, "[package]\nrust-version = \"2.0.5\"\n");
    }
}
