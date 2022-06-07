use crate::Event;
use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;
use storyteller::EventHandler;

pub struct TestingHandler {
    event_log: Rc<RefCell<Vec<Event>>>,
}

impl Default for TestingHandler {
    fn default() -> Self {
        Self {
            event_log: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl TestingHandler {
    pub fn log(&self) -> Ref<'_, Vec<Event>> {
        self.event_log.borrow()
    }
}

impl EventHandler for TestingHandler {
    type Event = super::Event;

    fn handle(&self, _event: Self::Event) {}
}
