use crate::context::{DebugOutputContext, EnvironmentContext, UserOutputContext};

pub struct ShowContext {
    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}
