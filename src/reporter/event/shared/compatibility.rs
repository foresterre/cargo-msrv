use crate::toolchain::OwnedToolchainSpec;

/// Reports whether a crate is compatible with a certain toolchain, or not.
/// If it's not compatible, it may specify a reason why it is not compatible.

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Compatibility {
    toolchain: OwnedToolchainSpec,
    is_compatible: bool,
    report: CompatibilityReport,
}

impl Compatibility {
    pub fn compatible(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            toolchain: toolchain.into(),
            is_compatible: true,
            report: CompatibilityReport::Compatible,
        }
    }

    pub fn incompatible(toolchain: impl Into<OwnedToolchainSpec>, error: Option<String>) -> Self {
        Self {
            toolchain: toolchain.into(),
            is_compatible: false,
            report: CompatibilityReport::Incompatible {
                error: error.map(Into::into),
            },
        }
    }

    pub fn toolchain(&self) -> &OwnedToolchainSpec {
        &self.toolchain
    }

    pub fn is_compatible(&self) -> bool {
        self.is_compatible
    }

    pub fn report(&self) -> &CompatibilityReport {
        &self.report
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
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
