use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(serde::Serialize, Clone)]
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

impl IntoIdentifiableEvent for NewCompatibilityCheck {
    fn identifier(&self) -> &'static str {
        "new_compatibility_check"
    }
}

impl From<NewCompatibilityCheck> for Event {
    fn from(it: NewCompatibilityCheck) -> Self {
        Message::NewCompatibilityCheck(it).into()
    }
}
