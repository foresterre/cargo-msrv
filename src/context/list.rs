use crate::config::list::ListMsrvVariant;
use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};

pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}
