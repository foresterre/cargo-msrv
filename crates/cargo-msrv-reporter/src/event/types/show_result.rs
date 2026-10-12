use crate::Event;
use crate::event::Message;
use crate::event::subcommand_result::SubcommandResult;
use camino::{Utf8Path, Utf8PathBuf};
use cargo_msrv_context::SelectedPackage;
use msrv::MSRV;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ShowResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<SelectedPackage>,
    result: ResultDetails,
}

impl ShowResult {
    pub fn new(version: impl Into<MSRV>, manifest_path: Utf8PathBuf) -> Self {
        Self {
            package: None,
            result: ResultDetails {
                version: version.into(),
                manifest_path,
            },
        }
    }

    pub fn with_package(mut self, package: Option<SelectedPackage>) -> Self {
        self.package = package;
        self
    }

    pub fn package(&self) -> Option<&SelectedPackage> {
        self.package.as_ref()
    }

    pub fn version(&self) -> &MSRV {
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
    version: MSRV,
    manifest_path: Utf8PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestReporterWrapper;
    use crate::event::Message;
    use storyteller::EventReporter;
    use version_number::Version;

    #[test]
    fn reported_event() {
        let reporter = TestReporterWrapper::default();

        let version = Version::new_full_version(1, 2, 3);
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
            assert_eq!(
                msg.version(),
                &MSRV::new(Version::new_full_version(1, 2, 3))
            );
            assert_eq!(&msg.manifest_path(), &Utf8Path::new("lv"));
        }
    }
}
