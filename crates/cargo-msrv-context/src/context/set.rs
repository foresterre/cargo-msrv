use crate::context::{EnvironmentContext, RustReleasesContext};
use camino::Utf8PathBuf;
use version_number::Version;

#[derive(Debug)]
pub struct SetContext {
    /// MSRV to set.
    pub msrv: Version,

    pub target: SetTarget,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}

#[derive(Debug)]
pub enum SetTarget {
    Packages,
    WorkspaceRoot(Utf8PathBuf),
}
