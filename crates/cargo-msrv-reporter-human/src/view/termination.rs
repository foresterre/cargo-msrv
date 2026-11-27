use crate::view::Human;
use cargo_msrv_reporter::event::TerminateWithFailure;
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, TerminateWithFailure> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let termination = self.0;

        if termination.should_highlight() {
            write!(f, "\n\n{}", termination.as_message().red())
        } else {
            write!(f, "\n\n{}", termination.as_message().dimmed().bold())
        }
    }
}
