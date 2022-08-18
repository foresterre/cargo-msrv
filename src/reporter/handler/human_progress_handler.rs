use crate::formatting::TermWidth;
use crate::reporter::event::{CheckResult, CheckToolchain, FindResult, Message, SubcommandResult};
use crate::{semver, Event, SubcommandId};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use storyteller::EventHandler;
use tabled::object::Segment;
use tabled::width::Percent;
use tabled::{Alignment, Disable, Header, Margin, Modify, Style, Table, Width};
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
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner} {msg:<16} Elapsed {elapsed}")
                .unwrap()
                .tick_chars("◐◓◑◒"),
        );
        pb.finish_and_clear(); // Hide the spinner on startup
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
            Message::SubcommandInit(it) if it.subcommand_id().should_enable_spinner() => {
                self.pb.reset(); // We'll reset here to ensure the steady tick call below works
                self.pb.enable_steady_tick(Duration::from_millis(150));
            }
            Message::CheckToolchain(it) if event.is_scope_start() => {
                self.pb.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.start_runner_progress(it.toolchain.version());
            }
            Message::CheckToolchain(it) /* is scope end */ => {
                let version = it.toolchain.version();
                self.finish_runner_progress();
            }
            // Message::Compatibility(CheckResult {  compatibility_report: CompatibilityReport::Compatible, toolchain, .. }) => {
            Message::CheckResult(CheckResult {  compatibility }) if compatibility.is_compatible() => {
                let version = compatibility.toolchain().version();
                let message = Status::ok("Is compatible");
                self.pb.println(message);
            }
            Message::CheckResult(CheckResult { compatibility }) if !compatibility.is_compatible() => {
                let version = compatibility.toolchain().version();
                let message = Status::fail("Is Incompatible");
                self.pb.println(message);

                if let Some(error_report) = compatibility.error() {
                    self.pb.println(message_box(error_report));
                }
            }
            Message::SubcommandResult(result) => self.handle_subcommand_result(result),
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

impl HumanProgressHandler {
    fn handle_subcommand_result(&self, result: &SubcommandResult) {
        match result {
            SubcommandResult::Find(inner) => {
                self.pb.println(format!("\n{}\n", inner.summary()));
            }
            SubcommandResult::List(inner) => {
                self.pb.println(inner.to_string());
            }
            SubcommandResult::Set(inner) => {
                let message = Status::with_lead(
                    "Set".bright_green(),
                    format_args!("Rust {}", inner.version()),
                );
                self.pb.println(message);
            }
            SubcommandResult::Show(inner) => {
                let message = Status::with_lead(
                    "Show".bright_green(),
                    format_args!("MSRV is Rust {}", inner.version()),
                );
                self.pb.println(message);
            }
            SubcommandResult::Verify(inner) => {
                // tbd.
            }
        }
    }
}

impl SubcommandId {
    pub fn should_enable_spinner(&self) -> bool {
        matches!(self, Self::Find | Self::Verify)
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

impl FindResult {
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
        .with(Width::wrap(TermWidth::width()))
        .to_string()
}

fn result_table(result: &FindResult) -> String {
    fn msrv(result: &FindResult) -> String {
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
