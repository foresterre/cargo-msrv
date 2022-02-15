use std::cell::Cell;

use json::object::Object;
use json::JsonValue;
use rust_releases::semver;

use crate::config::ModeIntent;
use crate::reporter::ProgressAction;

#[derive(Debug)]
pub struct JsonPrinter<'a> {
    finished: Cell<u64>,
    steps: Cell<u64>,
    toolchain: &'a str,
    cmd: Option<&'a str>,
}

impl<'a> JsonPrinter<'a> {
    pub fn new(steps: u64, toolchain: &'a str, cmd: Option<&'a str>) -> Self {
        Self {
            finished: Cell::new(0),
            steps: Cell::new(steps),
            toolchain,
            cmd,
        }
    }

    fn reason(mode: ModeIntent) -> &'static str {
        match mode {
            ModeIntent::Find => "msrv-complete",
            ModeIntent::Verify => "verify-complete",
            ModeIntent::List => "list-complete",
            ModeIntent::Set => "set-complete",
            ModeIntent::Show => "show-complete",
        }
    }
}

impl<'a> crate::Output for JsonPrinter<'a> {
    fn mode(&self, mode: ModeIntent) {
        let mode: &str = mode.into();

        let mut object = Object::new();
        object.insert("reason", "mode".into());
        object.insert("mode", mode.into());
        object.insert("toolchain", self.toolchain.into());

        if let Some(cmd) = self.cmd {
            object.insert("check_cmd", cmd.into());
        }

        println!("{}", JsonValue::Object(object));
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
                let mut object = Object::new();
                object.insert("reason", action_str.into());
                object.insert("version", version.to_string().into());
                object.insert("step", self.finished.get().into());
                object.insert("total", self.steps.get().into());
                object.insert("toolchain", self.toolchain.into());

                if let Some(cmd) = self.cmd {
                    object.insert("check_cmd", cmd.into());
                }

                println!("{}", JsonValue::Object(object));
            }
            ProgressAction::FetchingIndex => println!(
                "{}",
                json::object! {
                    reason: action_str,
                }
            ),
        };
    }

    fn complete_step(&self, version: &semver::Version, success: bool) {
        println!(
            "{}",
            json::object! {
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

        let mut object = Object::new();
        object.insert("reason", reason.into());
        object.insert("success", true.into());

        if let Some(version) = version {
            object.insert("msrv", version.to_string().into());
        }

        object.insert("toolchain", self.toolchain.into());

        if let Some(cmd) = self.cmd {
            object.insert("check_cmd", cmd.into());
        }

        println!("{}", JsonValue::Object(object));
    }

    fn finish_failure(&self, mode: ModeIntent, _: Option<&str>) {
        let reason = Self::reason(mode);

        let mut object = Object::new();
        object.insert("reason", reason.into());
        object.insert("success", false.into());
        object.insert("toolchain", self.toolchain.into());

        if let Some(cmd) = self.cmd {
            object.insert("check_cmd", cmd.into());
        }

        println!("{}", JsonValue::Object(object));
    }

    fn write_line(&self, content: &str) {
        println!("{}", content);
    }
}
