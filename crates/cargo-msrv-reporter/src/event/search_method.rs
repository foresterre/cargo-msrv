use crate::Event;
use crate::event::Message;
use cargo_msrv_context::context::SearchMethod as Method;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FindMsrv {
    search_method: Method,
}

impl FindMsrv {
    pub fn new(method: Method) -> Self {
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
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use storyteller::EventReporter;

    #[yare::parameterized(
        linear = { Method::Linear },
        bisect = { Method::Bisect },
    )]
    fn reported_event(method: Method) {
        let reporter = TestReporterWrapper::default();
        let event = FindMsrv::new(method);

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::FindMsrv(event)),]
        );
    }
}
