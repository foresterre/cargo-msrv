use crate::reporter::event::Message;
use crate::Event;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use storyteller::EventHandler;
use thiserror::private::PathAsDisplay;

pub struct HumanProgressHandler {
    mp: indicatif::MultiProgress,
    pb_handles: Arc<Mutex<HashMap<&'static str, indicatif::ProgressBar>>>,
}

impl HumanProgressHandler {
    pub fn new() -> Self {
        let mp = indicatif::MultiProgress::new();
        let pb_handles = Arc::new(Mutex::new(HashMap::new()));

        Self { mp, pb_handles }
    }

    fn add_spinner(&self, name: &'static str) {
        let sub = self.mp.add(Self::styled_progress_bar());
        self.pb_handles.lock().unwrap().insert(name, sub);
    }

    fn styled_progress_bar() -> indicatif::ProgressBar {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("[{elapsed_precise}] {spinner} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
        );
        pb
    }
}

impl EventHandler for HumanProgressHandler {
    type Event = super::Event;

    fn handle(&self, event: Self::Event) {
        let _ = match event.message() {
            Message::Meta(it) => self.mp.println(it.summary()),
            _ => todo!(),
        };
    }
}

// impl Action {
//     fn to_message(&self) -> String {
//         let message = HumanStatusMessage::new(self.status());
//
//         match self.details() {
//             ActionDetails::FetchingIndex {
//                 source: release_source,
//             } => message.fmt("Obtaining rust-releases index"),
//             ActionDetails::SetupToolchain { .. } => {
//                 message.fmt("Installing toolchain (if not present)")
//             }
//             ActionDetails::CheckToolchain { toolchain } => message.fmt(format_args!(
//                 "Preparing to test Rust '{}' against target '{}'",
//                 toolchain.version(),
//                 toolchain.target(),
//             )),
//             ActionDetails::RunToolchainCheck { version } => message.fmt("Toolchain check"),
//             ActionDetails::RunToolchainCheckPass { version } => {
//                 message.fmt(format_args!("Rust '{}' check passed", version))
//             }
//             ActionDetails::RunToolchainCheckFail {
//                 version,
//                 error_message: error_msg,
//             } => message.fmt(format_args!(
//                 "Rust '{}' check failed with:\n{}",
//                 version,
//                 message_box(error_msg.as_str())
//             )),
//         }
//     }
// }

// #[derive(serde::Serialize)]
// struct HumanStatusMessage {
//     status: ActionStatus, // e.g. Compiling, Downloading, ...
// }
//
// impl HumanStatusMessage {
//     pub fn new(status: ActionStatus) -> Self {
//         Self { status }
//     }
//
//     pub fn fmt<'text>(&self, message: impl fmt::Display) -> String {
//         match self.status {
//             ActionStatus::Passed => {
//                 format!("{:>12} {}", self.status.as_str().green(), message)
//             }
//             ActionStatus::Failed => {
//                 format!("{:>12} {}", self.status.as_str().red(), message)
//             }
//             _ => format!("{:>12} {}", self.status.as_str().bright_green(), message),
//         }
//     }
// }

fn message_box(message: &str) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .add_row(vec![message.trim()]);

    table
        .lines()
        .map(|line| format!("            {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}
