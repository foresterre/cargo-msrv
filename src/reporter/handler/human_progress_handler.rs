use crate::reporter::event::{Compatibility, CompatibilityReport, Message, NewCompatibilityCheck};
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
    mp: indicatif::ProgressBar,
    sequence_number: AtomicU32,
}

impl HumanProgressHandler {
    pub fn new() -> Self {
        let mp = Self::styled_progress_bar();

        Self {
            mp,
            sequence_number: AtomicU32::new(1),
        }
    }

    fn init_progress(&self, version: &semver::Version) {
        self.sequence_number.fetch_add(1, Ordering::SeqCst);
        self.mp.reset();
        self.mp.set_message(format!("Rust {}", version));
    }

    fn finish_progress(&self) {
        // todo: finish_with_message!
        self.mp.finish_and_clear();
    }

    fn styled_progress_bar() -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner} {msg:<16} Elapsed {elapsed_precise}")
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
                self.mp.println(it.summary());
            }
            Message::NewCompatibilityCheck(it) if event.is_scope_start() => {
                self.mp.println(it.header(self.sequence_number.load(Ordering::SeqCst)));
                self.init_progress(it.toolchain.version());
            }
            Message::NewCompatibilityCheck(it) /* is scope end */ => {
                let version = it.toolchain.version();
                self.finish_progress();
            }
            Message::Compatibility(Compatibility {  compatibility_report: CompatibilityReport::Incompatible { error }, toolchain, .. }) => {
                let version = toolchain.version();
                self.mp.println(message_box(error));
            }

            _ => {
                self.mp.println("Warning: not implemented!");
            }
        };
    }
}

impl NewCompatibilityCheck {
    fn header(&self, nth: u32) -> String {
        format!(
            "{} #{}: {}",
            "Compatibility Check",
            nth,
            self.toolchain.version(),
        )
        .bold()
        .to_string()
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
