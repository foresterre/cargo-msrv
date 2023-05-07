use crate::manifest::bare_version::BareVersion;
use crate::reporter::event::subcommand_result::SubcommandResult;
use crate::reporter::event::Message;
use crate::Event;
use camino::{Utf8Path, Utf8PathBuf};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SetResult {
    result: ResultDetails,
}

impl SetResult {
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

        let version = BareVersion::TwoComponents(14, 10);
        let event = SetResult::new(version, Path::new("wave").to_path_buf());

        reporter.reporter().report_event(event.clone()).unwrap();
        let events = reporter.wait_for_events();

        assert_eq!(
            &events,
            &[Event::unscoped(Message::SubcommandResult(
                SubcommandResult::Set(event)
            ))]
        );

        if let Message::SubcommandResult(SubcommandResult::Set(msg)) = &events[0].message {
            assert_eq!(msg.version(), &BareVersion::TwoComponents(14, 10));
            assert_eq!(&msg.manifest_path(), &Path::new("wave"));
        }
    }
}
