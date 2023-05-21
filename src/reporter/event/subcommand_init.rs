use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SubcommandInit {
    subcommand_id: &'static str,
}

impl SubcommandInit {
    pub fn new(subcommand_id: &'static str) -> Self {
        Self { subcommand_id }
    }

    pub fn subcommand_id(&self) -> &'static str {
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
    use crate::reporter::TestReporterWrapper;
    use crate::{Event, SubcommandInit};
    use storyteller::EventReporter;

    #[test]
    fn reported_action() {
        let reporter = TestReporterWrapper::default();
        let event = SubcommandInit::new("find");

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SubcommandInit(event)),]
        );
    }
}
