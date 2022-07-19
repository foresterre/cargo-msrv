use crate::reporter::handler::TestingHandler;
use crate::Event;
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelFinalizeHandler, ChannelReporter, EventListener,
    FinishProcessing, Reporter, ReporterError,
};

pub struct TestReporter {
    reporter: ChannelReporter<Event>,
    #[allow(unused)]
    listener: ChannelEventListener<Event>,
    handler: Arc<TestingHandler>,
    finalizer: ChannelFinalizeHandler,
}

impl Default for TestReporter {
    fn default() -> Self {
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
}

impl TestReporter {
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

///  A test reporter which does absolutely nothing.
#[derive(Default)]
pub struct FakeTestReporter;

impl Reporter for FakeTestReporter {
    type Event = Event;
    type Err = ReporterError<Event>;

    fn report_event(&self, _event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        Ok(())
    }

    fn disconnect(self) -> Result<(), Self::Err> {
        Ok(())
    }
}
