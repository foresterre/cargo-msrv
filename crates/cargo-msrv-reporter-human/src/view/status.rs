use crate::draw_target::term_width;
use cargo_msrv_table::{Margin, Style, Table, Width};
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

pub enum Lead {
    Meta,
    Mode,
    Ok,
    Fail,
    Info,
    Command(&'static str),
}

impl Display for Lead {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Lead::Meta => write!(f, "[{}]", "Meta".bright_blue()),
            Lead::Mode => write!(f, "[{}]", "Mode".bright_blue()),
            Lead::Ok => write!(f, "[{}]", "OK".bright_green()),
            Lead::Fail => write!(f, "[{}]", "FAIL".bright_red()),
            Lead::Info => write!(f, "[{}]", "INFO".bright_yellow()),
            Lead::Command(name) => write!(f, "[{}]", name.bright_green()),
        }
    }
}

pub fn status(lead: Lead, message: impl Display) -> String {
    const MAX_LEAD_WIDTH: usize = 6;

    let mut table = Table::new(Style::Blank, Width::AtMost(term_width()));
    table
        .row([format!("{lead}"), format!("{message}")])
        .min_width(0, MAX_LEAD_WIDTH)
        .margin(Margin::new(1, 0, 0));

    table.to_string()
}
