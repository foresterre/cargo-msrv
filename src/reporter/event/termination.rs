use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::{CargoMSRVError, Event};

/// Represents a serializable reason why the program should terminate with a failure (a non-zero
/// exit code).
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct TerminateWithFailure {
    // Not all failure terminations are errors, for example, if we fail to verify we want to exit
    // with a non-zero exit code i.e. 'Terminate with failure',
    is_error: bool,
    reason: SerializableReason,
}

impl TerminateWithFailure {
    pub fn new(error: CargoMSRVError) -> Self {
        let is_error = match error {
            CargoMSRVError::UnableToFindAnyGoodVersion { .. } => false,
            _ => true,
        };

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

impl IntoIdentifiableEvent for TerminateWithFailure {
    fn identifier(&self) -> &'static str {
        "terminate_with_failure"
    }
}

impl From<TerminateWithFailure> for Event {
    fn from(it: TerminateWithFailure) -> Self {
        Message::TerminateWithFailure(it).into()
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
struct SerializableReason {
    description: String,
}
