use storyteller::{event_channel, ChannelEventListener, ChannelReporter, EventListener};

use crate::reporter::event::EventScope;
use crate::TResult;

pub use handler::DiscardOutputHandler;
pub use handler::HumanProgressHandler;
pub use handler::JsonHandler;

pub use event::{
    Event, Message,
    TerminateWithFailure, /* fixme: Needed by binary crate, how much do we want to expose here? */
};

pub(crate) mod event;
pub(crate) mod handler;

#[cfg(test)]
mod testing;

#[cfg(test)]
pub use testing::{FakeTestReporter, TestReporter};

// Alias trait with convenience methods
// This way we don't have to specify the associated type Event
// So instead of `fn hello(reporter: &impl Reporter<Event = Event>)`, we write:
// `fn hello(reporter: &impl Reporter)`
pub trait Reporter:
    storyteller::Reporter<Event = Event, Err = storyteller::ReporterError<Event>>
{
    /// Perform a (fallible) action within the scope of the `f` closure, and report the start and
    /// end of this action.
    ///
    /// NB: returns a `crate::TResult` (unlike the regular `report_event` which returns
    /// a `Result<(), reporter::Reporter::Err>`), so the result is flattened to `cargo-msrv's`
    /// error data structure.
    fn run_scoped_event<T>(
        &self,
        event: impl Into<Event>,
        f: impl Fn() -> TResult<T>,
    ) -> TResult<T> {
        let event = event.into();

        // Report that the action is starting
        let begin = event.with_scope(EventScope::Start);
        self.report_event(begin)?;

        // Perform the action
        let result = f();

        // Report that the action has finished
        let end = event.with_scope(EventScope::End);
        self.report_event(end)?;

        result
    }
}
impl<T> Reporter for T where
    T: storyteller::Reporter<Event = Event, Err = storyteller::ReporterError<Event>>
{
}

#[derive(Default)]
pub struct ReporterSetup;

impl ReporterSetup {
    pub fn create(self) -> (impl Reporter, impl EventListener<Event = Event>) {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = ChannelReporter::new(sender);
        let listener = ChannelEventListener::new(receiver);

        (reporter, listener)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use crate::{Action, ActionMessage, CargoMSRVError, Reporter};

    #[test]
    fn report_successful_scoped_event() {
        let reporter = TestReporter::default();
        let event = ActionMessage::new(Action::Find);

        let out = reporter
            .reporter()
            .run_scoped_event(event.clone(), || TResult::<bool>::Ok(true))
            .unwrap();

        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[
                Event::new(Message::Action(event.clone())).with_scope(EventScope::Start),
                Event::new(Message::Action(event)).with_scope(EventScope::End)
            ]
        );

        assert!(out);
    }

    #[test]
    fn report_failed_scoped_event() {
        let reporter = TestReporter::default();
        let event = ActionMessage::new(Action::Find);

        let out = reporter
            .reporter()
            .run_scoped_event(event.clone(), || {
                TResult::<bool>::Err(CargoMSRVError::Storyteller)
            })
            .unwrap_err();

        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[
                Event::new(Message::Action(event.clone())).with_scope(EventScope::Start),
                Event::new(Message::Action(event)).with_scope(EventScope::End)
            ]
        );

        assert!(matches!(out, CargoMSRVError::Storyteller));
    }
}
