use crate::combinators::ThenSome;
use crate::error::IoErrorSource;
use crate::reporter::event::{
    AuxiliaryOutput, Destination, Item as AuxiliaryOutputItem, ToolchainFileKind,
};
use crate::reporter::Reporter;
use crate::{semver, CargoMSRVError, Config, TResult};
use std::fmt;
use std::path::{Path, PathBuf};

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

// - consider: replace toolchain file with 'best' rust-toolchain(.toml) format variant available
// - consider: support old toolchain-file format
// - consider: do not simply override, also support components, targets, profile
//     - in reverse: use the values from rust-toolchain file to auto configure config
pub fn write_toolchain_file(
    config: &Config,
    reporter: &impl Reporter,
    stable_version: &semver::Version,
) -> TResult<()> {
    let path_prefix = config.context().crate_root_path()?;
    let path = toolchain_file(path_prefix);
    let content = format_toolchain_file(stable_version);

    std::fs::write(&path, content).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::WriteFile(path.clone()),
    })?;

    reporter.report_event(AuxiliaryOutput::new(
        Destination::File(path),
        AuxiliaryOutputItem::toolchain_file(ToolchainFileKind::Toml),
    ))?;

    Ok(())
}

/// Determine whether we should use a .toml extension or no extension for the rust-toolchain file.
fn toolchain_file(path: &Path) -> PathBuf {
    fn without_extension(path: &Path) -> Option<PathBuf> {
        let file = path.join(TOOLCHAIN_FILE);
        ThenSome::then_some(file.exists(), file)
    }

    fn with_extension(path: &Path) -> Option<PathBuf> {
        let file = path.join(TOOLCHAIN_FILE_TOML);
        ThenSome::then_some(file.exists(), file)
    }

    // Without extension variant has precedence over with extension variant
    // https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file
    without_extension(path)
        .or_else(|| with_extension(path))
        .unwrap_or_else(|| path.join(TOOLCHAIN_FILE))
}

fn format_toolchain_file<D>(channel: &D) -> String
where
    D: fmt::Display,
{
    format!(
        r#"[toolchain]
channel = "{}"
"#,
        channel
    )
}
