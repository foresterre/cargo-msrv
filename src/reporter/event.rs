use std::fmt;
use std::fmt::Formatter;

pub use action::ActionMessage;
pub use auxiliary_output::{
    AuxiliaryOutput, Destination, Item as AuxiliaryOutputItem, MsrvKind, ToolchainFileKind,
};
pub use check_toolchain::CheckToolchain;
pub use compatibility::{Compatibility, CompatibilityReport};
pub use compatibility_check_method::{CompatibilityCheckMethod, Method};
pub use fetch_index::FetchIndex;
pub use list_dep::ListDep;
pub use meta::Meta;
pub use msrv_result::MsrvResult;
pub use progress::Progress;
pub use search_method::FindMsrv;
pub use set_output::SetOutputMessage;
pub use setup_toolchain::SetupToolchain;
pub use show_output::ShowOutputMessage;
pub use termination::TerminateWithFailure;
pub use verify_output::VerifyOutput;

mod action;
mod auxiliary_output;
mod check_toolchain;
mod compatibility;
mod compatibility_check_method;
mod fetch_index;
mod list_dep;
mod meta;
mod msrv_result;
mod progress;
mod search_method;
mod set_output;
mod setup_toolchain;
mod show_output;
mod termination;
mod verify_output;

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
    Action(ActionMessage),

    // setup
    Meta(Meta),

    // get rust-releases index
    FetchIndex(FetchIndex), // todo!

    // todo: SkippedRustVersions // +reason

    // install toolchain
    SetupToolchain(SetupToolchain),

    // runner + pass/reject
    CheckToolchain(CheckToolchain),
    CompatibilityCheckMethod(CompatibilityCheckMethod),
    Compatibility(Compatibility),

    // output written by the program
    AuxiliaryOutput(AuxiliaryOutput),

    // command: find
    MsrvResult(MsrvResult),
    FindMsrv(FindMsrv),
    Progress(Progress),

    // command: verify
    // Verify
    Verify(VerifyOutput),

    // command: list
    ListDep(ListDep),

    // command: set
    SetOutput(SetOutputMessage),

    // command: show
    ShowOutput(ShowOutputMessage),

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
