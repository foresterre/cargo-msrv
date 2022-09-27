use crate::reporter::event::Message;
use crate::Event;

/// Progression indicates how far we are
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Progress {
    // index of the currently running check into the sorted search space
    current: u64,
    // size of the search space
    search_space_size: u64,
    // how many iterations have been completed, including the currently running one
    iteration: u64,
}

impl From<Progress> for Event {
    fn from(it: Progress) -> Self {
        Message::Progress(it).into()
    }
}

impl Progress {
    pub fn new(current: u64, search_space_size: u64, iteration: u64) -> Self {
        Self {
            current,
            search_space_size,
            iteration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::Reporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = Progress::new(10, 100, 30);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::Progress(event)),]
        );
    }
}
