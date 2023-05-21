use crate::reporter::event::{ScopeCounter, SupplyScopeGenerator, TestScopeGenerator};
use crate::reporter::handler::TestingHandler;
use crate::{Event, Reporter};
use std::sync::Arc;
use storyteller::{
    event_channel, ChannelEventListener, ChannelFinalizeHandler, ChannelReporter, EventListener,
    EventReporter, EventReporterError, FinishProcessing,
};

pub struct TestReporterWrapper {
    reporter: TestReporter,
    #[allow(unused)]
    listener: ChannelEventListener<Event>,
    handler: Arc<TestingHandler>,
    finalizer: ChannelFinalizeHandler,
}

impl Default for TestReporterWrapper {
    fn default() -> Self {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = TestReporter::new(ChannelReporter::new(sender));
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

impl TestReporterWrapper {
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

    pub fn get(&self) -> &impl Reporter {
        &self.reporter
    }
}

struct TestReporter {
    inner: ChannelReporter<Event>,
    scope_generator: ScopeCounter,
}

impl TestReporter {
    pub fn new(reporter: ChannelReporter<Event>) -> Self {
        Self {
            inner: reporter,
            scope_generator: ScopeCounter::new(),
        }
    }
}

impl EventReporter for TestReporter {
    type Event = Event;
    type Err = storyteller::EventReporterError<Event>;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.inner.report_event(event)
    }

    fn disconnect(self) -> Result<(), Self::Err> {
        self.inner.disconnect()
    }
}

impl SupplyScopeGenerator for TestReporter {
    type ScopeGen = ScopeCounter;

    fn scope_generator(&self) -> &Self::ScopeGen {
        &self.scope_generator
    }
}

///  A test reporter which does absolutely nothing.
#[derive(Default)]
pub struct FakeTestReporter(TestScopeGenerator);

impl EventReporter for FakeTestReporter {
    type Event = Event;
    type Err = EventReporterError<Event>;

    fn report_event(&self, _event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        Ok(())
    }

    fn disconnect(self) -> Result<(), Self::Err> {
        Ok(())
    }
}

impl SupplyScopeGenerator for FakeTestReporter {
    type ScopeGen = TestScopeGenerator;

    fn scope_generator(&self) -> &Self::ScopeGen {
        &self.0
    }
}
