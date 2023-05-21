use crate::reporter::event::shared::compatibility::Compatibility;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VerifyResult {
    pub result: Compatibility,
}

impl VerifyResult {
    pub fn compatible(toolchain: impl Into<OwnedToolchainSpec>) -> Self {
        Self {
            result: Compatibility::compatible(toolchain),
        }
    }

    pub fn incompatible(toolchain: impl Into<OwnedToolchainSpec>, error: Option<String>) -> Self {
        Self {
            result: Compatibility::incompatible(toolchain, error),
        }
    }

    pub fn toolchain(&self) -> &OwnedToolchainSpec {
        self.result.toolchain()
    }

    pub fn is_compatible(&self) -> bool {
        self.result.is_compatible()
    }
}

impl From<VerifyResult> for SubcommandResult {
    fn from(it: VerifyResult) -> Self {
        Self::Verify(it)
    }
}

impl From<VerifyResult> for Event {
    fn from(it: VerifyResult) -> Self {
        Message::SubcommandResult(it.into()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use crate::{semver, Event};
    use storyteller::EventReporter;

    #[test]
    fn reported_compatible_toolchain() {
        let reporter = TestReporterWrapper::default();
        let event = VerifyResult::compatible(OwnedToolchainSpec::new(
            &semver::Version::new(1, 2, 3),
            "test_target",
        ));

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Verify(event)
            )),]
        );
    }

    #[yare::parameterized(
        none = { None },
        some = { Some("whoo!".to_string()) },
    )]
    fn reported_incompatible_toolchain(error_message: Option<String>) {
        let reporter = TestReporterWrapper::default();
        let event = VerifyResult::incompatible(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            error_message,
        );

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Verify(event)
            ))]
        );
    }
}
