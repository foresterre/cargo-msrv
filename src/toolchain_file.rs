use crate::errors::IoErrorSource;
use crate::paths::crate_root_folder;
use crate::{semver, CargoMSRVError, Config, TResult};

const TOOLCHAIN_FILE: &str = "rust-toolchain";
const TOOLCHAIN_FILE_TOML: &str = "rust-toolchain.toml";

pub fn write_toolchain_file(config: &Config, stable_version: &semver::Version) -> TResult<()> {
    let path_prefix = crate_root_folder(config)?;

    // check if the rust-toolchain(.toml) file already exists
    if path_prefix.join(TOOLCHAIN_FILE).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE
        );
        return Ok(());
    } else if path_prefix.join(TOOLCHAIN_FILE_TOML).exists() {
        eprintln!(
            "Not writing toolchain file, '{}' already exists",
            TOOLCHAIN_FILE_TOML
        );
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
    eprintln!("Written toolchain file to '{}'", &path.display());

    Ok(())
}
