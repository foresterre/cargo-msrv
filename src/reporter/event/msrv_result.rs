use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::toolchain::OwnedToolchainSpec;
use crate::typed_bool::{False, True};
use crate::{semver, Event};

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MsrvResult {
    Msrv {
        version: semver::Version,
        success: True,
    },
    None {
        success: False,
    },
}

impl MsrvResult {
    pub fn new_msrv(version: semver::Version) -> Self {
        Self::Msrv {
            version,
            success: True,
        }
    }

    pub fn none() -> Self {
        Self::None { success: False }
    }
}

impl IntoIdentifiableEvent for MsrvResult {
    fn identifier(&self) -> &'static str {
        "msrv_result"
    }
}

impl From<MsrvResult> for Event {
    fn from(it: MsrvResult) -> Self {
        Message::MsrvResult(it).into_event()
    }
}
