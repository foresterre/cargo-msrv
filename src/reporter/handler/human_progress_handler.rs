use crate::formatting::TermWidth;
use crate::reporter::event::{
    CheckResult, CheckToolchain, FindResult, Message, Meta, SubcommandResult,
};
use crate::{semver, Event, SubcommandId};
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use storyteller::EventHandler;
use tabled::builder::Builder;
use tabled::object::{Columns, Rows, Segment};
use tabled::{Alignment, Disable, Margin, Modify, Style, Table, Width};

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
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        #[allow(unused_must_use)]
        match event.message() {
            Message::Meta(it) => {
                let message = it.format_human();
                self.pb.println(message);
            }
            Message::SubcommandInit(it) if it.subcommand_id().should_enable_spinner() => {
                self.pb.reset(); // We'll reset here to ensure the steady tick call below works
                self.pb.enable_steady_tick(Duration::from_millis(150));
            }
            Message::UnableToConfirmValidReleaseVersion(_) => {
                let message = Status::info("Unable to verify if provided version is an existing Rust release version");
                self.pb.println(message);
            }
            Message::CheckToolchain(it) if event.is_scope_start() => {
                self.pb.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.start_runner_progress(it.toolchain.version());
            }
            Message::CheckToolchain(_it) /* is scope end */ => {
                self.finish_runner_progress();
            }
            // Message::Compatibility(CheckResult {  compatibility_report: CompatibilityReport::Compatible, toolchain, .. }) => {
            Message::CheckResult(CheckResult {  compatibility }) if compatibility.is_compatible() => {
                let message = Status::ok("Is compatible");
                self.pb.println(message);
            }
            Message::CheckResult(CheckResult { compatibility }) if !compatibility.is_compatible() => {
                let message = Status::fail("Is incompatible");
                self.pb.println(message);

                if let Some(error_report) = compatibility.error() {
                    self.pb.println(message_box(error_report));
                }
            }
            Message::SubcommandResult(result) => self.handle_subcommand_result(result),
            Message::TerminateWithFailure(termination) if termination.should_highlight() => {
                self.pb.println(format!("\n\n{}", termination.as_message().red()));
            }
            Message::TerminateWithFailure(termination) if !termination.should_highlight() => {
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
            SubcommandResult::Verify(_inner) => {
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
        let title = "Result:".bold();
        let table = result_table(self);

        format!("{}\n{}", title, table)
    }
}

struct Status;

impl Status {
    fn meta(message: impl Display) -> String {
        let lead = format!("[{}]", "Meta".bright_blue());
        status(lead, message)
    }

    fn ok(message: impl Display) -> String {
        let lead = format!("[{}]", "OK".bright_green());

        status(lead, message)
    }

    fn fail(message: impl Display) -> String {
        let lead = format!("[{}]", "FAIL".bright_red());

        status(lead, message)
    }

    fn info(message: impl Display) -> String {
        let lead = format!("[{}]", "INFO".bright_yellow());
        status(lead, message)
    }

    fn with_lead(lead: impl Display, message: impl Display) -> String {
        let lead = format!("[{}]", lead);
        status(lead, message)
    }
}

fn status(lead: impl Display, message: impl Display) -> String {
    let mut builder = Builder::default();
    builder.add_record([format!("{lead}"), format!("{message}")]);

    let mut table = builder.build();

    table
        .with(Alignment::left())
        .with(Modify::new(Columns::first()).with(Width::increase(6)))
        .with(Width::wrap(TermWidth::width()))
        .with(Style::blank())
        .with(Margin::new(1, 0, 0, 0))
        .to_string()
}

fn message_box(message: &str) -> String {
    let mut builder = Builder::default();
    builder.add_record([format!("{}", message.dimmed())]);

    let mut table = builder.build();

    table
        .with(Width::wrap(TermWidth::width()))
        .with(Style::rounded())
        .with(Margin::new(2, 0, 1, 1))
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
        .with(Disable::row(Rows::first())) // Disables the header
        .with(Modify::new(Segment::all()).with(Alignment::left()))
        .with(Style::blank())
        .with(Margin::new(2, 0, 0, 1))
        .to_string()
}

impl Meta {
    fn format_human(&self) -> String {
        let sha_short = self.sha_short();

        let sha_fmt = if sha_short.is_empty() {
            String::new()
        } else {
            format!("({})", sha_short)
        };

        Status::meta(format_args!(
            "{} {} {}",
            self.instance(),
            self.version(),
            sha_fmt.trim(),
        ))
    }
}
