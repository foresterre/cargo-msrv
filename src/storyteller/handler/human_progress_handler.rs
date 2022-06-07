use crate::storyteller::event::action::{ActionDetails, ActionStatus};
use crate::{Action, Event};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use owo_colors::OwoColorize;
use std::fmt;
use storyteller::EventHandler;

pub struct HumanProgressHandler {
    bar: indicatif::ProgressBar,
}

impl HumanProgressHandler {
    pub fn new() -> Self {
        Self {
            bar: indicatif::ProgressBar::new(0),
        }
    }
}

impl EventHandler for HumanProgressHandler {
    type Event = super::Event;

    fn handle(&self, event: Self::Event) {
        match event {
            Event::Meta(it) => self.bar.println(it.summary()),
            Event::Todo(msg) => self.bar.println(msg),
            Event::Progress(progress) => {}
            Event::Action(action) if action.must_report() => {
                self.bar.println(action.to_message());
            }
            Event::Action(_) => {}
        }
    }
}

impl Action {
    fn to_message(&self) -> String {
        let message = HumanStatusMessage::new(self.status());

        match self.details() {
            ActionDetails::FetchingIndex {
                source: release_source,
            } => message.fmt("Obtaining rust-releases index"),
            ActionDetails::RunToolchainCheck { version } => message.fmt(version),
            ActionDetails::RunToolchainCheckPass { version } => message.fmt(version),
            ActionDetails::RunToolchainCheckFail {
                version,
                error_message: error_msg,
            } => message.fmt(format_args!(
                "{}\n{:>12}",
                version,
                message_box(error_msg.as_str())
            )),
        }
    }
}

#[derive(serde::Serialize)]
struct HumanStatusMessage {
    status: ActionStatus, // e.g. Compiling, Downloading, ...
}

impl HumanStatusMessage {
    pub fn new(status: ActionStatus) -> Self {
        Self { status }
    }

    pub fn fmt<'text>(&self, message: impl fmt::Display) -> String {
        match self.status {
            ActionStatus::Passed => {
                format!("{:>12} {}", self.status.as_str().green(), message)
            }
            ActionStatus::Failed => {
                format!("{:>12} {}", self.status.as_str().red(), message)
            }
            _ => format!("{:>12} {}", self.status.as_str().bright_green(), message),
        }
    }
}

fn message_box(message: &str) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .add_row(vec![message.trim()]);

    table
}
