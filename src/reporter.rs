use std::fmt::Debug;

use crate::Config;
use rust_releases::semver;

use crate::config::{ModeIntent, OutputFormat};
use crate::formatter::FormatUserOutput;
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

pub fn write_succeeded_check(
    success_outcome: &SuccessOutcome,
    config: &Config,
    output: &impl Output,
) {
    write(success_outcome, config, output);
}

pub fn write_failed_check(failure_outcome: &FailureOutcome, config: &Config, output: &impl Output) {
    write(failure_outcome, config, output);
}

fn write(outcome: &impl FormatUserOutput, config: &Config, output: &impl Output) {
    if config.no_check_feedback() {
        return;
    }

    match config.output_format() {
        OutputFormat::Human => {
            output.write_line(&outcome.format_human());
        }
        OutputFormat::Json => {
            output.write_line(&outcome.format_json());
        }
        OutputFormat::None => {}
    };
}
