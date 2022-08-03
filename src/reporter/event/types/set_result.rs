use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::Event;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SetResult {
    version: BareVersion,
    manifest_path: PathBuf,
}

impl SetResult {
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

impl From<SetResult> for SubcommandResult {
    fn from(it: SetResult) -> Self {
        Self::Set(it)
    }
}

impl From<SetResult> for Event {
    fn from(it: SetResult) -> Self {
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

        let version = BareVersion::TwoComponents(14, 10);
        let event = SetResult::new(version, Path::new("wave").to_path_buf());

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::new(Message::SubcommandResult(
                SubcommandResult::Set(event)
            ))]
        );

        if let Message::SubcommandResult(SubcommandResult::Set(msg)) = &events[0].message {
            assert_eq!(msg.version(), &BareVersion::TwoComponents(14, 10));
            assert_eq!(&msg.manifest_path(), &Path::new("wave"));
        }
    }
}
