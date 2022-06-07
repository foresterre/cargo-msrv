#![allow(unused)]

use std::fmt;
use std::fmt::Formatter;

use action::Action;
use meta::Meta;
use progress::Progression;

pub(crate) mod action;
pub(crate) mod message;
pub(crate) mod meta;
pub(crate) mod progress;

/// Messages are a kind of event which report the state of this program to the user
// TODO: fix capitalization of keys
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Meta(Meta),
    Progress(Progression),
    Todo(String), // todo! remove!
    Action(Action),
}

// needed for derive thiserror::Error with #[error(transparent)]
impl fmt::Display for Event {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[test]
fn test() {
    let message = Event::Progress(Progression::new(100));

    let serialized = serde_json::to_string(&message).unwrap();
    println!("serialized = {}", serialized);
}
