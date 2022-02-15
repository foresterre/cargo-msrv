use std::borrow::Cow;

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use rust_releases::semver;

use crate::config::ModeIntent;

pub struct HumanPrinter<'a> {
    term: Term,
    progress: ProgressBar,
    toolchain: &'a str,
    cmd: &'a str,
}

impl std::fmt::Debug for HumanPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "toolchain = {}, cmd = {}",
            self.toolchain, self.cmd
        ))
    }
}

impl<'a> HumanPrinter<'a> {
    pub fn new(steps: u64, toolchain: &'a str, cmd: &'a str) -> Self {
        let term = Term::stderr();

        let progress = ProgressBar::new(steps).with_style(
            ProgressStyle::default_spinner()
                .template(" {spinner} {msg:<30} {wide_bar} {elapsed_precise}"),
        );

        Self {
            term,
            progress,
            toolchain,
            cmd,
        }
    }

    fn welcome(&self, target: &str, cmd: &str, action_intent: ModeIntent) {
        let verb = match action_intent {
            ModeIntent::Find => "Determining",
            ModeIntent::Verify => "Verifying",
            ModeIntent::List | ModeIntent::Set | ModeIntent::Show => "",
        };

        let _ = self.term.write_line(
            format!(
                "{} the Minimum Supported Rust Version (MSRV) for toolchain {}",
                verb,
                style(target).bold()
            )
            .as_str(),
        );

        let _ = self.term.write_line(
            format!(
                "Using {} command {}",
                style("check").bold(),
                style(cmd).italic(),
            )
            .as_str(),
        );

        self.progress.enable_steady_tick(250);
    }

    fn show_progress(&self, action: &str, version: &semver::Version) {
        self.progress.set_message(format!(
            "{} {}",
            style(action).green().bold(),
            style(version).cyan()
        ));
    }

    fn set_progress_bar_length(&self, len: u64) {
        self.progress.set_length(len);
    }

    fn complete_step(&self, message: impl Into<Cow<'static, str>>) {
        self.progress.set_message(message);
        self.progress.inc(1);
    }

    // for DetermineMSRV
    fn finish_with_ok(&self, message: &str, version: &semver::Version) {
        self.progress.finish_with_message(format!(
            "{} {} {}",
            style("Finished").green().bold(),
            message,
            style(version).cyan()
        ));
    }

    fn finish_with_err(&self, cmd: &str) {
        self.progress.abandon();
        let _ = self.term.write_line(
            format!(
                "   {} {} command {} didn't succeed",
                style("Failed").red().bold(),
                style("check").bold(),
                style(cmd).italic()
            )
            .as_str(),
        );
    }
}

impl<'a> crate::Output for HumanPrinter<'a> {
    fn mode(&self, action: ModeIntent) {
        if let ModeIntent::List | ModeIntent::Show = action {
            return;
        }

        self.welcome(self.toolchain, self.cmd, action);
    }

    fn set_steps(&self, steps: u64) {
        self.set_progress_bar_length(steps);
    }

    fn progress(&self, action: crate::ProgressAction) {
        let (action, version) = match action {
            crate::ProgressAction::Installing(version) => ("Installing", Some(version)),
            crate::ProgressAction::Checking(version) => ("Checking", Some(version)),
            crate::ProgressAction::FetchingIndex => ("Fetching index", None),
        };

        if let Some(version) = version {
            self.show_progress(action, version);
        } else {
            let _ = self.term.write_line(action);
        }
    }

    fn complete_step(&self, version: &semver::Version, success: bool) {
        if success {
            self.complete_step(format!(
                "{} Good check for {}",
                style("Done").green().bold(),
                style(version).cyan()
            ));
        } else {
            self.complete_step(format!(
                "{} Bad check for {}",
                style("Done").green().bold(),
                style(version).cyan()
            ));
        }
    }

    fn finish_success(&self, mode: ModeIntent, version: Option<&semver::Version>) {
        // for determine-msrv and verify-msrv, we report the status
        if let Some(version) = version {
            match mode {
                ModeIntent::Find => self.finish_with_ok("The MSRV is:", version),
                ModeIntent::Verify => self.finish_with_ok("Satisfied MSRV check:", version),
                ModeIntent::Show => {
                    let _ = self.term.write_line(&format!("{}", version));
                }
                ModeIntent::List | ModeIntent::Set => {}
            }
        }
    }

    fn finish_failure(&self, mode: ModeIntent, cmd: Option<&str>) {
        if let ModeIntent::Show = mode {
            let _ = self.term.write_line("No MSRV in Cargo manifest");
            return;
        }

        if let Some(cmd) = cmd {
            self.finish_with_err(cmd);
        }
    }

    fn write_line(&self, content: &str) {
        let _ = self.progress.println(content);
    }
}
