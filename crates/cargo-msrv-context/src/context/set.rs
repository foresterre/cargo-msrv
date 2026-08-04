use crate::context::{EnvironmentContext, RustReleasesContext};
use cargo_msrv_types::BareVersion;

#[derive(Debug)]
pub struct SetContext {
    /// MSRV to set.
    pub msrv: BareVersion,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}
