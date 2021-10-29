use std::cell::Cell;

use json::object;
use rust_releases::semver;

use crate::config::ModeIntent;
use crate::reporter::ProgressAction;

#[derive(Debug)]
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

    fn reason(&self, mode: ModeIntent) -> &'static str {
        match mode {
            ModeIntent::DetermineMSRV => "msrv-complete",
            ModeIntent::VerifyMSRV => "verify-complete",
            ModeIntent::List => "msrv-list",
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

    fn progress(&self, action: crate::ProgressAction) {
        let action_str = match action {
            ProgressAction::Installing(_) => "installing",
            ProgressAction::Checking(_) => "checking",
            ProgressAction::FetchingIndex => "fetching-index",
        };

        match action {
            ProgressAction::Installing(version) | ProgressAction::Checking(version) => println!(
                "{}",
                object! {
                    reason: action_str,
                    version: version.to_string(),
                    step: self.finished.get(),
                    total: self.steps.get(),
                    toolchain: self.toolchain,
                    check_cmd: self.cmd,
                }
            ),
            ProgressAction::FetchingIndex => println!(
                "{}",
                object! {
                    reason: action_str,
                    check_cmd: self.cmd
                }
            ),
        };
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
        let reason = self.reason(mode);

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
        let reason = self.reason(mode);

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

    fn write_line(&self, _content: &str) {
        // TODO Here we want more than str?
        // let reason = self.reason(mode);
        // println!(
        //     "{}",
        //     object! {
        //         reason: reason,
        //         success: true,
        //         list: [{
        //             name: name,
        //             version: version,
        //             msrv: "",
        //             dependencies: ,
        //         }, {
        //             name: name,
        //             version: version,
        //             msrv: "",
        //             dependencies: ,
        //         }, ...]
        //     }
        // )

        todo!()
    }
}
