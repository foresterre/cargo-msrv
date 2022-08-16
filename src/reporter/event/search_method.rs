use crate::config::SearchMethod as Method;
use crate::reporter::event::Message;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FindMsrv {
    search_method: Method,
}

impl FindMsrv {
    pub(crate) fn new(method: Method) -> Self {
        Self {
            search_method: method,
        }
    }
}

impl From<FindMsrv> for Event {
    fn from(it: FindMsrv) -> Self {
        Message::FindMsrv(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use storyteller::Reporter;

    #[yare::parameterized(
        linear = { Method::Linear },
        bisect = { Method::Bisect },
    )]
    fn reported_event(method: Method) {
        let reporter = TestReporter::default();
        let event = FindMsrv::new(method);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::FindMsrv(event)),]
        );
    }
}
