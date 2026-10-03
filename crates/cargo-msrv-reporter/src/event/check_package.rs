use crate::Event;
use crate::event::Message;
use cargo_msrv_context::SelectedPackage;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckPackage {
    pub package: SelectedPackage,
}

impl CheckPackage {
    pub fn new(package: SelectedPackage) -> Self {
        Self { package }
    }
}

impl From<CheckPackage> for Event {
    fn from(it: CheckPackage) -> Self {
        Message::CheckPackage(it).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use camino::Utf8PathBuf;
    use storyteller::EventReporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();
        let event = CheckPackage::new(SelectedPackage {
            name: "a".to_string(),
            path: Utf8PathBuf::from("a/Cargo.toml"),
        });

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::CheckPackage(event))]
        );
    }
}
