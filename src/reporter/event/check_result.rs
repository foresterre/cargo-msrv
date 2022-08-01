use crate::reporter::event::shared::compatibility::Compatibility;
use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckResult {
    #[serde(flatten)]
    pub compatibility: Compatibility,
}

impl CheckResult {
    pub fn compatible(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            compatibility: Compatibility::compatible(toolchain),
        }
    }

    pub fn incompatible(toolchain: impl Into<OwnedToolchainSpec>, error: Option<String>) -> Self {
        Self {
            compatibility: Compatibility::incompatible(toolchain, error),
        }
    }

    pub fn toolchain(&self) -> &OwnedToolchainSpec {
        self.compatibility.toolchain()
    }

    pub fn is_compatible(&self) -> bool {
        self.compatibility.is_compatible()
    }
}

impl From<CheckResult> for Event {
    fn from(it: CheckResult) -> Self {
        Message::Compatibility(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use crate::{semver, Event};
    use storyteller::Reporter;

    #[test]
    fn reported_compatible_toolchain() {
        let reporter = TestReporter::default();
        let event = CheckResult::compatible(OwnedToolchainSpec::new(
            &semver::Version::new(1, 2, 3),
            "test_target",
        ));

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::Compatibility(event)),]
        );
    }

    #[yare::parameterized(
        none = { None },
        some = {Some("whoo!".to_string()) },
    )]
    fn reported_incompatible_toolchain(error_message: Option<String>) {
        let reporter = TestReporter::default();
        let event = CheckResult::incompatible(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            error_message,
        );

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::Compatibility(event)),]
        );
    }
}
