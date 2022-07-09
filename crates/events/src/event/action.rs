use crate::event::{Event, Message};
use cargo_msrv_types::action::Action;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ActionMessage {
    action: Action,
}

impl ActionMessage {
    pub fn new(action: Action) -> Self {
        Self { action }
    }

    pub fn action(&self) -> Action {
        self.action
    }
}

impl From<ActionMessage> for Event {
    fn from(it: ActionMessage) -> Self {
        Message::Action(it).into()
    }
}
