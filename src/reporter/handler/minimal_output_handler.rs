use crate::io::SendWriter;
use crate::reporter::Message;
use crate::Action;
use std::cell::Cell;
use std::io;
use std::io::{Stderr, Stdout};
use std::sync::{Arc, Mutex, MutexGuard};
use storyteller::EventHandler;

/// An output handler which reports just some minimal results.
///
/// It can be used when machine parsing is used, but parsing json would be too much work
/// or there is no desire for the extended output which can be found in the json output.
// Consider: lock stderr for the process, and use writeln!(self.stderr, "{}", ...);
#[derive(Debug)]
pub struct MinimalOutputHandler<W: SendWriter> {
    writer: Arc<Mutex<W>>,
}

impl<W: SendWriter> MinimalOutputHandler<W> {
    fn new(writer: W) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
        }
    }
}

impl MinimalOutputHandler<Stderr> {
    pub fn stderr() -> Self {
        Self {
            writer: Arc::new(Mutex::new(io::stderr())),
        }
    }
}

#[cfg(test)]
impl<W: SendWriter> MinimalOutputHandler<W> {
    fn inner(&self) -> MutexGuard<'_, W> {
        self.writer.lock().expect("Unable to lock writer")
    }
}

impl<W: SendWriter> EventHandler for MinimalOutputHandler<W> {
    type Event = super::Event;

    fn handle(&self, event: Self::Event) {
        use std::io::Write;

        // Early return when message is not a final result message.
        // Also ensures we don't unnecessarily lock the writer.
        if !event.message().is_final_result() {
            return;
        }

        let mut writer = self.writer.lock().expect("Unable to lock writer");

        match event.message() {
            // cargo msrv (find)
            Message::MsrvResult(find) => match find.msrv() {
                Some(v) => writeln!(&mut writer, "{}", v),
                None => writeln!(&mut writer, "none"),
            },
            // cargo msrv list
            // Consider: simplify output for this command, if you want the full output you can use
            //           the default human output mode
            //           for now: unsupported
            Message::ListDep(list) => {
                writeln!(&mut writer, "unsupported")
            }
            // cargo msrv set output
            Message::SetOutput(set) => {
                writeln!(&mut writer, "{}", set.version())
            }
            // cargo msrv show
            Message::ShowOutput(show) => {
                writeln!(&mut writer, "{}", show.version())
            } // cargo msrv verify
            Message::Verify(verify) => {
                writeln!(&mut writer, "{}", verify.is_compatible())
            }
            // If not a final result, discard
            _ => {
                unreachable!("Early return missing, see `Message::is_final_result`.")
            }
        };
    }
}

impl Message {
    fn is_final_result(&self) -> bool {
        matches!(
            &self,
            Message::MsrvResult(_)
                | Message::ListDep(_)
                | Message::SetOutput(_)
                | Message::SetOutput(_)
                | Message::ShowOutput(_)
                | Message::Verify(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::config::list::ListMsrvVariant;
    use crate::dependency_graph::DependencyGraph;
    use crate::manifest::bare_version::BareVersion;
    use crate::reporter::event::{
        ListDep, MsrvResult, Progress, SetOutputMessage, ShowOutputMessage, VerifyOutput,
    };
    use crate::reporter::handler::minimal_output_handler::MinimalOutputHandler;
    use crate::reporter::{Message, ReporterSetup, TestReporter};
    use crate::toolchain::OwnedToolchainSpec;
    use crate::{semver, Action, Config, Event};
    use cargo_metadata::PackageId;
    use serde::Deserialize;
    use std::convert::TryInto;
    use std::path::Path;
    use std::sync::Arc;
    use storyteller::{EventHandler, EventListener, FinishProcessing, Reporter};

    #[test]
    fn find_with_result() {
        let config = Config::new(Action::Find, "my-target");
        let min_available = BareVersion::ThreeComponents(1, 0, 0);
        let max_available = BareVersion::ThreeComponents(2, 0, 0);

        let event = MsrvResult::new_msrv(
            semver::Version::new(1, 10, 100),
            &config,
            min_available,
            max_available,
        );

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "1.10.100\n");
    }

    #[test]
    fn find_without_result() {
        let config = Config::new(Action::Find, "my-target");
        let min_available = BareVersion::ThreeComponents(1, 0, 0);
        let max_available = BareVersion::ThreeComponents(2, 0, 0);

        let event = MsrvResult::none(&config, min_available, max_available);

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "none\n");
    }

    #[test]
    fn list_direct_deps() {
        let config = Config::new(Action::Find, "my-target");
        let min_available = BareVersion::ThreeComponents(1, 0, 0);
        let max_available = BareVersion::ThreeComponents(2, 0, 0);

        let package_id = PackageId {
            repr: "hello_world".to_string(),
        };
        let dep_graph = DependencyGraph::empty(package_id);
        let event = ListDep::new(ListMsrvVariant::DirectDeps, dep_graph);

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "unsupported\n");
    }

    #[test]
    fn set_output() {
        let event = SetOutputMessage::new(
            BareVersion::TwoComponents(1, 20),
            Path::new("/my/path").to_path_buf(),
        );

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "1.20\n");
    }

    #[test]
    fn show_output() {
        let event = ShowOutputMessage::new(
            BareVersion::ThreeComponents(1, 40, 3),
            Path::new("/my/path").to_path_buf(),
        );

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "1.40.3\n");
    }

    #[test]
    fn verify_true() {
        let event = VerifyOutput::compatible(OwnedToolchainSpec::new(
            &semver::Version::new(1, 2, 3),
            "test_target",
        ));

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "true\n");
    }

    #[test]
    fn verify_false_no_error_message() {
        let event = VerifyOutput::incompatible(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            None,
        );

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "false\n");
    }

    #[test]
    fn verify_false_with_error_message() {
        let event = VerifyOutput::incompatible(
            OwnedToolchainSpec::new(&semver::Version::new(1, 2, 3), "test_target"),
            Some("error message".to_string()),
        );

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert_eq!(content.as_ref(), "false\n");
    }

    #[test]
    fn unreported_event() {
        let event = Progress::new(1, 100, 50);

        let buffer = Vec::new();
        let handler = MinimalOutputHandler::new(buffer);
        handler.handle(event.into());

        let output = handler.inner().clone();
        let content = String::from_utf8_lossy(&output);
        assert!(content.as_ref().is_empty());
    }
}
