use storyteller::EventHandler;

pub struct DiscardOutputHandler;

impl EventHandler for DiscardOutputHandler {
    type Event = super::Event;

    fn handle(&self, _event: Self::Event) {}
}
