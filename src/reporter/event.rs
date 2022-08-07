use std::fmt;
use std::fmt::Formatter;

// shared
pub use shared::{compatibility::Compatibility, compatibility::CompatibilityReport};

// events
pub use auxiliary_output::{
    AuxiliaryOutput, Destination, Item as AuxiliaryOutputItem, MsrvKind, ToolchainFileKind,
};
pub use check_method::{CheckMethod, Method};
pub use check_result::CheckResult;
pub use check_toolchain::CheckToolchain;
pub use fetch_index::FetchIndex;
pub use meta::Meta;
pub use progress::Progress;
pub use search_method::FindMsrv;
pub use setup_toolchain::SetupToolchain;
pub use subcommand_init::SubcommandInit;
pub use subcommand_result::SubcommandResult;
pub use termination::TerminateWithFailure;

// types
pub use types::{
    find_result::FindResult, list_result::ListResult, set_result::SetResult,
    show_result::ShowResult, verify_result::VerifyResult,
};

// shared
mod shared;

// items which themselves aren't events, but can be used to compose an event
mod types;

// specific events
mod auxiliary_output;
mod check_method;
mod check_result;
mod check_toolchain;
mod fetch_index;
mod meta;
mod progress;
mod search_method;
mod setup_toolchain;
mod subcommand_init;
mod subcommand_result;
mod termination;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Event {
    #[serde(flatten)]
    message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<EventScope>,
}

impl Event {
    #[cfg(test)]
    pub(crate) fn new(message: Message) -> Self {
        Self {
            message,
            scope: None,
        }
    }

    pub fn message(&self) -> &Message {
        &self.message
    }

    pub(crate) fn with_scope(&self, scope: EventScope) -> Self {
        let mut cloned = self.clone();
        cloned.scope = Some(scope);
        cloned
    }

    /// Returns `true` if this is the start of the scope, _or_, if this event has no inner scope.
    pub fn is_scope_start(&self) -> bool {
        matches!(self.scope, None | Some(EventScope::Start))
    }
}

/// Messages are a kind of event which report the state of this program to the user
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Message {
    // setup
    Meta(Meta),

    // get rust-releases index
    FetchIndex(FetchIndex), // todo!

    // todo: ReducedSearchSpace
    //       with: reason, reduction

    // runner, setup of toolchain, method, result
    CheckToolchain(CheckToolchain),
    SetupToolchain(SetupToolchain),
    CheckMethod(CheckMethod),
    CheckResult(CheckResult),

    // output written by the program
    AuxiliaryOutput(AuxiliaryOutput),

    // progression events for command: find
    FindMsrv(FindMsrv),
    Progress(Progress),

    // command init and final result
    SubcommandInit(SubcommandInit),
    SubcommandResult(SubcommandResult),

    // Termination, for example when caused by an unrecoverable error
    TerminateWithFailure(TerminateWithFailure),
}

impl From<Message> for Event {
    fn from(message: Message) -> Self {
        Event {
            message,
            scope: None,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventScope {
    Start,
    End,
}
