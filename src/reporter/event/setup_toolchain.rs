use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use crate::semver;
    use storyteller::EventReporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = SetupToolchain::new(OwnedToolchainSpec::new(
            &semver::Version::new(1, 2, 3),
            "test_target",
        ));

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SetupToolchain(event)),]
        );
    }
}
