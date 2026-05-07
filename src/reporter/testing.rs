use crate::reporter::event::{ScopeCounter, SupplyScopeGenerator, TestScopeGenerator};
use crate::reporter::ui::TestingHandler;
use crate::{Event, Reporter};
use std::sync::Arc;
use storyteller::{
    ChannelEventListener, ChannelHandlerGuard, ChannelReporter, DisconnectToken, EventListener,
    EventReporter, EventReporterError, HandlerGuard, event_channel,
};

pub struct TestReporterWrapper {
    reporter: Option<TestReporter>,
    #[allow(unused)]
    listener: ChannelEventListener<Event>,
    handler: Option<Arc<TestingHandler>>,
    guard: Option<ChannelHandlerGuard>,
}

impl Default for TestReporterWrapper {
    fn default() -> Self {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = TestReporter::new(ChannelReporter::new(sender));
        let listener = ChannelEventListener::new(receiver);
        let handler = Arc::new(TestingHandler::default());
        let guard = listener.run_handler(handler.clone());

        Self {
            reporter: Some(reporter),
            listener,
            handler: Some(handler),
            guard: Some(guard),
        }
    }
}

impl Drop for TestReporterWrapper {
    fn drop(&mut self) {
        if let (Some(reporter), Some(guard)) = (self.reporter.take(), self.guard.take()) {
            let token = reporter.disconnect().unwrap();
            guard.join(token).unwrap();
        }
    }
}

impl TestReporterWrapper {
    pub fn events(&self) -> Vec<Event> {
        self.handler
            .as_ref()
            .unwrap()
            .clone()
            .events()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn wait_for_events(mut self) -> Vec<Event> {
        let token = self.reporter.take().unwrap().disconnect().unwrap();
        self.guard.take().unwrap().join(token).unwrap();

        let handler = Arc::try_unwrap(self.handler.take().unwrap()).unwrap();

        handler.unwrap_events()
    }

    pub fn get(&self) -> &impl Reporter {
        self.reporter.as_ref().unwrap()
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
    type DisconnectToken = DisconnectToken;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.inner.report_event(event)
    }

    fn disconnect(self) -> Result<Self::DisconnectToken, Self::Err> {
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
    type DisconnectToken = DisconnectToken;

    fn report_event(&self, _event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        Ok(())
    }

    fn disconnect(self) -> Result<Self::DisconnectToken, Self::Err> {
        let (sender, _receiver) = event_channel::<Event>();
        ChannelReporter::new(sender).disconnect()
    }
}

impl SupplyScopeGenerator for FakeTestReporter {
    type ScopeGen = TestScopeGenerator;

    fn scope_generator(&self) -> &Self::ScopeGen {
        &self.0
    }
}
