use crate::layout::Width;
use crate::style::{Line, Style};
use crate::wrap::Wrap;
use console::{Alignment, measure_text_width, pad_str};
use std::fmt;

const PADDING: usize = 2;

pub struct Table {
    style: Style,
    width: Width,
    margin: Margin,
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
    min_widths: Vec<usize>,
}

impl Table {
    pub fn new(style: Style, width: Width) -> Self {
        Self {
            style,
            width,
            margin: Margin::default(),
            header: None,
            rows: Vec::new(),
            min_widths: Vec::new(),
        }
    }

    pub fn header(&mut self, cells: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.header = Some(cells.into_iter().map(Into::into).collect());
        self
    }

    pub fn row(&mut self, cells: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    pub fn rows<R>(&mut self, rows: impl IntoIterator<Item = R>) -> &mut Self
    where
        R: IntoIterator<Item: Into<String>>,
    {
        for row in rows {
            self.row(row);
        }
        self
    }

    pub fn min_width(&mut self, column: usize, width: usize) -> &mut Self {
        if self.min_widths.len() <= column {
            self.min_widths.resize(column + 1, 0);
        }
        self.min_widths[column] = width;
        self
    }

    pub fn margin(&mut self, margin: Margin) -> &mut Self {
        self.margin = margin;
        self
    }

    fn lines(&self) -> Vec<String> {
        let widths = self.column_widths();

        if widths.is_empty() {
            return Vec::new();
        }

        let content = self.style.content();
        let mut lines = Vec::new();

        if let Some(top) = self.style.top() {
            lines.push(top.draw(&widths));
        }

        for (index, row) in self.all_rows().enumerate() {
            if index > 0
                && let Some(separator) = self.style.separator()
            {
                lines.push(separator.draw(&widths));
            }

            lines.extend(row_lines(&content, row, &widths));
        }

        if let Some(bottom) = self.style.bottom() {
            lines.push(bottom.draw(&widths));
        }

        lines
    }

    fn all_rows(&self) -> impl Iterator<Item = &Vec<String>> {
        self.header.iter().chain(&self.rows)
    }

    fn column_widths(&self) -> Vec<usize> {
        let columns = self.all_rows().map(Vec::len).max().unwrap_or(0);
        let minimums = (0..columns)
            .map(|column| self.min_widths.get(column).copied().unwrap_or(0).max(1))
            .collect::<Vec<_>>();

        let content_widths = (0..columns)
            .map(|column| {
                let widest_line = self
                    .all_rows()
                    .filter_map(|row| row.get(column))
                    .flat_map(|cell| cell.split('\n'))
                    .map(measure_text_width)
                    .max()
                    .unwrap_or(0);

                widest_line.max(minimums[column])
            })
            .collect();

        let available = self
            .width
            .total()
            .saturating_sub(self.style.content().border_width(columns))
            .saturating_sub(PADDING * columns);

        self.width
            .adjust_column_widths(content_widths, &minimums, available)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = " ".repeat(self.margin.left);
        let body = self
            .lines()
            .iter()
            .map(|line| format!("{indent}{line}"))
            .collect::<Vec<_>>()
            .join("\n");

        write!(
            f,
            "{}{body}{}",
            "\n".repeat(self.margin.top),
            "\n".repeat(self.margin.bottom)
        )
    }
}

fn row_lines(content: &Line, row: &[String], widths: &[usize]) -> Vec<String> {
    let cells = widths
        .iter()
        .enumerate()
        .map(|(column, &width)| {
            let text = row.get(column).map_or("", String::as_str);
            Wrap::new(width).lines(text)
        })
        .collect::<Vec<_>>();

    let height = cells.iter().map(Vec::len).max().unwrap_or(0);

    (0..height)
        .map(|line| {
            let segments = cells.iter().zip(widths).map(|(cell, &width)| {
                let text = cell.get(line).map_or("", String::as_str);
                format!(" {} ", pad_str(text, width, Alignment::Left, None))
            });

            content.join(segments)
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Margin {
    left: usize,
    top: usize,
    bottom: usize,
}

impl Margin {
    pub fn new(left: usize, top: usize, bottom: usize) -> Self {
        Self { left, top, bottom }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_with_header() {
        let mut table = Table::new(Style::Rounded, Width::AtMost(80));
        table.header(["Name", "MSRV"]).row(["foo", "1.70.0"]);

        let expected = "\
╭──────┬────────╮
│ Name │ MSRV   │
├──────┼────────┤
│ foo  │ 1.70.0 │
╰──────┴────────╯";

        assert_eq!(table.to_string(), expected);
    }

    #[test]
    fn blank_wraps_to_width_with_margin() {
        let mut table = Table::new(Style::Blank, Width::AtMost(20));
        table
            .row(["[OK]", "a short message"])
            .min_width(0, 6)
            .margin(Margin::new(1, 1, 0));

        let expected = "\n  [OK]     a short   \n           message   ";

        assert_eq!(table.to_string(), expected);
    }

    #[test]
    fn margin_indents_every_line() {
        let mut table = Table::new(Style::Blank, Width::AtMost(80));
        table
            .rows([["a", "b"], ["c", "d"]])
            .margin(Margin::new(2, 0, 0));

        assert_eq!(table.to_string(), "   a   b \n   c   d ");
    }

    #[test]
    fn margin_adds_empty_lines_around_table() {
        let mut table = Table::new(Style::Blank, Width::AtMost(80));
        table.rows([["a", "b"]]).margin(Margin::new(0, 1, 2));

        assert_eq!(table.to_string(), "\n a   b \n\n");
    }

    #[test]
    fn exact_width_fills_the_line() {
        let mut table = Table::new(Style::Rounded, Width::Exactly(12));
        table.row(["hi"]);

        let expected = "\
╭──────────╮
│ hi       │
╰──────────╯";

        assert_eq!(table.to_string(), expected);
    }
}
