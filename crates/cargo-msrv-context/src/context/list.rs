use crate::context::EnvironmentContext;
use crate::types::ListMsrvVariant;

#[derive(Debug)]
pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}
