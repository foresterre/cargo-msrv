use json::object;
use std::cell::Cell;

use crate::config::ModeIntent;
use crate::reporter::ProgressAction;
use rust_releases::semver;

pub struct JsonPrinter<'s, 't> {
    finished: Cell<u64>,
    steps: Cell<u64>,
    toolchain: &'s str,
    cmd: &'t str,
}

impl<'s, 't> JsonPrinter<'s, 't> {
    pub fn new(steps: u64, toolchain: &'s str, cmd: &'t str) -> Self {
        Self {
            finished: Cell::new(0),
            steps: Cell::new(steps),
            toolchain,
            cmd,
        }
    }

    fn complete_reason(&self, mode: ModeIntent) -> &'static str {
        match mode {
            ModeIntent::DetermineMSRV => "msrv-complete",
            ModeIntent::VerifyMSRV => "verify-complete",
        }
    }
}

impl<'s, 't> crate::Output for JsonPrinter<'s, 't> {
    fn mode(&self, mode: ModeIntent) {
        let mode: &str = mode.into();

        println!(
            "{}",
            object! {
                reason: "mode",
                mode: mode,
                toolchain: self.toolchain,
                check_cmd: self.cmd
            }
        )
    }

    fn set_steps(&self, steps: u64) {
        self.steps.set(steps);
    }

    fn progress(&self, action: crate::ProgressAction, version: &semver::Version) {
        let action = match action {
            ProgressAction::Installing => "installing",
            ProgressAction::Checking => "checking",
        };

        println!(
            "{}",
            object! {
                reason: action,
                version: version.to_string(),
                step: self.finished.get(),
                total: self.steps.get(),
                toolchain: self.toolchain,
                check_cmd: self.cmd,
            }
        );
    }

    fn complete_step(&self, version: &semver::Version, success: bool) {
        println!(
            "{}",
            object! {
                reason: "check-complete",
                version: version.to_string(),
                step: self.finished.get(),
                total_steps: self.steps.get(),
                success: success,
                toolchain: self.toolchain,
                check_cmd: self.cmd,
            }
        );
        self.finished.set(self.finished.get() + 1);
    }

    fn finish_success(&self, mode: ModeIntent, version: &semver::Version) {
        let reason = self.complete_reason(mode);

        println!(
            "{}",
            object! {
                reason: reason,
                success: true,
                msrv: version.to_string(),
                toolchain: self.toolchain,
                check_cmd: self.cmd,
            }
        )
    }

    fn finish_failure(&self, mode: ModeIntent, _: &str) {
        let reason = self.complete_reason(mode);

        println!(
            "{}",
            object! {
                reason: reason,
                success: false,
                toolchain: self.toolchain,
                check_cmd: self.cmd,
            }
        );
    }
}
