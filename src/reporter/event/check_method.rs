use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckMethod {
    toolchain: OwnedToolchainSpec,
    method: Method,
}

impl CheckMethod {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>, method: Method) -> Self {
        Self {
            toolchain: toolchain.into(),
            method,
        }
    }
}

impl From<CheckMethod> for Event {
    fn from(it: CheckMethod) -> Self {
        Message::CheckMethod(it).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Method {
    RustupRun {
        args: Vec<String>,
        path: Utf8PathBuf,
    },
    #[cfg(test)]
    TestRunner,
}

impl Method {
    pub fn rustup_run(
        args: impl IntoIterator<Item = impl AsRef<str>>,
        path: impl AsRef<Utf8Path>,
    ) -> Self {
        Self::RustupRun {
            args: args.into_iter().map(|s| s.as_ref().to_string()).collect(),
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporterWrapper;
    use crate::semver;
    use camino::Utf8Path;
    use storyteller::EventReporter;

    #[yare::parameterized(
        rustup_run_with_path = { Method::rustup_run(["hello"], Utf8Path::new("haha")) },
        test_runner = { Method::TestRunner },
    )]
    fn reported_event(method: Method) {
        let reporter = TestReporterWrapper::default();
        let event = CheckMethod::new(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            method,
        );

        reporter.get().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::unscoped(Message::CheckMethod(event)),]
        );
    }
}
