#![deny(clippy::all)]

use crate::state::{Done, Setup, State};
use cargo_msrv_reporter::Event;
use std::sync::{Mutex, MutexGuard};
use storyteller::EventHandler;

mod draw_target;
mod state;
mod sub_command;
mod view;

pub struct HumanProgressHandler {
    state: Mutex<State>,
}

impl Default for HumanProgressHandler {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::Setup(Setup::new())),
        }
    }
}

impl HumanProgressHandler {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl EventHandler for HumanProgressHandler {
    type Event = Event;

    fn handle(&self, event: Self::Event) {
        let mut state = self.lock();
        let current = std::mem::replace(&mut *state, State::Done(Done));
        *state = current.step(&event);
    }

    fn finish(&self) {
        let mut state = self.lock();
        let current = std::mem::replace(&mut *state, State::Done(Done));
        *state = State::Done(current.finish());
    }
}
