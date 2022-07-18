use crate::reporter::event::Message;
use crate::{CargoMSRVError, Event};

/// Represents a serializable reason why the program should terminate with a failure (a non-zero
/// exit code).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TerminateWithFailure {
    // Not all failure terminations are errors, for example, if we fail to verify we want to exit
    // with a non-zero exit code i.e. 'Terminate with failure',
    is_error: bool,
    reason: SerializableReason,
}

impl TerminateWithFailure {
    pub fn new(error: CargoMSRVError) -> Self {
        let is_error = matches!(error, CargoMSRVError::UnableToFindAnyGoodVersion { .. });

        Self {
            is_error,
            reason: SerializableReason {
                description: format!("{}", &error),
            },
        }
    }

    pub fn is_error(&self) -> bool {
        self.is_error
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
    use crate::reporter::TestReporter;
    use storyteller::Reporter;

    #[test]
    fn reported_non_is_not_error_event() {
        let reporter = TestReporter::default();

        let event = TerminateWithFailure::new(CargoMSRVError::Storyteller);

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(&events, &[Event::new(Message::TerminateWithFailure(event))]);

        if let Message::TerminateWithFailure(msg) = &events[0].message {
            assert!(!msg.is_error());
            assert_eq!(msg.as_message(), "Unable to print event output");
        }
    }

    #[test]
    fn reported_non_is_error_event() {
        let reporter = TestReporter::default();

        let event = TerminateWithFailure::new(CargoMSRVError::UnableToFindAnyGoodVersion {
            command: "cargo build --all".to_string(),
        });

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(&events, &[Event::new(Message::TerminateWithFailure(event))]);

        if let Message::TerminateWithFailure(msg) = &events[0].message {
            assert!(msg.is_error());
            assert!(msg
                .as_message()
                .starts_with("Unable to find a Minimum Supported Rust Version (MSRV)"));
        }
    }
}
