use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{Metadata, PackageId};
use std::panic;

/// See [Cargo Book: package selection](https://doc.rust-lang.org/nightly/cargo/commands/cargo-build.html#package-selection)
pub trait PackageSelection {
    type Error;

    fn select_packages(&self, select: Select) -> Result<(), Self::Error>;
}

pub struct CargoMetadataPackageSelection {
    root_manifest_path: Option<Utf8PathBuf>,
}

impl PackageSelection for CargoMetadataPackageSelection {
    type Error = Error;

    fn select_packages(
        &self,
        select: Select,
    ) -> Result<impl IntoIterator<Item = PackageId>, Self::Error> {
        let mut cargo_metadata = cargo_metadata::MetadataCommand::new();

        if let Some(path) = self.root_manifest_path.as_deref() {
            cargo_metadata.manifest_path(path);
        }

        let metadata = cargo_metadata.exec().map_err(|_| Error::CargoMetadata)?;

        if selected_packages.is_empty() {
            if workspace_flag {
                return Ok(metadata.workspace_members);
            } else if let Some(default_members) = workspace_default_members(&metadata) {
                return Ok(default_members);
            } else if let Some(root_package) = metadata.root_package().as_deref() {
                return Ok(vec![root_package.id]);
            } else {
                return Err(Error::NoSelectablePackage);
            }
        }

        Ok(metadata
            .workspace_packages()
            .into_iter()
            .filter(|&pkg| {
                selected_packages
                    .iter()
                    .any(|&selected| pkg.name == selected)
            })
            .collect())
    }
}

pub enum Select {
    WorkspaceFlag {
        select: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    },
    RootPackageOrVirtual {
        select: Option<Vec<String>>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to execute cargo-metadata")]
    CargoMetadata,

    #[error("Unable to select package")]
    NoSelectablePackage,
}

pub fn workspace_default_members(metadata: &Metadata) -> Option<Vec<PackageId>> {
    panic::catch_unwind(|| metadata.workspace_default_members.into_iter().collect()).ok()
}
