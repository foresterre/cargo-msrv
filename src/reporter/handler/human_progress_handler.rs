use crate::reporter::event::{
    Compatibility, CompatibilityReport, Message, MsrvResult, NewCompatibilityCheck,
};
use crate::{semver, Event};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use storyteller::EventHandler;
use thiserror::private::PathAsDisplay;

pub struct HumanProgressHandler {
    pb: indicatif::ProgressBar,
    sequence_number: AtomicU32,
}

impl HumanProgressHandler {
    pub fn new() -> Self {
        let mp = Self::styled_progress_bar();

        Self {
            pb: mp,
            sequence_number: AtomicU32::new(1),
        }
    }

    fn init_progress(&self, version: &semver::Version) {
        self.sequence_number.fetch_add(1, Ordering::SeqCst);
        self.pb.reset();
        self.pb.set_message(format!("Rust {}", version));
    }

    fn finish_progress(&self) {
        // todo: finish_with_message!
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
                self.pb.println(it.summary());
            }
            Message::NewCompatibilityCheck(it) if event.is_scope_start() => {
                self.pb.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.init_progress(it.toolchain.version());
            }
            Message::NewCompatibilityCheck(it) /* is scope end */ => {
                let version = it.toolchain.version();
                self.finish_progress();
            }
            Message::Compatibility(Compatibility {  compatibility_report: CompatibilityReport::Compatible, toolchain, .. }) => {
                let version = toolchain.version();
                self.pb.println(format!("  [{}] {:>16}", "OK".bright_green(), "Is compatible"));
            }
            Message::Compatibility(Compatibility {  compatibility_report: CompatibilityReport::Incompatible { error }, toolchain, .. }) => {
                let version = toolchain.version();
                self.pb.println(format!("  [{}] {:>16}", "FAIL".bright_red(), "Is Incompatible"));
                self.pb.println(message_box(error));
            }
            Message::MsrvResult(result) => {
                self.pb.println(result.summary());
            }

            _ => {}
        };
    }
}

impl NewCompatibilityCheck {
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
        use std::fmt::Write;
        let mut out = String::with_capacity(32);
        let target = self.target.as_str();

        writeln!(&mut out, "\n{}", "Result:".bold());
        writeln!(
            &mut out,
            "  {:>16}:      (Rust {:>8} … Rust {:>8})",
            format_args!("Considered ({} … {})", "min".cyan(), "max".yellow(),),
            &self.minimum_version.cyan(),
            &self.maximum_version.yellow(),
        );

        writeln!(
            &mut out,
            "  Search method: {:>7}",
            Into::<&'static str>::into(self.search_method).bright_purple()
        );

        if let Some(version) = self.msrv() {
            writeln!(
                &mut out,
                "  {} {:>16}       {}",
                "MSRV:",
                version.green().bold().underline(),
                format_args!("(target: {})", target).dimmed(),
            )
        } else {
            writeln!(
                &mut out,
                "  {}: {:>16}       {}",
                "None".red(),
                "No compatible Rust version found!",
                format_args!("(target: {})", target).dimmed() // todo!: fix summary report when no available compatible versions
            )
        };

        out
    }
}

fn message_box(message: &str) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .add_row(vec![message.trim()]);

    table
        .lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}
