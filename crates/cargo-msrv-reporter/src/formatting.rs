use comfy_table::{ContentArrangement, ContentLineStyle, LineStyle, Row, Table, TableStyle};
use std::sync::OnceLock;

static TERM_WIDTH: OnceLock<u16> = OnceLock::new();

static TABLE_CORRECTION: u16 = 4;

pub fn term_width() -> u16 {
    let minimum = 40;

    let width = *TERM_WIDTH.get_or_init(|| {
        terminal_size::terminal_size()
            .map(|(w, _)| w.0)
            .unwrap_or(80)
    });

    width.checked_sub(TABLE_CORRECTION).unwrap_or(minimum)
}

pub const ROUNDED: TableStyle = TableStyle::new()
    .top_border(LineStyle::new('╭', '─', '┬', '╮'))
    .header_lines(ContentLineStyle::new('│', '│', '│'))
    .header_separator(LineStyle::new('├', '─', '┼', '┤'))
    .content_lines(ContentLineStyle::new('│', '│', '│'))
    .row_separator(LineStyle::new('├', '─', '┼', '┤'))
    .bottom_border(LineStyle::new('╰', '─', '┴', '╯'));

pub const BLANK: TableStyle = TableStyle::new()
    .header_lines(ContentLineStyle::none().junction(' '))
    .content_lines(ContentLineStyle::none().junction(' '));

pub fn new_table(style: TableStyle, arrangement: ContentArrangement) -> Table {
    let mut table = Table::new();
    table
        .load_style(style)
        .set_content_arrangement(arrangement)
        .set_width(term_width());
    table
}

pub struct Margin {
    left: usize,
    top: usize,
    bottom: usize,
}

impl Margin {
    pub fn new(left: usize, top: usize, bottom: usize) -> Self {
        Self { left, top, bottom }
    }

    pub fn apply(&self, table: &Table) -> String {
        let indent = " ".repeat(self.left);
        let lines = table
            .lines()
            .map(|line| format!("{indent}{line}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!("{}{lines}{}", "\n".repeat(self.top), "\n".repeat(self.bottom))
    }
}

pub fn table<R: Into<Row>>(header: impl Into<Row>, rows: impl IntoIterator<Item = R>) -> String {
    let mut table = new_table(ROUNDED, ContentArrangement::DynamicFullWidth);
    table.set_header(header).add_rows(rows);

    Margin::new(2, 1, 0).apply(&table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_table(rows: &[[&str; 2]]) -> Table {
        let mut table = Table::new();
        table.load_style(BLANK).add_rows(rows);
        table
    }

    #[test]
    fn margin_indents_every_line() {
        let table = blank_table(&[["a", "b"], ["c", "d"]]);

        let output = Margin::new(2, 0, 0).apply(&table);

        assert_eq!(output, "   a   b \n   c   d ");
    }

    #[test]
    fn margin_adds_empty_lines_around_table() {
        let table = blank_table(&[["a", "b"]]);

        let output = Margin::new(0, 1, 2).apply(&table);

        assert_eq!(output, "\n a   b \n\n");
    }
}
