use crate::event::Event;
use crate::event::Message;
use crate::toolchain::OwnedToolchainSpec;

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
