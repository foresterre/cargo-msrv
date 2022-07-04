use crate::reporter::event::Message;
use crate::Event;

/// Progression indicates how far we are
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Progress {
    current: u64,
    max: u64,
    iteration: u64,
}

impl From<Progress> for Event {
    fn from(it: Progress) -> Self {
        Message::Progress(it).into()
    }
}

impl Progress {
    pub fn new(current: u64, max: u64, iteration: u64) -> Self {
        Self {
            current,
            max,
            iteration,
        }
    }
}
