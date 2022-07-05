use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckToolchain {
    pub toolchain: OwnedToolchainSpec,
}

impl CheckToolchain {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            toolchain: toolchain.into(),
        }
    }
}

impl From<CheckToolchain> for Event {
    fn from(it: CheckToolchain) -> Self {
        Message::NewCompatibilityCheck(it).into()
    }
}
