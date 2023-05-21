use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::Event;
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ShowResult {
    result: ResultDetails,
}

impl ShowResult {
    pub fn new(version: impl Into<BareVersion>, manifest_path: Utf8PathBuf) -> Self {
        Self {
            result: ResultDetails {
                version: version.into(),
                manifest_path,
            },
        }
    }

    pub fn version(&self) -> &BareVersion {
        &self.result.version
    }

    pub fn manifest_path(&self) -> &Utf8Path {
        &self.result.manifest_path
    }
}

impl From<ShowResult> for SubcommandResult {
    fn from(it: ShowResult) -> Self {
        Self::Show(it)
    }
}

impl From<ShowResult> for Event {
    fn from(it: ShowResult) -> Self {
        Message::SubcommandResult(it.into()).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
struct ResultDetails {
    version: BareVersion,
    manifest_path: Utf8PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use storyteller::EventReporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();

        let version = BareVersion::ThreeComponents(1, 2, 3);
        let path = Utf8Path::new("lv").to_path_buf();
        let event = ShowResult::new(version, path);

        reporter.get().report_event(event.clone()).unwrap();

        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Show(event)
            ))]
        );

        if let Message::SubcommandResult(SubcommandResult::Show(msg)) = &events[0].message {
            assert_eq!(msg.version(), &BareVersion::ThreeComponents(1, 2, 3));
            assert_eq!(&msg.manifest_path(), &Utf8Path::new("lv"));
        }
    }
}
