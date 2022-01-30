use std::fmt::Debug;

use crate::Config;
use rust_releases::semver;

use crate::config::{ModeIntent, OutputFormat};
use crate::formatter::{FormatUserOutput, Human, Json};
use crate::outcome::{FailureOutcome, SuccessOutcome};

pub mod json;
pub mod no_output;
pub mod ui;

#[derive(Debug, Clone, Copy)]
pub enum ProgressAction<'a> {
    Installing(&'a semver::Version),
    Checking(&'a semver::Version),
    FetchingIndex,
}

pub trait Output: Debug {
    // Shows the mode in which cargo-msrv will operate
    fn mode(&self, mode: ModeIntent);

    // Sets the remaining amount of steps for the mode
    fn set_steps(&self, steps: u64);

    // Reports the currently running
    fn progress(&self, action: ProgressAction);
    fn complete_step(&self, version: &semver::Version, success: bool);
    fn finish_success(&self, mode: ModeIntent, version: Option<&semver::Version>);
    fn finish_failure(&self, mode: ModeIntent, cmd: Option<&str>);

    fn write_line(&self, content: &str);
}

pub mod __private {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rust_releases::semver;

    use crate::config::ModeIntent;
    use crate::reporter::{Output, ProgressAction};

    /// This is meant to be used for testing
    #[derive(Debug)]
    pub struct SuccessOutput {
        successes: Rc<RefCell<Vec<(bool, semver::Version)>>>,
    }

    impl SuccessOutput {
        pub fn expose_successes(&self) -> Vec<(bool, semver::Version)> {
            self.successes()
        }
    }

    impl Output for SuccessOutput {
        fn mode(&self, _action: ModeIntent) {}
        fn set_steps(&self, _steps: u64) {}
        fn progress(&self, _action: ProgressAction) {}
        fn complete_step(&self, version: &semver::Version, success: bool) {
            let mut successes = self.successes.borrow_mut();
            successes.push((success, version.clone()));
        }
        fn finish_success(&self, _mode: ModeIntent, _version: Option<&semver::Version>) {}
        fn finish_failure(&self, _mode: ModeIntent, _cmd: Option<&str>) {}
        fn write_line(&self, _content: &str) {}
    }

    impl Default for SuccessOutput {
        fn default() -> Self {
            Self {
                successes: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }
    impl SuccessOutput {
        pub fn successes(&self) -> Vec<(bool, semver::Version)> {
            self.successes.clone().borrow().to_owned()
        }
    }
}

pub fn write_succeeded_check(
    success_outcome: &SuccessOutcome,
    config: &Config,
    output: &impl Output,
) {
    if config.no_check_feedback() {
        return;
    }

    match config.output_format() {
        OutputFormat::Human => {
            output.write_line(&FormatUserOutput::<Human>::format_line(success_outcome))
        }
        OutputFormat::Json => {
            output.write_line(&FormatUserOutput::<Json>::format_line(success_outcome))
        }
        _ => {}
    };
}

pub fn write_failed_check(failure_outcome: &FailureOutcome, config: &Config, output: &impl Output) {
    if config.no_check_feedback() {
        return;
    }

    match config.output_format() {
        OutputFormat::Human => {
            output.write_line(&FormatUserOutput::<Human>::format_line(failure_outcome))
        }
        OutputFormat::Json => {
            output.write_line(&FormatUserOutput::<Json>::format_line(failure_outcome))
        }
        _ => {}
    };
}
