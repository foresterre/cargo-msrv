//! The types which are used as values of command line options.
//!
//! These are part of the user interface, but are also used by the program itself, which is why
//! they are re-exported by the `cargo-msrv` crate.

pub mod list_msrv_variant;
pub mod log_level;
pub mod output_format;
pub mod release_source;
pub mod tracing_target_option;

pub use list_msrv_variant::{
    DIRECT_DEPS, ListMsrvVariant, ORDERED_BY_MSRV, ParseListMsrvVariantError,
};
pub use log_level::{LogLevel, ParseLogLevelError};
pub use output_format::{OutputFormat, ParseOutputFormatError};
pub use release_source::{ParseReleaseSourceError, ReleaseSource};
pub use tracing_target_option::{ParseTracingTargetOptionError, TracingTargetOption};
