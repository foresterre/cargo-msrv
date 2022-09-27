use std::fmt;
use std::fmt::Formatter;

// internals defining an event
pub use scope::{Marker, Scope, ScopeCounter, ScopeGenerator, SupplyScopeGenerator};

#[cfg(test)]
pub use scope::TestScopeGenerator;

// shared
pub use shared::compatibility::Compatibility;

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

// internals defining an event
mod scope;

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
    scope: Option<Scope>,
}

impl Event {
    /// Create a new unscoped event.
    #[cfg(test)]
    pub(crate) fn unscoped(message: Message) -> Self {
        Self {
            message,
            scope: None,
        }
    }

    /// Create a new scoped event.
    #[cfg(test)]
    pub(crate) fn scoped(message: Message, scope: Scope) -> Self {
        Self {
            message,
            scope: Some(scope),
        }
    }

    /// Get the context of the event.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return two copies of the event in its scoped form.
    /// The first copy will be the `start` event while the second copy will be the `end` event.
    ///
    /// Unlike unscoped events, which are sent situationally, at one point in time, scoped events
    /// mark a span during which an event took place and are intended to send an event at the start
    /// of its span, and one at the end.
    pub fn into_scoped(self, generator: &impl ScopeGenerator) -> (Self, Self) {
        let (start_scope, end_scope) = generator.generate();
        let (mut start_event, mut end_event) = (self.clone(), self);

        start_event.scope = Some(start_scope);
        end_event.scope = Some(end_scope);

        (start_event, end_event)
    }

    /// Returns `true` if this is the start of the scope, _or_, if this event has no inner scope.
    pub fn is_scope_start(&self) -> bool {
        matches!(
            self.scope,
            None | Some(Scope {
                id: _,
                marker: Marker::Start
            })
        )
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
