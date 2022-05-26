// todo: rename-to reporter.rs

use crate::storyteller::message::Message;
use crate::storyteller::progress::{Progression, ProgressionState};

pub(crate) mod message;
pub(crate) mod progress;

/// Messages are a kind of event which report the state of this program to the user
// TODO: fix capitalization of keys
#[derive(serde::Serialize)]
pub enum Event<P: ProgressionState> {
    Progress(Progression<P>),
    Message(Message),
}

#[test]
fn test() {
    let message = Event::Progress(Progression::init(100));

    let serialized = serde_json::to_string(&message).unwrap();
    println!("serialized = {}", serialized);
}
