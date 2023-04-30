use storyteller::{event_channel, ChannelEventListener, ChannelReporter, EventListener};

use crate::reporter::event::ScopeCounter;
use crate::TResult;

pub use handler::DiscardOutputHandler;
pub use handler::HumanProgressHandler;
pub use handler::JsonHandler;
pub use handler::MinimalOutputHandler;

pub use event::{
    Event, Marker, Message, Scope, ScopeGenerator, SubcommandResult, SupplyScopeGenerator,
    TerminateWithFailure, /* fixme: Needed by binary crate, how much do we want to expose here? */
};

pub(crate) mod event;
pub(crate) mod handler;

#[cfg(test)]
mod testing;

#[cfg(test)]
pub use testing::{FakeTestReporter, TestReporterWrapper};

// Alias trait with convenience methods
// This way we don't have to specify the associated type Event
// So instead of `fn hello(reporter: &impl EventReporter<Event = Event>)`, we write:
// `fn hello(reporter: &impl Reporter)`
pub trait Reporter:
    storyteller::EventReporter<Event = Event, Err = storyteller::EventReporterError<Event>>
    + SupplyScopeGenerator
{
    /// Perform a (fallible) action within the scope of the `f` closure, and report the start and
    /// end of this action.
    ///
    /// NB: returns a `crate::TResult` (unlike `EventReporter::report_event` which returns
    /// a `Result<(), reporter::EventReporter::Err>`), so the result is flattened to `cargo-msrv's`
    /// error data structure.
    fn run_scoped_event<T>(
        &self,
        event: impl Into<Event>,
        action: impl Fn() -> TResult<T>,
    ) -> TResult<T> {
        let event = event.into();
        let (start_event, end_event) = event.into_scoped(self.scope_generator());

        // Report that the action is starting
        self.report_event(start_event)?;

        // Perform the action contained by the scope
        let result = action();

        // Report that the action has finished
        self.report_event(end_event)?;

        result
    }
}

impl<R> Reporter for R where
    R: storyteller::EventReporter<Event = Event, Err = storyteller::EventReporterError<Event>>
        + SupplyScopeGenerator
{
}

#[derive(Default)]
pub struct ReporterSetup;

impl ReporterSetup {
    pub fn create(self) -> (impl Reporter, impl EventListener<Event = Event>) {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = MainReporter::new(ChannelReporter::new(sender));
        let listener = ChannelEventListener::new(receiver);

        (reporter, listener)
    }
}

struct MainReporter {
    inner: ChannelReporter<Event>,
    scope_generator: ScopeCounter,
}

impl MainReporter {
    pub fn new(reporter: ChannelReporter<Event>) -> Self {
        Self {
            inner: reporter,
            scope_generator: ScopeCounter::new(),
        }
    }
}

impl storyteller::EventReporter for MainReporter {
    type Event = Event;
    type Err = storyteller::EventReporterError<Event>;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.inner.report_event(event)
    }

    fn disconnect(self) -> Result<(), Self::Err> {
        self.inner.disconnect()
    }
}

impl SupplyScopeGenerator for MainReporter {
    type ScopeGen = ScopeCounter;

    fn scope_generator(&self) -> &Self::ScopeGen {
        &self.scope_generator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::{Marker, Message, Meta, Scope};
    use crate::reporter::TestReporterWrapper;
    use crate::{CargoMSRVError, Reporter, SubcommandInit};
    use std::collections::BTreeSet;
    use storyteller::EventReporter;

    #[test]
    fn report_successful_scoped_event() {
        let reporter = TestReporterWrapper::default();
        let content = SubcommandInit::new("find");

        let out = reporter
            .get()
            .run_scoped_event(content.clone(), || TResult::<bool>::Ok(true))
            .unwrap();

        let events = reporter.wait_for_events();

        let start = Event::scoped(
            Message::SubcommandInit(content.clone()),
            Scope::new(0, Marker::Start),
        );
        let end = Event::scoped(Message::SubcommandInit(content), Scope::new(0, Marker::End));

        assert_eq!(&events, &[start, end]);

        assert!(out);
    }

    #[test]
    fn report_failed_scoped_event() {
        let reporter = TestReporterWrapper::default();
        let content = SubcommandInit::new("find");

        let out = reporter
            .get()
            .run_scoped_event(content.clone(), || {
                TResult::<bool>::Err(CargoMSRVError::Storyteller)
            })
            .unwrap_err();

        let events = reporter.wait_for_events();
        let start = Event::scoped(
            Message::SubcommandInit(content.clone()),
            Scope::new(0, Marker::Start),
        );
        let end = Event::scoped(Message::SubcommandInit(content), Scope::new(0, Marker::End));

        assert_eq!(&events, &[start, end]);

        assert!(matches!(out, CargoMSRVError::Storyteller));
    }

    #[test]
    fn report_event() {
        let setup = ReporterSetup::default();

        let (reporter, _listener) = setup.create();

        let result = reporter.report_event(Meta::default());
        assert!(result.is_ok());

        let disconnect = reporter.disconnect();
        assert!(disconnect.is_ok());
    }

    #[test]
    fn scopes() {
        use std::iter;

        let setup = ReporterSetup::default();

        let (reporter, _listener) = setup.create();
        let gen = reporter.scope_generator();

        let set = iter::repeat_with(|| gen.generate())
            .take(1000)
            .map(|(start, end)| {
                assert!(start.is_start());
                assert!(!end.is_start());

                assert_eq!(start.id, end.id);

                start.id
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(set.len(), 1000);
    }
}
