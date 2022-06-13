use storyteller::{
    disconnect_channel, event_channel, ChannelEventListener, ChannelReporter, DisconnectReceiver,
    DisconnectSender, EventListener,
};

use crate::reporter::event::EventScope;
use crate::TResult;

pub use handler::DiscardOutputHandler;
pub use handler::HumanProgressHandler;
pub use handler::JsonHandler;

pub use event::{
    Event,
    TerminateWithFailure, /* fixme: Needed by binary crate, how much do we want to expose here? */
};

pub(crate) mod event;
pub(crate) mod handler;

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

pub struct StorytellerSetup {
    disconnect_sender: DisconnectSender,
    disconnect_receiver: DisconnectReceiver,
}

impl StorytellerSetup {
    pub fn new() -> Self {
        let (disconnect_sender, disconnect_receiver) = disconnect_channel();

        Self {
            disconnect_sender,
            disconnect_receiver,
        }
    }

    pub fn create_channels(self) -> (impl Reporter, impl EventListener<Event = Event>) {
        let (sender, receiver) = event_channel::<Event>();
        let Self {
            disconnect_sender,
            disconnect_receiver,
        } = self;

        let reporter = ChannelReporter::new(sender, disconnect_receiver);
        let listener = ChannelEventListener::new(receiver, disconnect_sender);

        (reporter, listener)
    }
}

#[cfg(test)]
mod tests {
    use storyteller::{EventListener, Reporter};

    use crate::reporter::event::progress::Progression;
    use crate::reporter::handler::HumanProgressHandler;
    use crate::reporter::{Event, StorytellerSetup};
    use crate::{Reporter, Reporter};

    use super::handler::JsonHandler;

    #[test]
    fn setup() {
        fn report(reporter: &impl Reporter, event: Event) -> Result<(), String> {
            reporter
                .report_event(event)
                .map_err(|_| "Failed to report event".to_string())
        }

        let setup = StorytellerSetup::new();
        let (reporter, listener): (impl Reporter, _) = setup.create_channels();

        let handler = HumanProgressHandler::new();
        // let handler = JsonHandler::stderr();
        listener.run_handler(handler);

        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();
        report(&reporter, Event::Progress(Progression::new(100))).unwrap();

        let d = reporter.disconnect();

        assert!(d.is_ok());
    }
}
