use crate::reporter::event::{
    CheckToolchain, Compatibility, CompatibilityReport, Message, MsrvResult,
};
use crate::{semver, Event};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use storyteller::EventHandler;
use tabled::object::Segment;
use tabled::{Alignment, Disable, Header, Margin, Modify, Style, Table};
use thiserror::private::PathAsDisplay;

pub struct HumanProgressHandler {
    pb: indicatif::ProgressBar,
    sequence_number: AtomicU32,
}

impl Default for HumanProgressHandler {
    fn default() -> Self {
        let mp = Self::styled_progress_bar();

        Self {
            pb: mp,
            sequence_number: AtomicU32::new(1),
        }
    }
}

impl HumanProgressHandler {
    fn start_runner_progress(&self, version: &semver::Version) {
        self.sequence_number.fetch_add(1, Ordering::SeqCst);
        self.pb.reset();
        self.pb.set_message(format!("Rust {}", version));
    }

    fn finish_runner_progress(&self) {
        self.pb.finish_and_clear();
    }

    fn styled_progress_bar() -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner} {msg:<16} Elapsed {elapsed}")
                .unwrap()
                .tick_chars("◐◓◑◒"),
        );
        pb
    }
}

impl EventHandler for HumanProgressHandler {
    type Event = super::Event;

    fn handle(&self, event: Self::Event) {
        #[allow(unused_must_use)]
        match event.message() {
            Message::Meta(it) => {
                let message = Status::meta(format_args!(
                    "{} {} ({})",
                    it.instance(),
                    it.version(),
                    it.sha_short(),
                ));
                self.pb.println(message);
            }
            Message::NewCompatibilityCheck(it) if event.is_scope_start() => {
                self.pb.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.start_runner_progress(it.toolchain.version());
            }
            Message::NewCompatibilityCheck(it) /* is scope end */ => {
                let version = it.toolchain.version();
                self.finish_runner_progress();
            }
            Message::Compatibility(Compatibility {  compatibility_report: CompatibilityReport::Compatible, toolchain, .. }) => {
                let version = toolchain.version();
                let message = Status::ok("Is compatible");
                self.pb.println(message);
            }
            Message::Compatibility(Compatibility {  compatibility_report: CompatibilityReport::Incompatible { error }, toolchain, .. }) => {
                let version = toolchain.version();
                let message = Status::fail("Is Incompatible");
                self.pb.println(message);

                if let Some(error_report) = error.as_deref() {
                    self.pb.println(message_box(error_report));
                }
            }
            Message::MsrvResult(result) => {
                self.pb.println(format!("\n{}", result.summary()));
            }
            Message::ListDep(list) => {
                self.pb.println(list.to_string());
            }
            Message::SetOutput(output) => {
                let message = Status::with_lead("Set".bright_green(), format_args!("Rust {}", output.version()));
                self.pb.println(message);
            }
            Message::TerminateWithFailure(termination) if termination.is_error() => {
                self.pb.println(format!("\n\n{}", termination.as_message().red()));
            }
            Message::TerminateWithFailure(termination) if !termination.is_error() => {
                self.pb.println(format!("\n\n{}", termination.as_message().dimmed().bold()));
            }
            _ => {}
        };
    }
}

impl CheckToolchain {
    fn header(&self, nth: u32) -> String {
        format!(
            "\n{} #{}: Rust {}",
            "Compatibility Check",
            nth,
            self.toolchain.version(),
        )
        .bold()
        .to_string()
    }
}

impl MsrvResult {
    fn summary(&self) -> String {
        result_table(self)
    }
}

struct Status;

impl Status {
    fn meta(message: impl Display) -> String {
        let lead = format!("[{}]", "Meta".bright_blue());
        format!("  {:>16}  {}", lead, message)
    }

    fn ok(message: impl Display) -> String {
        let lead = format!("[{}]", "OK".bright_green());
        format!("  {:>16}  {}", lead, message)
    }

    fn fail(message: impl Display) -> String {
        let lead = format!("[{}]", "FAIL".bright_red());
        format!("  {:>16}  {}", lead, message)
    }

    fn with_lead(lead: impl Display, message: impl Display) -> String {
        let lead = format!("[{}]", lead);
        format!("  {:>16}  {}", lead, message)
    }
}

fn message_box(message: &str) -> String {
    Table::new(&[format!("{}", message.dimmed())])
        .with(Disable::Row(..1)) // Disables the header; Style::header_off doesn't work! ordering matters!
        .with(Style::rounded())
        .to_string()
}

fn result_table(result: &MsrvResult) -> String {
    fn msrv(result: &MsrvResult) -> String {
        result
            .msrv()
            .map(|version| format!("{}", version.green().bold().underline()))
            .unwrap_or_else(|| format!("{}", "N/A".red()))
    }

    let target = result.target.as_str();
    let search_method: &str = result.search_method.into();

    let content = &[
        &[
            format!("Considered ({} … {}):", "min".cyan(), "max".yellow()),
            format!(
                "Rust {} … Rust {}",
                result.minimum_version.cyan(),
                result.maximum_version.yellow()
            ),
        ],
        &[
            "Search method:".to_string(),
            format!("{}", search_method.bright_purple()),
        ],
        &["MSRV:".to_string(), msrv(result)],
        &[
            format!("{}", "Target:".dimmed()),
            format!("{}", target.dimmed()),
        ],
    ];

    Table::new(content)
        .with(Disable::Row(..1)) // Disables the header; Style::header_off doesn't work! ordering matters!
        .with(Header(format!("{}", "Result:".bold())))
        .with(Modify::new(Segment::all()).with(Alignment::left()))
        .with(Style::blank())
        .to_string()
}
