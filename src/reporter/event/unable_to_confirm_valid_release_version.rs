use crate::reporter::{Event, Message};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnableToConfirmValidReleaseVersion {}

impl From<UnableToConfirmValidReleaseVersion> for Event {
    fn from(it: UnableToConfirmValidReleaseVersion) -> Self {
        Message::UnableToConfirmValidReleaseVersion(it).into()
    }
}
