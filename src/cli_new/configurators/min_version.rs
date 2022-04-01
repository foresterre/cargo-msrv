use crate::cli_new::configurators::Configure;
use crate::cli_new::CargoMsrvOpts;
use crate::config::ConfigBuilder;
use std::path;
use std::path::PathBuf;

use crate::cli_new::rust_releases_opts::Edition;
use crate::{CargoMSRVError, TResult};

pub(in crate::cli_new) struct MinVersion;

impl Configure for MinVersion {
    fn configure<'c>(
        builder: ConfigBuilder<'c>,
        opts: &'c CargoMsrvOpts,
    ) -> TResult<ConfigBuilder<'c>> {
        if let Some(v) = &opts.find_opts.rust_releases_opts.min {
            let version = v.as_bare_version();
            Ok(builder.minimum_version(version))
        } else {
            configure_min_version_not_as_opt(builder, opts)
        }
    }
}

fn configure_min_version_not_as_opt<'c>(
    builder: ConfigBuilder<'c>,
    opts: &'c CargoMsrvOpts,
) -> TResult<ConfigBuilder<'c>> {
    if opts.find_opts.no_read_min_edition {
        Ok(builder)
    } else {
        let manifest = find_manifest(&builder)?;
        set_min_version_from_manifest(builder, &manifest)
    }
}

// TODO{foresterre}: finding Cargo manifest should not be task of configurator fn
//  And probably, we'll want to read the manifest at most once, instead of here and elsewhere during
//  the program execution.
fn find_manifest(builder: &ConfigBuilder) -> TResult<PathBuf> {
    use crate::errors::IoErrorSource;

    let crate_folder = if let Some(path) = builder.get_crate_path() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir().map_err(|error| CargoMSRVError::Io {
            error,
            source: IoErrorSource::CurrentDir,
        })
    }?;

    Ok(crate_folder.join("Cargo.toml"))
}

// TODO{foresterre}: reading and parsing of manifest should not be task of configurator fn
fn set_min_version_from_manifest<'c>(
    builder: ConfigBuilder<'c>,
    cargo_toml: &path::Path,
) -> TResult<ConfigBuilder<'c>> {
    use crate::errors::IoErrorSource;
    use toml_edit::Document;
    use toml_edit::Item;

    let contents = std::fs::read_to_string(cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml.to_path_buf()),
    })?;
    let document = contents
        .parse::<Document>()
        .map_err(CargoMSRVError::ParseToml)?;

    if let Some(edition) = document
        .as_table()
        .get("package")
        .and_then(Item::as_table)
        .and_then(|package_table| package_table.get("edition"))
        .and_then(Item::as_str)
    {
        let edition = edition.parse::<Edition>()?;
        Ok(builder.minimum_version(edition.as_bare_version()))
    } else {
        Ok(builder)
    }
}
