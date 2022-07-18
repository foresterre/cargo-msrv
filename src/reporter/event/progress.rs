use crate::reporter::event::Message;
use crate::Event;

/// Progression indicates how far we are
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use storyteller::Reporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporter::default();
        let event = Progress::new(10, 100, 30);

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::Progress(event)),]
        );
    }
}
