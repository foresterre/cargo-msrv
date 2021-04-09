use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use rust_releases::semver;

pub struct Printer {
    term: Term,
    progress: ProgressBar,
}

impl Printer {
    pub fn new(steps: u64) -> Self {
        let term = Term::stderr();

        let progress = ProgressBar::new(steps).with_style(
            ProgressStyle::default_spinner()
                .template(" {spinner} {msg:<30} {wide_bar} {elapsed_precise}"),
        );

        Self { term, progress }
    }

    pub fn welcome(&self, target: &str, cmd: &str) {
        let _ = self.term.write_line(
            format!(
                "Determining the Minimum Supported Rust Version (MSRV) for toolchain {}",
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

        self.progress.enable_steady_tick(500);
    }

    pub fn complete_step(&self, message: &str) {
        self.progress.set_message(message);
        self.progress.inc(1);
    }

    pub fn show_progress(&self, action: &str, version: &semver::Version) {
        self.progress.set_message(
            format!("{} {}", style(action).green().bold(), style(version).cyan()).as_str(),
        );
    }

    pub fn set_progress_bar_length(&self, len: u64) {
        self.progress.set_length(len)
    }

    pub fn finish_with_ok(&self, version: &semver::Version) {
        self.progress.finish_with_message(
            format!(
                "{} The MSRV is {}",
                style("Finished").green().bold(),
                style(version).cyan()
            )
            .as_str(),
        )
    }

    pub fn finish_with_err(&self, cmd: &str) {
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
