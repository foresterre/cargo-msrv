use crate::storyteller::event::action::{ActionDetails, ActionStatus};
use crate::{Action, Event};
use owo_colors::OwoColorize;
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
            Event::Todo(msg) => self.bar.println(msg),
            Event::Progress(progress) => {}
            Event::Action(action) => {
                self.bar.println(action.to_message());
            }
        }
    }
}

impl Action {
    fn to_message(&self) -> String {
        match self.details() {
            ActionDetails::FetchingIndex {
                source: release_source,
            } => {
                HumanStatusMessage::new(ActionStatus::Fetching).fmt("Obtaining rust-releases index")
            }
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

    pub fn fmt<'text>(&self, message: impl Into<&'text str>) -> String {
        format!("{:>12} {}", self.status.as_str().green(), message.into(),)
    }
}
