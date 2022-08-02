use crate::toolchain::OwnedToolchainSpec;

/// Reports whether a crate is compatible with a certain toolchain, or not.
/// If it's not compatible, it may specify a reason why it is not compatible.

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
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

    pub fn incompatible(toolchain: impl Into<OwnedToolchainSpec>, error: Option<String>) -> Self {
        Self {
            toolchain: toolchain.into(),
            decision: false,
            compatibility_report: CompatibilityReport::Incompatible {
                error: error.map(Into::into),
            },
        }
    }

    pub fn toolchain(&self) -> &OwnedToolchainSpec {
        &self.toolchain
    }

    pub fn is_compatible(&self) -> bool {
        self.decision
    }

    pub fn compatibility_report(&self) -> &CompatibilityReport {
        &self.compatibility_report
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityReport {
    Compatible,
    Incompatible { error: Option<String> },
}

impl CompatibilityReport {
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Compatible => None,
            Self::Incompatible { error } => error.as_deref(),
        }
    }
}
