use crate::Event;
use crate::event::Message;
use cargo_msrv_types::Toolchain;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SetupToolchain {
    toolchain: Toolchain,
}

impl SetupToolchain {
    pub fn new(toolchain: impl Into<Toolchain>) -> Self {
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
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use storyteller::EventReporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = SetupToolchain::new(Toolchain::new(
            semver::Version::new(1, 2, 3),
            "test_target",
            &[],
        ));

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::SetupToolchain(event)),]
        );
    }
}
