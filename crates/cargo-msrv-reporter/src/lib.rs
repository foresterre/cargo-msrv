//! User facing output of [`cargo-msrv`](https://github.com/foresterre/cargo-msrv)

#![deny(clippy::all)]
#![allow(
    clippy::uninlined_format_args,
    clippy::items_after_test_module,
    clippy::new_without_default
)]

use storyteller::{ChannelEventListener, ChannelReporter, DisconnectToken, event_channel};

use crate::event::ScopeCounter;

pub use ui::DiscardOutputHandler;
pub use ui::HumanProgressHandler;
pub use ui::JsonHandler;
pub use ui::MinimalOutputHandler;

pub use event::{
    Event, Marker, Message, Scope, ScopeGenerator, SubcommandResult, SupplyScopeGenerator,
    TerminateWithFailure, /* fixme: Needed by binary crate, how much do we want to expose here? */
};

pub mod event;
pub mod io;
pub mod typed_bool;
pub(crate) mod ui;

mod formatting;

#[cfg(any(test, feature = "testing"))]
mod testing;

#[cfg(any(test, feature = "testing"))]
pub use testing::{FakeTestReporter, TestReporterWrapper};

// Alias trait with convenience methods
// This way we don't have to specify the associated type Event
// So instead of `fn hello(reporter: &impl EventReporter<Event = Event>)`, we write:
// `fn hello(reporter: &impl Reporter)`
pub trait Reporter:
    storyteller::EventReporter<
        Event = Event,
        Err = storyteller::EventReporterError<Event>,
        DisconnectToken = DisconnectToken,
    > + SupplyScopeGenerator
{
    /// Perform a (fallible) action within the scope of the `f` closure, and report the start and
    /// end of this action.
    ///
    /// NB: returns the error type of the action (unlike `EventReporter::report_event` which returns
    /// a `Result<(), reporter::EventReporter::Err>`), so the result is flattened to the caller's
    /// error data structure.
    fn run_scoped_event<T, E>(
        &self,
        event: impl Into<Event>,
        action: impl Fn() -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<storyteller::EventReporterError<Event>>,
    {
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
    R: storyteller::EventReporter<
            Event = Event,
            Err = storyteller::EventReporterError<Event>,
            DisconnectToken = DisconnectToken,
        > + SupplyScopeGenerator
{
}

#[derive(Default)]
pub struct ReporterSetup;

impl ReporterSetup {
    pub fn create(self) -> (impl Reporter, ChannelEventListener<Event>) {
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
    type DisconnectToken = DisconnectToken;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.inner.report_event(event)
    }

    fn disconnect(self) -> Result<Self::DisconnectToken, Self::Err> {
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
    use crate::Reporter;
    use crate::TestReporterWrapper;
    use crate::event::SubcommandInit;
    use crate::event::{Marker, Message, Meta, Scope};
    use std::collections::BTreeSet;
    use storyteller::EventReporter;

    #[derive(Debug, Eq, PartialEq)]
    struct TestError;

    impl From<storyteller::EventReporterError<Event>> for TestError {
        fn from(_: storyteller::EventReporterError<Event>) -> Self {
            Self
        }
    }

    #[test]
    fn report_successful_scoped_event() {
        let reporter = TestReporterWrapper::default();
        let content = SubcommandInit::new("find");

        let out = reporter
            .get()
            .run_scoped_event(content.clone(), || Result::<bool, TestError>::Ok(true))
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
                Result::<bool, TestError>::Err(TestError)
            })
            .unwrap_err();

        let events = reporter.wait_for_events();
        let start = Event::scoped(
            Message::SubcommandInit(content.clone()),
            Scope::new(0, Marker::Start),
        );
        let end = Event::scoped(Message::SubcommandInit(content), Scope::new(0, Marker::End));

        assert_eq!(&events, &[start, end]);

        assert_eq!(out, TestError);
    }

    #[test]
    fn report_event() {
        let setup = ReporterSetup;

        let (reporter, _listener) = setup.create();

        let result = reporter.report_event(Meta::new(
            "cargo-msrv",
            "1.2.3",
            Some("aaa1111"),
            Some("x86_64-unknown-linux-gnu"),
            Some("default"),
            Some("1.91.1"),
        ));
        assert!(result.is_ok());

        let disconnect = reporter.disconnect();
        assert!(disconnect.is_ok());
    }

    #[test]
    fn scopes() {
        use std::iter;

        let setup = ReporterSetup;

        let (reporter, _listener) = setup.create();
        let generator = reporter.scope_generator();

        let set = iter::repeat_with(|| generator.generate())
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
