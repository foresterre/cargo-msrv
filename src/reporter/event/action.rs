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

    pub fn action(&self) -> Action {
        self.action
    }
}

impl From<ActionMessage> for Event {
    fn from(it: ActionMessage) -> Self {
        Message::Action(it).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use crate::{Action, ActionMessage, Event};
    use storyteller::Reporter;

    #[test]
    fn reported_action() {
        let reporter = TestReporter::default();
        let event = ActionMessage::new(Action::Find);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::Action(event)),]
        );
    }
}
