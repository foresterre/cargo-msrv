use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::Event;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ShowResult {
    version: BareVersion,
    manifest_path: PathBuf,
}

impl ShowResult {
    pub fn new(version: impl Into<BareVersion>, manifest_path: PathBuf) -> Self {
        Self {
            version: version.into(),
            manifest_path,
        }
    }

    pub fn version(&self) -> &BareVersion {
        &self.version
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use storyteller::Reporter;

    #[test]
    fn reported_event() {
        let reporter = TestReporter::default();

        let version = BareVersion::ThreeComponents(1, 2, 3);
        let path = Path::new("lv").to_path_buf();
        let event = ShowResult::new(version, path);

        reporter.reporter().report_event(event.clone()).unwrap();

        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::new(Message::SubcommandResult(
                SubcommandResult::Show(event)
            ))]
        );

        if let Message::SubcommandResult(SubcommandResult::Show(msg)) = &events[0].message {
            assert_eq!(msg.version(), &BareVersion::ThreeComponents(1, 2, 3));
            assert_eq!(&msg.manifest_path(), &Path::new("lv"));
        }
    }
}
