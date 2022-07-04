use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

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
