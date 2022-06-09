use crate::reporter::event::{IntoIdentifiableEvent, Message};
use crate::toolchain::OwnedToolchainSpec;
use crate::{semver, Event};

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Compatibility {
    toolchain: OwnedToolchainSpec,
    decision: bool,
    compatibility_report: CompatibilityReport,
}

impl Compatibility {
    pub fn compatible(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            toolchain: toolchain.into(),
            decision: true,
            compatibility_report: CompatibilityReport::Compatible,
        }
    }

    pub fn incompatible(
        toolchain: impl Into<OwnedToolchainSpec>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            toolchain: toolchain.into(),
            decision: false,
            compatibility_report: CompatibilityReport::Incompatible {
                error: error.into(),
            },
        }
    }
}

impl IntoIdentifiableEvent for Compatibility {
    fn identifier(&self) -> &'static str {
        "compatibility"
    }
}

impl From<Compatibility> for Event {
    fn from(it: Compatibility) -> Self {
        Message::Compatibility(it).into_event()
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityReport {
    Compatible,
    Incompatible { error: String },
}
