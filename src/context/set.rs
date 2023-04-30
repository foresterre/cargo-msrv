use crate::context::{DebugOutputContext, EnvironmentContext, UserOutputContext};
use crate::manifest::bare_version::BareVersion;

pub struct SetContext {
    /// MSRV to set.
    pub msrv: BareVersion,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}
