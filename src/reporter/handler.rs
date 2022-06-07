#![allow(unused)]

use crate::{Action, Event};
use owo_colors::OwoColorize;
use std::io::Stderr;
use std::sync::{Arc, Mutex, RwLock};
use std::{io, marker};
use storyteller::{EventHandler, Reporter};

mod discard_output_handler;
mod human_progress_handler;
mod json_handler;

#[cfg(test)]
mod testing_handler;

pub use discard_output_handler::DiscardOutputHandler;
pub use human_progress_handler::HumanProgressHandler;
pub use json_handler::JsonHandler;

#[cfg(test)]
pub use testing_handler::TestingHandler;
