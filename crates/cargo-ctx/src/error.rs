use camino::Utf8PathBuf;

/// Errors returned by [`Selection::resolve`](crate::Selection::resolve).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Running `cargo metadata` failed.
    #[error(transparent)]
    CargoMetadata(#[from] cargo_metadata::Error),

    /// The selected package is not a member of the workspace.
    #[error("Unable to select package: '{0}' is not a member of this workspace")]
    UnknownPackage(String),

    /// Cargo didn't report the default members, and no packages were selected.
    #[error(
        "Unable to find the default members of the workspace at '{path}' (likely because Cargo before 1.71 doesn't report them; use '--workspace' or '--package' to select packages)"
    )]
    DefaultMembersUnavailable { path: Utf8PathBuf },

    /// The default members could not be stored in the metadata.
    #[error("Unable to store the default members of the workspace: {0}")]
    StoreDefaultMembers(serde_json::Error),
}
