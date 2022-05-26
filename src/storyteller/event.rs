#![allow(unused)]

use message::Message;
use progress::Progression;

pub(crate) mod message;
pub(crate) mod progress;

/// Messages are a kind of event which report the state of this program to the user
// TODO: fix capitalization of keys
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Progress(Progression),
    Message(Message),
}

#[test]
fn test() {
    let message = Event::Progress(Progression::new(100));

    let serialized = serde_json::to_string(&message).unwrap();
    println!("serialized = {}", serialized);
}
