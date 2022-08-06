#![allow(unused)]

use crate::{Event, SubcommandId};
use owo_colors::OwoColorize;
use std::io::Stderr;
use std::sync::{Arc, Mutex, RwLock};
use std::{io, marker};
use storyteller::{EventHandler, Reporter};

mod discard_output_handler;
mod human_progress_handler;
mod json_handler;
mod minimal_output_handler;

#[cfg(test)]
mod testing;

pub use discard_output_handler::DiscardOutputHandler;
pub use human_progress_handler::HumanProgressHandler;
pub use json_handler::JsonHandler;
pub use minimal_output_handler::MinimalOutputHandler;

#[cfg(test)]
pub use testing::TestingHandler;
