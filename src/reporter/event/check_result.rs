use crate::reporter::event::shared::compatibility::Compatibility;
use crate::reporter::event::Message;
use crate::rust::Toolchain;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckResult {
    #[serde(flatten)]
    pub compatibility: Compatibility,
}

impl CheckResult {
    pub fn compatible(toolchain: impl Into<Toolchain>) -> Self {
        Self {
            compatibility: Compatibility::compatible(toolchain),
        }
    }

    pub fn incompatible(toolchain: impl Into<Toolchain>, error: Option<String>) -> Self {
        Self {
            compatibility: Compatibility::incompatible(toolchain, error),
        }
    }

    pub fn toolchain(&self) -> &Toolchain {
        self.compatibility.toolchain()
    }

    pub fn is_compatible(&self) -> bool {
        self.compatibility.is_compatible()
    }
}

impl From<CheckResult> for Event {
    fn from(it: CheckResult) -> Self {
        Message::CheckResult(it).into()
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
        let event = CheckResult::compatible(Toolchain::new(
            semver::Version::new(1, 2, 3),
            "test_target",
            &[],
        ));

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::CheckResult(event)),]
        );
    }

    #[yare::parameterized(
        none = { None },
        some = {Some("whoo!".to_string()) },
    )]
    fn reported_incompatible_toolchain(error_message: Option<String>) {
        let reporter = TestReporterWrapper::default();
        let event = CheckResult::incompatible(
            Toolchain::new(semver::Version::new(1, 2, 3), "test_target", &[]),
            error_message,
        );

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::CheckResult(event)),]
        );
    }
}
