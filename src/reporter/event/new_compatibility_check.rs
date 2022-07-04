use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NewCompatibilityCheck {
    pub toolchain: OwnedToolchainSpec,
}

impl NewCompatibilityCheck {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            toolchain: toolchain.into(),
        }
    }
}

impl From<NewCompatibilityCheck> for Event {
    fn from(it: NewCompatibilityCheck) -> Self {
        Message::NewCompatibilityCheck(it).into()
    }
}
