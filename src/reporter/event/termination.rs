use crate::reporter::event::Message;
use crate::{CargoMSRVError, Event};

/// Represents a serializable reason why the program should terminate with a failure (a non-zero
/// exit code).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TerminateWithFailure {
    // Whether the reason should be highlighted or not.
    #[serde(skip)]
    highlight: bool,
    reason: SerializableReason,
}

impl TerminateWithFailure {
    pub fn new(error: CargoMSRVError) -> Self {
        let highlight = matches!(
            error,
            CargoMSRVError::UnableToFindAnyGoodVersion { .. } | CargoMSRVError::InvalidMsrvSet(_)
        );

        Self {
            highlight,
            reason: SerializableReason {
                description: format!("{}", &error),
            },
        }
    }

    pub fn should_highlight(&self) -> bool {
        self.highlight
    }

    pub fn as_message(&self) -> &str {
        &self.reason.description
    }
}

impl From<TerminateWithFailure> for Event {
    fn from(it: TerminateWithFailure) -> Self {
        Message::TerminateWithFailure(it).into()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct SerializableReason {
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::EventReporter;

    #[test]
    fn reported_non_is_not_error_event() {
        let reporter = TestReporterWrapper::default();

        let event = TerminateWithFailure::new(CargoMSRVError::Storyteller);

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::unscoped(Message::TerminateWithFailure(event))]
        );

        if let Message::TerminateWithFailure(msg) = &events[0].message {
            assert!(!msg.should_highlight());
            assert_eq!(msg.as_message(), "Unable to print event output");
        }
    }

    #[test]
    fn reported_non_is_error_event() {
        let reporter = TestReporterWrapper::default();

        let event = TerminateWithFailure::new(CargoMSRVError::UnableToFindAnyGoodVersion {
            command: "cargo build --all".to_string(),
        });

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::unscoped(Message::TerminateWithFailure(event))]
        );

        if let Message::TerminateWithFailure(msg) = &events[0].message {
            assert!(msg.should_highlight());
            assert!(msg
                .as_message()
                .starts_with("Unable to find a Minimum Supported Rust Version (MSRV)"));
        }
    }
}
