use crate::reporter::event::Message;
use crate::toolchain::OwnedToolchainSpec;
use crate::Event;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CompatibilityCheckMethod {
    toolchain: OwnedToolchainSpec,
    method: Method,
}

impl CompatibilityCheckMethod {
    pub fn new(toolchain: impl Into<OwnedToolchainSpec>, method: Method) -> Self {
        Self {
            toolchain: toolchain.into(),
            method,
        }
    }
}

impl From<CompatibilityCheckMethod> for Event {
    fn from(it: CompatibilityCheckMethod) -> Self {
        Message::CompatibilityCheckMethod(it).into()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    RustupRun {
        args: Vec<String>,
        crate_path: PathBuf,
    },
    #[cfg(test)]
    TestRunner,
}

impl Method {
    pub fn rustup_run(
        args: impl IntoIterator<Item = impl AsRef<str>>,
        path: impl AsRef<Path>,
    ) -> Self {
        Self::RustupRun {
            args: args.into_iter().map(|s| s.as_ref().to_string()).collect(),
            crate_path: path.as_ref().to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporter::event::Message;
    use crate::reporter::TestReporter;
    use crate::semver;
    use storyteller::Reporter;

    #[yare::parameterized(
        rustup_run = { Method::rustup_run(&["hello"], Path::new("haha")) },
        test_runner = { Method::TestRunner },
    )]
    fn reported_event(method: Method) {
        let reporter = TestReporter::default();
        let event = CompatibilityCheckMethod::new(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            method,
        );

        reporter.reporter().report_event(event.clone()).unwrap();

        assert_eq!(
            reporter.wait_for_events(),
            vec![Event::new(Message::CompatibilityCheckMethod(event)),]
        );
    }
}
