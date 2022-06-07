use crate::Event;
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
                self.bar.println(Into::<String>::into(action));
            }
        }
    }
}
