use crate::io::SendWriter;
use std::io;
use std::io::Stderr;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};
use storyteller::EventHandler;

#[cfg(test)]
mod testing;

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

#[cfg(test)]
impl<W: SendWriter> JsonHandler<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    pub fn inner_writer(&self) -> MutexGuard<'_, W> {
        self.writer.lock().expect("Unable to unlock writer")
    }
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
