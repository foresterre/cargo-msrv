use crate::draw_target::term_width;
use cargo_msrv_table::{Margin, Style, Table, Width};
use owo_colors::OwoColorize;

pub fn message_box(message: &str) -> String {
    let mut table = Table::new(Style::Rounded, Width::Exactly(term_width()));
    table
        .row([format!("{}", message.dimmed())])
        .margin(Margin::new(2, 1, 1));

    table.to_string()
}
