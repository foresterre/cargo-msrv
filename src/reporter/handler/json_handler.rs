use crate::io::SendWriter;
use std::io;
use std::io::Stderr;
use std::sync::{Arc, Mutex};
use storyteller::EventHandler;

pub struct JsonHandler<W: SendWriter> {
    writer: Arc<Mutex<W>>,
}

impl<W: SendWriter> JsonHandler<W> {
    const LOCK_FAILURE_MSG: &'static str =
        "{ \"panic\": true, \"cause\": \"Unable to lock writer for JsonHandle\", \"experimental\": true }";
    const SERIALIZE_FAILURE_MSG: &'static str =
        "{ \"panic\": true, \"cause\": \"Unable to serialize event for JsonHandle\", \"experimental\": true }";
    const WRITE_FAILURE_MSG: &'static str =
        "{ \"panic\": true, \"cause\": \"Unable to write serialized event for JsonHandle\", \"experimental\": true }";
}

impl JsonHandler<Stderr> {
    pub fn stderr() -> Self {
        Self {
            writer: Arc::new(Mutex::new(io::stderr())),
        }
    }
}

impl<W: SendWriter> EventHandler for JsonHandler<W> {
    type Event = super::Event;

    fn handle(&self, event: Self::Event) {
        let mut w = self.writer.lock().expect(Self::LOCK_FAILURE_MSG);
        let serialized_event = serde_json::to_string(&event).expect(Self::SERIALIZE_FAILURE_MSG);

        writeln!(&mut w, "{}", &serialized_event).expect(Self::WRITE_FAILURE_MSG);
    }
}
