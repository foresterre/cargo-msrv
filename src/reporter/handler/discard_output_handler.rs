use crate::Event;
use storyteller::EventHandler;

pub struct DiscardOutputHandler;

impl EventHandler for DiscardOutputHandler {
    type Event = Event;

    fn handle(&self, _event: Self::Event) {}
}
