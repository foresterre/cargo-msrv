use crate::sub_command::{Find, List, Set, Show, Verify};
use cargo_msrv_reporter::Event;

pub use compatibility::{Level, Phase};
pub use running::Running;
pub use scope::ScopeStack;
pub use setup::Setup;
pub use summary::{Outcome, Summary, SummaryRow};

mod compatibility;
mod running;
mod scope;
mod setup;
mod summary;

pub enum State {
    Setup(Setup),
    Find(Phase<Find>),
    Verify(Phase<Verify>),
    List(Running<List>),
    Set(Running<Set>),
    Show(Running<Show>),
    Done(Done),
}

pub struct Done;

pub enum Next<S> {
    Continue(S),
    Done(Done),
}

pub struct Unexpected;

impl State {
    pub fn step(self, event: &Event) -> State {
        match self {
            State::Setup(setup) => setup.step(event),
            State::Find(phase) => phase.step(event).map(State::Find),
            State::Verify(phase) => phase.step(event).map(State::Verify),
            State::List(running) => running.step(event).map(State::List),
            State::Set(running) => running.step(event).map(State::Set),
            State::Show(running) => running.step(event).map(State::Show),
            State::Done(done) => State::Done(unexpected_event(done, event)),
        }
    }

    pub fn finish(self) -> Done {
        match self {
            State::Setup(_) => Done,
            State::Find(phase) => phase.finish(),
            State::Verify(phase) => phase.finish(),
            State::List(running) => running.finish(),
            State::Set(running) => running.finish(),
            State::Show(running) => running.finish(),
            State::Done(done) => done,
        }
    }
}

impl<S> Next<S> {
    pub fn map(self, f: impl FnOnce(S) -> State) -> State {
        match self {
            Next::Continue(state) => f(state),
            Next::Done(done) => State::Done(done),
        }
    }
}

pub fn unexpected_event<S>(state: S, event: &Event) -> S {
    tracing::warn!(
        state = std::any::type_name::<S>(),
        ?event,
        "unexpected event"
    );

    state
}
