use crate::reporter::event::{
    CheckResult, CheckToolchain, FindResult, Message, Meta, SubcommandInit, SubcommandResult,
};
use crate::{semver, table_settings, Event};
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use storyteller::EventHandler;

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
                .tick_chars("◜◠◝◞◡◟"),
        );
        pb.finish_and_clear(); // Hide the spinner on startup
        pb
    }

    fn println(&self, message: impl Display) {
        self.pb.suspend(|| println!("{message}"))
    }
}

impl EventHandler for HumanProgressHandler {
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        #[allow(unused_must_use)]
        match event.message() {
            Message::Meta(it) => {
                let message = it.format_human();
                self.println(message);
            }
            Message::SubcommandInit(it) if it.should_enable_spinner() => {
                self.pb.reset(); // We'll reset here to ensure the steady tick call below works
                self.pb.enable_steady_tick(Duration::from_millis(150));
            }
            Message::UnableToConfirmValidReleaseVersion(_) => {
                let message = Status::info("Unable to verify if provided version is an existing Rust release version");
                self.println(message);
            }
            Message::CheckToolchain(it) if event.is_scope_start() => {
                self.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.start_runner_progress(it.toolchain.version());
            }
            Message::CheckToolchain(_it) /* is scope end */ => {
                self.finish_runner_progress();
            }
            // Message::Compatibility(CheckResult {  compatibility_report: CompatibilityReport::Compatible, toolchain, .. }) => {
            Message::CheckResult(CheckResult {  compatibility }) if compatibility.is_compatible() => {
                let message = Status::ok("Is compatible");
                self.println(message);
            }
            Message::CheckResult(CheckResult { compatibility }) if !compatibility.is_compatible() => {
                let message = Status::fail("Is incompatible");
                self.println(message);

                if let Some(error_report) = compatibility.error() {
                    self.println(message_box(error_report));
                }
            }
            Message::SubcommandResult(result) => self.handle_subcommand_result(result),
            Message::TerminateWithFailure(termination) if termination.should_highlight() => {
                self.println(format!("\n\n{}", termination.as_message().red()));
            }
            Message::TerminateWithFailure(termination) if !termination.should_highlight() => {
                self.println(format!("\n\n{}", termination.as_message().dimmed().bold()));
            }
            _ => {}
        };
    }
}

impl HumanProgressHandler {
    fn handle_subcommand_result(&self, result: &SubcommandResult) {
        match result {
            SubcommandResult::Find(inner) => {
                self.println(format!("\n{}\n", inner.summary()));
            }
            SubcommandResult::List(inner) => {
                self.println(inner);
            }
            SubcommandResult::Set(inner) => {
                let message = Status::with_lead(
                    "Set".bright_green(),
                    format_args!("Rust {}", inner.version()),
                );
                self.println(message);
            }
            SubcommandResult::Show(inner) => {
                let message = Status::with_lead(
                    "Show".bright_green(),
                    format_args!("MSRV is Rust {}", inner.version()),
                );
                self.println(message);
            }
            SubcommandResult::Verify(_inner) => {
                // tbd.
            }
        }
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
    use tabled::builder::Builder;
    use tabled::settings::{object::Columns, Margin, Modify, Style, Width};

    const MAX_LEAD_WIDTH: usize = 6;

    let mut builder = Builder::default();
    builder.push_record([format!("{lead}"), format!("{message}")]);

    let mut table = builder.build();

    table
        .with(Style::blank())
        .with(Modify::new(Columns::first()).with(Width::increase(MAX_LEAD_WIDTH)))
        .with(table_settings!())
        .with(Margin::new(1, 0, 0, 0))
        .to_string()
}

fn message_box(message: &str) -> String {
    use tabled::builder::Builder;
    use tabled::settings::{Margin, Style};

    let mut builder = Builder::default();
    builder.push_record([format!("{}", message.dimmed())]);

    let mut table = builder.build();

    table
        // The remove_{left, right} is a bit of a hack, because their formatting
        // was often flaky (these vertical lines often had unaligned characters)
        .with(Style::modern_rounded())
        .with(table_settings!())
        .with(Margin::new(2, 0, 1, 1))
        .to_string()
}

fn result_table(result: &FindResult) -> String {
    use tabled::settings::{object::Rows, Alignment, Disable, Margin, Style};
    use tabled::Table;

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
        .with(Disable::row(Rows::first()))
        .with(Style::blank()) // Disables the header
        .with(table_settings!())
        .with(Alignment::left())
        .with(Alignment::top())
        .with(Margin::new(2, 0, 0, 1))
        .to_string()
}

impl Meta {
    fn format_human(&self) -> String {
        let sha_fmt = if let Some(sha) = self.sha_short() {
            format!("({})", sha)
        } else {
            String::new()
        };

        Status::meta(format_args!(
            "{} {} {}",
            self.instance(),
            self.version(),
            sha_fmt.trim(),
        ))
    }
}

impl SubcommandInit {
    fn should_enable_spinner(&self) -> bool {
        let id = self.subcommand_id();

        matches!(id, "find" | "verify")
    }
}
