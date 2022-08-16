use crate::reporter::event::Message;
use crate::{Event, SubcommandId};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SubcommandInit {
    subcommand_id: SubcommandId,
}

impl SubcommandInit {
    pub fn new(subcommand_id: SubcommandId) -> Self {
        Self { subcommand_id }
    }

    pub fn subcommand_id(&self) -> SubcommandId {
        self.subcommand_id
    }
}

impl From<SubcommandInit> for Event {
    fn from(it: SubcommandInit) -> Self {
        Message::SubcommandInit(it).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use crate::{Event, SubcommandId, SubcommandInit};
    use storyteller::Reporter;

    #[test]
    fn reported_action() {
        let reporter = TestReporter::default();
        let event = SubcommandInit::new(SubcommandId::Find);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::SubcommandInit(event)),]
        );
    }
}
