use cargo_msrv_table::{Margin, Style, Table, Width};
use std::sync::OnceLock;

static TERM_WIDTH: OnceLock<usize> = OnceLock::new();

static FALLBACK_TERM_WIDTH: usize = 80;

static TABLE_CORRECTION: usize = 4;

pub fn term_width() -> usize {
    let minimum = 40;

    let width = *TERM_WIDTH.get_or_init(|| {
        terminal_size::terminal_size()
            .map(|(w, _)| usize::from(w.0))
            .unwrap_or(FALLBACK_TERM_WIDTH)
    });

    width.checked_sub(TABLE_CORRECTION).unwrap_or(minimum)
}

pub fn table<R>(
    header: impl IntoIterator<Item: Into<String>>,
    rows: impl IntoIterator<Item = R>,
) -> String
where
    R: IntoIterator<Item: Into<String>>,
{
    let mut table = Table::new(Style::Rounded, Width::Exactly(term_width()));
    table.header(header).rows(rows).margin(Margin::new(2, 1, 0));

    table.to_string()
}
