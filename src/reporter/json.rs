use std::cell::Cell;

use json::{number::Number, object, JsonValue};
use rust_releases::semver;

use crate::{config::ModeIntent, reporter::ProgressAction};

#[derive(Debug)]
pub struct JsonPrinter<'s, 't> {
    finished: Cell<u64>,
    steps: Cell<u64>,
    toolchain: &'s str,
    cmd: Option<&'t str>,
}

impl<'s, 't> JsonPrinter<'s, 't> {
    pub fn new(steps: u64, toolchain: &'s str, cmd: Option<&'t str>) -> Self {
        Self {
            finished: Cell::new(0),
            steps: Cell::new(steps),
            toolchain,
            cmd,
        }
    }

    fn reason(mode: ModeIntent) -> &'static str {
        match mode {
            ModeIntent::DetermineMSRV => "msrv-complete",
            ModeIntent::VerifyMSRV => "verify-complete",
            ModeIntent::List => "list-complete",
            ModeIntent::Show => "show-complete",
        }
    }
}

impl<'s, 't> crate::Output for JsonPrinter<'s, 't> {
    fn mode(&self, mode: ModeIntent) {
        let mode: &str = mode.into();

        let mut object = JsonValue::new_object();
        let _ = object.insert("reason", JsonValue::String("mode".to_string()));
        let _ = object.insert("mode", JsonValue::String(mode.to_string()));
        let _ = object.insert("toolchain", JsonValue::String(self.toolchain.to_string()));

        if let Some(cmd) = self.cmd {
            let _ = object.insert("check_cmd", JsonValue::String(cmd.to_string()));
        }

        println!("{}", object);
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
            ProgressAction::Installing(version) | ProgressAction::Checking(version) => {
                let mut object = JsonValue::new_object();
                let _ = object.insert("reason", JsonValue::String(action_str.to_string()));
                let _ = object.insert("version", JsonValue::String(version.to_string()));
                let _ = object.insert(
                    "step",
                    JsonValue::Number(Number::from(self.finished.get() as f64)),
                );
                let _ = object.insert(
                    "total",
                    JsonValue::Number(Number::from(self.steps.get() as f64)),
                );
                let _ = object.insert("toolchain", JsonValue::String(self.toolchain.to_string()));

                if let Some(cmd) = self.cmd {
                    let _ = object.insert("check_cmd", JsonValue::String(cmd.to_string()));
                }

                println!("{}", object);
            }
            ProgressAction::FetchingIndex => println!(
                "{}",
                object! {
                    reason: action_str,
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

    fn finish_success(&self, mode: ModeIntent, version: Option<&semver::Version>) {
        let reason = Self::reason(mode);

        let mut object = JsonValue::new_object();
        let _ = object.insert("reason", JsonValue::String(reason.to_string()));
        let _ = object.insert("success", JsonValue::Boolean(true));

        if let Some(version) = version {
            let _ = object.insert("msrv", version.to_string());
        }

        let _ = object.insert("toolchain", JsonValue::String(self.toolchain.to_string()));

        if let Some(cmd) = self.cmd {
            let _ = object.insert("check_cmd", cmd.to_string());
        }

        println!("{}", object);
    }

    fn finish_failure(&self, mode: ModeIntent, _: Option<&str>) {
        let reason = Self::reason(mode);

        let mut object = JsonValue::new_object();
        let _ = object.insert("reason", reason);
        let _ = object.insert("success", JsonValue::Boolean(false));
        let _ = object.insert("toolchain", JsonValue::String(self.toolchain.to_string()));

        if let Some(cmd) = self.cmd {
            let _ = object.insert("check_cmd", JsonValue::String(cmd.to_string()));
        }

        println!("{}", object);
    }

    fn write_line(&self, content: &str) {
        println!("{}", content);
    }
}
