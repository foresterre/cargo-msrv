use crate::formatter::FormatUserOutput;
use crate::toolchain::OwnedToolchainSpec;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use rust_releases::semver;

#[derive(Clone, Debug)]
pub enum Outcome {
    Success(SuccessOutcome),
    Failure(FailureOutcome),
}

impl Outcome {
    pub fn new_success(toolchain_spec: OwnedToolchainSpec) -> Self {
        Self::Success(SuccessOutcome { toolchain_spec })
    }

    pub fn new_failure(toolchain_spec: OwnedToolchainSpec, error_message: String) -> Self {
        Self::Failure(FailureOutcome {
            toolchain_spec,
            error_message,
        })
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn version(&self) -> &semver::Version {
        self.toolchain_spec().version()
    }

    pub fn toolchain_spec(&self) -> &OwnedToolchainSpec {
        match self {
            Self::Success(outcome) => &outcome.toolchain_spec,
            Self::Failure(outcome) => &outcome.toolchain_spec,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessOutcome {
    pub(crate) toolchain_spec: OwnedToolchainSpec,
}

impl FormatUserOutput for SuccessOutcome {
    fn format_human(&self) -> String {
        format!(
            "Check for toolchain '{}' succeeded",
            self.toolchain_spec.spec(),
        )
    }

    fn format_json(&self) -> String {
        let version = self.toolchain_spec.version();
        let toolchain = self.toolchain_spec.spec();

        format!(
            "{}",
            json::object! {
                reason: "last-check-success-message",
                version: format!("{}", version),
                experimental: true, // Message is more unstable other messages and will likely change in the future
                toolchain: toolchain,
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureOutcome {
    pub(crate) toolchain_spec: OwnedToolchainSpec,
    pub(crate) error_message: String,
}

impl FormatUserOutput for FailureOutcome {
    fn format_human(&self) -> String {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .add_row(vec![self.error_message.as_str().trim()]);

        format!(
            "\nCheck for toolchain '{}' failed with:\n{}",
            self.toolchain_spec.spec(),
            table
        )
    }

    fn format_json(&self) -> String {
        let version = self.toolchain_spec.version();
        let toolchain = self.toolchain_spec.spec();
        let error_message = self.error_message.as_str();

        format!(
            "{}",
            json::object! {
                reason: "last-check-failure-message",
                version: format!("{}", version),
                experimental: true,  // Message is more unstable other messages and will likely change in the future
                toolchain: toolchain,
                error_message: error_message,
            }
        )
    }
}
