use crate::error::IoErrorSource;
use crate::paths::crate_root_folder;
use crate::reporter::event::{
    AuxiliaryOutput, Destination, Item as AuxiliaryOutputItem, ToolchainFileKind,
};
use crate::reporter::Reporter;
use crate::{semver, CargoMSRVError, Config, TResult};

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

pub fn write_toolchain_file(
    config: &Config,
    reporter: &impl Reporter,
    stable_version: &semver::Version,
) -> TResult<()> {
    let path_prefix = crate_root_folder(config)?;

    // todo refactor to be more complete
    // - consider: replace toolchain file type with same type
    // - consider: replace toolchain file with 'best' rust-toolchain(.toml) format variant available
    // - consider: do overwrite the rust-toolchain file, since it was requested by the user by providing
    //             the flag
    //
    // check if the rust-toolchain(.toml) file already exists
    #[allow(clippy::if_same_then_else)]
    if path_prefix.join(TOOLCHAIN_FILE).exists() {
        // todo!: replace with events
        // eprintln!(
        //     "Not writing toolchain file, '{}' already exists",
        //     TOOLCHAIN_FILE
        // );
        return Ok(());
    } else if path_prefix.join(TOOLCHAIN_FILE_TOML).exists() {
        // todo!: replace with events
        // eprintln!(
        //     "Not writing toolchain file, '{}' already exists",
        //     TOOLCHAIN_FILE_TOML
        // );
        return Ok(());
    }

    let path = path_prefix.join(TOOLCHAIN_FILE);
    let content = format!(
        r#"[toolchain]
channel = "{}"
"#,
        stable_version
    );

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
