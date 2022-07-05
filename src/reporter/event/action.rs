use crate::reporter::event::Message;
use crate::{Action, Event};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActionMessage {
    action: Action,
}

impl ActionMessage {
    pub fn new(action: Action) -> Self {
        Self { action }
    }
}

impl From<ActionMessage> for Event {
    fn from(it: ActionMessage) -> Self {
        Message::Action(it).into()
    }
}
