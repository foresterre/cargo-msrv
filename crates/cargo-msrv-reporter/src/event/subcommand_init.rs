use crate::Event;
use crate::event::Message;
use cargo_msrv_context::Context;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubcommandId {
    Find,
    List,
    Set,
    Show,
    Verify,
}

impl From<&Context> for SubcommandId {
    fn from(context: &Context) -> Self {
        match context {
            Context::Find(_) => Self::Find,
            Context::List(_) => Self::List,
            Context::Set(_) => Self::Set,
            Context::Show(_) => Self::Show,
            Context::Verify(_) => Self::Verify,
        }
    }
}

impl From<SubcommandInit> for Event {
    fn from(it: SubcommandInit) -> Self {
        Message::SubcommandInit(it).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::Event;
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use crate::event::{SubcommandId, SubcommandInit};
    use storyteller::EventReporter;

    #[test]
    fn reported_action() {
        let reporter = TestReporterWrapper::default();
        let event = SubcommandInit::new(SubcommandId::Find);

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SubcommandInit(event)),]
        );
    }
}
