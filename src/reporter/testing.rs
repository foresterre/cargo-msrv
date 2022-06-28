use crate::reporter::handler::TestingHandler;
use crate::{CargoMSRVError, Event};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelFinalizeHandler, ChannelReporter, EventListener,
    FinishProcessing, Reporter, ReporterError,
};

pub struct TestReporter {
    reporter: ChannelReporter<Event>,
    listener: ChannelEventListener<Event>,
    handler: Arc<TestingHandler>,
    finalizer: ChannelFinalizeHandler,
}

impl TestReporter {
    pub fn new() -> Self {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = ChannelReporter::new(sender);
        let listener = ChannelEventListener::new(receiver);
        let handler = Arc::new(TestingHandler::default());
        let finalizer = listener.run_handler(handler.clone());

        Self {
            reporter,
            listener,
            handler,
            finalizer,
        }
    }

    pub fn events(&self) -> Vec<Event> {
        self.handler
            .clone()
            .events()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn wait_for_events(self) -> Vec<Event> {
        self.reporter.disconnect().unwrap();
        self.finalizer.finish_processing().unwrap();

        let handler = Arc::try_unwrap(self.handler).unwrap();

        handler.unwrap_events()
    }

    pub fn reporter(&self) -> &impl Reporter<Event = Event, Err = ReporterError<Event>> {
        &self.reporter
    }
}
