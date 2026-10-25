use crate::context::EnvironmentContext;
use crate::types::ListMsrvVariant;
use cargo_util_schemas::core::PackageIdSpec;

#[derive(Debug)]
pub struct ListContext {
    /// The type of output expected by the user
    pub variant: ListMsrvVariant,

    /// The packages to highlight in the MSRV table
    pub highlight: Vec<PackageIdSpec>,

    /// Resolved environment options
    pub environment: EnvironmentContext,
}
