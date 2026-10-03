use crate::event::{
    CheckPackage, CheckResult, CheckToolchain, FindResult, Message, Meta, SubcommandInit,
    SubcommandResult,
};
use crate::{Event, table_settings};
use cargo_msrv_context::SelectedPackage;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use storyteller::EventHandler;

pub struct HumanProgressHandler {
    pb: indicatif::ProgressBar,
    sequence_number: AtomicU32,
    summary: Mutex<Vec<SummaryRow>>,
}

impl Default for HumanProgressHandler {
    fn default() -> Self {
        let mp = Self::styled_progress_bar();

        Self {
            pb: mp,
            sequence_number: AtomicU32::new(1),
            summary: Mutex::new(Vec::new()),
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

    fn add_to_summary(&self, package: Option<&SelectedPackage>, result: String) {
        if let Some(package) = package {
            let mut summary = self.summary.lock().unwrap_or_else(|e| e.into_inner());
            summary.push(SummaryRow {
                package: package.name.clone(),
                result,
            });
        }
    }

    fn print_summary(&self) {
        let rows = std::mem::take(&mut *self.summary.lock().unwrap_or_else(|e| e.into_inner()));

        if rows.len() > 1 {
            self.println(format!("\n{}\n{}", "Summary:".bold(), summary_table(&rows)));
        }
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
            Message::CheckPackage(it) if event.is_scope_start() => {
                self.sequence_number.store(1, Ordering::SeqCst);
                self.println(it.header());
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
                self.print_summary();
                self.println(format!("\n\n{}", termination.as_message().red()));
            }
            Message::TerminateWithFailure(termination) if !termination.should_highlight() => {
                self.print_summary();
                self.println(format!("\n\n{}", termination.as_message().dimmed().bold()));
            }
            _ => {}
        };
    }

    fn finish(&self) {
        self.print_summary();
    }
}

impl HumanProgressHandler {
    fn handle_subcommand_result(&self, result: &SubcommandResult) {
        match result {
            SubcommandResult::Find(inner) => {
                self.println(format!("\n{}\n", inner.summary()));

                let msrv = inner
                    .msrv()
                    .map_or_else(|| format!("{}", "N/A".red()), |v| format!("{}", v.green()));
                self.add_to_summary(inner.package(), msrv);
            }
            SubcommandResult::List(inner) => {
                self.println(inner);
            }
            SubcommandResult::Set(inner) => {
                let message = match inner.package() {
                    Some(package) => Status::with_lead(
                        "Set".bright_green(),
                        format_args!("Rust {} for '{}'", inner.version(), package.name),
                    ),
                    None => Status::with_lead(
                        "Set".bright_green(),
                        format_args!("Rust {}", inner.version()),
                    ),
                };
                self.println(message);
                self.add_to_summary(inner.package(), inner.version().to_string());
            }
            SubcommandResult::Show(inner) => {
                let message = match inner.package() {
                    Some(package) => Status::with_lead(
                        "Show".bright_green(),
                        format_args!("MSRV of '{}' is Rust {}", package.name, inner.version()),
                    ),
                    None => Status::with_lead(
                        "Show".bright_green(),
                        format_args!("MSRV is Rust {}", inner.version()),
                    ),
                };
                self.println(message);
                self.add_to_summary(inner.package(), inner.version().to_string());
            }
            SubcommandResult::Verify(inner) => {
                let version = inner.toolchain().version();
                let result = if inner.is_compatible() {
                    format!("{} {}", version, "(compatible)".green())
                } else {
                    format!("{} {}", version, "(incompatible)".red())
                };
                self.add_to_summary(inner.package(), result);
            }
        }
    }
}

impl CheckPackage {
    fn header(&self) -> String {
        format!("\nPackage '{}' ({})", self.package.name, self.package.path)
            .bold()
            .to_string()
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
        let title = match self.package() {
            Some(package) => format!("Result for '{}':", package.name),
            None => "Result:".to_string(),
        };
        let title = title.bold();
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
    use tabled::settings::{Margin, Modify, Style, Width, object::Columns};

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
    use tabled::Table;
    use tabled::settings::{Alignment, Disable, Margin, Style, object::Rows};

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

struct SummaryRow {
    package: String,
    result: String,
}

fn summary_table(rows: &[SummaryRow]) -> String {
    use tabled::builder::Builder;
    use tabled::settings::{Alignment, Margin, Style};

    let mut builder = Builder::default();
    builder.push_record([
        format!("{}", "Package".dimmed()),
        format!("{}", "MSRV".dimmed()),
    ]);

    for row in rows {
        builder.push_record([row.package.clone(), row.result.clone()]);
    }

    builder
        .build()
        .with(Style::blank())
        .with(table_settings!())
        .with(Alignment::left())
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
