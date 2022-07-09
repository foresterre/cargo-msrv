use crate::event::Event;
use crate::event::Message;
use crate::toolchain::OwnedToolchainSpec;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SetupToolchain {
    toolchain: OwnedToolchainSpec,
}

impl SetupToolchain {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            toolchain: toolchain.into(),
        }
    }
}

impl From<SetupToolchain> for Event {
    fn from(it: SetupToolchain) -> Self {
        Message::SetupToolchain(it).into()
    }
}
