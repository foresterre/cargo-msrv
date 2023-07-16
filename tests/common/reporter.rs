// Copied from src/reporter.rs for integration and end-to-end testing
// To do: Merge them back together in a testing dev-dep crate:
// * Requires: traits for Check, Output etc. to be separated to a library crate as
//      well.

use cargo_msrv::reporter::{Event, Marker, Reporter, Scope, ScopeGenerator, SupplyScopeGenerator};
use std::sync::{Arc, Mutex, MutexGuard};
use storyteller::{
    event_channel, ChannelEventListener, ChannelFinalizeHandler, ChannelReporter, EventHandler,
    EventListener, EventReporter, EventReporterError, EventSender, FinishProcessing,
};

pub struct IntegrationTestReporter {
    inner: ChannelReporter<Event>,
    id_gen: IntegrationTestScopeGenerator,
}

impl EventReporter for IntegrationTestReporter {
    type Event = Event;
    type Err = EventReporterError<Event>;

    fn report_event(&self, event: impl Into<Self::Event>) -> Result<(), Self::Err> {
        self.inner.report_event(event)
    }

    fn disconnect(self) -> Result<(), Self::Err> {
        self.inner.disconnect()
    }
}

impl IntegrationTestReporter {
    pub fn new(sender: EventSender<Event>) -> Self {
        Self {
            inner: ChannelReporter::new(sender),
            id_gen: IntegrationTestScopeGenerator,
        }
    }
}

impl SupplyScopeGenerator for IntegrationTestReporter {
    type ScopeGen = IntegrationTestScopeGenerator;

    fn scope_generator(&self) -> &Self::ScopeGen {
        &self.id_gen
    }
}

#[derive(Default)]
pub struct IntegrationTestScopeGenerator;

impl ScopeGenerator for IntegrationTestScopeGenerator {
    fn generate(&self) -> (Scope, Scope) {
        let id = 0;

        (Scope::new(id, Marker::Start), Scope::new(id, Marker::End))
    }
}

pub struct EventTestDevice {
    reporter: IntegrationTestReporter,
    #[allow(unused)]
    listener: ChannelEventListener<Event>,
    handler: Arc<TestingHandler>,
    finalizer: ChannelFinalizeHandler,
}

impl Default for EventTestDevice {
    fn default() -> Self {
        let (sender, receiver) = event_channel::<Event>();

        let reporter = IntegrationTestReporter::new(sender);
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

impl EventTestDevice {
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

    pub fn reporter(&self) -> &impl Reporter {
        &self.reporter
    }
}

#[derive(Debug)]
pub struct TestingHandler {
    event_log: Arc<Mutex<Vec<Event>>>,
}

impl Default for TestingHandler {
    fn default() -> Self {
        Self {
            event_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Clone for TestingHandler {
    fn clone(&self) -> Self {
        Self {
            event_log: self.event_log.clone(),
        }
    }
}

impl TestingHandler {
    pub fn events(&self) -> MutexGuard<'_, Vec<Event>> {
        self.event_log.lock().unwrap()
    }

    pub fn unwrap_events(self) -> Vec<Event> {
        let mutex = Arc::try_unwrap(self.event_log).unwrap();
        mutex.into_inner().unwrap()
    }
}

impl EventHandler for TestingHandler {
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        self.event_log.lock().unwrap().push(event);
    }
}
