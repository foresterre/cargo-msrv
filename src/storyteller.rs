// todo: rename-to reporter.rs

use storyteller::{
    disconnect_channel, event_channel, ChannelEventListener, ChannelReporter, DisconnectReceiver,
    DisconnectSender, EventListener, Reporter,
};

pub(crate) mod event;
pub(crate) mod handler;

type Event = event::Event;

struct StorytellerSetup {
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

    pub fn create_channels<Event: Send + 'static>(
        self,
    ) -> (
        impl Reporter<Event = Event>,
        impl EventListener<Event = Event>,
    ) {
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
    use super::handler::JsonHandler;
    use crate::storyteller::event::progress::Progression;
    use crate::storyteller::{Event, StorytellerSetup};
    use storyteller::{EventListener, Reporter};

    #[test]
    fn setup() {
        fn report(reporter: &impl Reporter<Event = Event>, event: Event) -> Result<(), String> {
            reporter
                .report_event(event)
                .map_err(|_| "Failed to report event".to_string())
        }

        let setup = StorytellerSetup::new();
        let (reporter, listener) = setup.create_channels::<Event>();

        let handler = JsonHandler::stderr();
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
