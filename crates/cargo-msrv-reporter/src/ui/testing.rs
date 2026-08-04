use crate::Event;
use std::sync::{Arc, Mutex, MutexGuard};
use storyteller::EventHandler;

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
