//! The `events` package contains typed events which may be reported by the reporter implementation
//! in `cargo-msrv`. It depends on the `types` package for commonly used definitions.

// event types

pub use event::{Event, EventScope, Message};

// specific events

pub use event::action::ActionMessage;
pub use event::auxiliary_output::{
    AuxiliaryOutput, Destination, Item, MSRVKind, ToolchainFileKind,
};
pub use event::compatibility::{Compatibility, CompatibilityReport};
pub use event::compatibility_check_method::{CompatibilityCheckMethod, Method};
pub use event::fetch_index::FetchIndex;
pub use event::list_dep::ListDep;
pub use event::meta::Meta;
pub use event::msrv_result::MsrvResult;
pub use event::new_compatibility_check::CheckToolchain;
pub use event::progress::Progress;
pub use event::search_method::FindMSRV;
pub use event::set_output::SetOutputMessage;
pub use event::setup_toolchain::SetupToolchain;
pub use event::show_output::ShowOutputMessage;
pub use event::termination::TerminateWithFailure;

mod event;
