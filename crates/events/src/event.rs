use crate::{
    ActionMessage, AuxiliaryOutput, CheckToolchain, Compatibility, CompatibilityCheckMethod,
    FetchIndex, FindMSRV, ListDep, Meta, MsrvResult, Progress, SetOutputMessage, SetupToolchain,
    ShowOutputMessage, TerminateWithFailure,
};
use std::fmt;
use std::fmt::Formatter;

pub mod action;
pub mod auxiliary_output;
pub mod compatibility;
pub mod compatibility_check_method;
pub mod fetch_index;
pub mod list_dep;
pub mod meta;
pub mod msrv_result;
pub mod new_compatibility_check;
pub mod progress;
pub mod search_method;
pub mod set_output;
pub mod setup_toolchain;
pub mod show_output;
pub mod termination;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Event {
    #[serde(flatten)]
    message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<EventScope>,
}

impl Event {
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
    NewCompatibilityCheck(CheckToolchain),
    CompatibilityCheckMethod(CompatibilityCheckMethod),
    Compatibility(Compatibility),

    // output written by the program
    AuxiliaryOutput(AuxiliaryOutput),

    // command: find
    MsrvResult(MsrvResult),
    FindMSRV(FindMSRV),
    Progress(Progress),

    // command: verify
    // Verify

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
