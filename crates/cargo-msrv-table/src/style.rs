use console::measure_text_width;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Rounded,
    Blank,
}

impl Style {
    pub fn top(self) -> Option<Rule> {
        self.rule(Line::new("╭", "┬", "╮"))
    }

    pub fn separator(self) -> Option<Rule> {
        self.rule(Line::new("├", "┼", "┤"))
    }

    pub fn bottom(self) -> Option<Rule> {
        self.rule(Line::new("╰", "┴", "╯"))
    }

    pub fn content(self) -> Line {
        match self {
            Style::Rounded => Line::new("│", "│", "│"),
            Style::Blank => Line::new("", " ", ""),
        }
    }

    fn rule(self, line: Line) -> Option<Rule> {
        match self {
            Style::Rounded => Some(Rule { line, fill: '─' }),
            Style::Blank => None,
        }
    }
}

pub struct Line {
    left: &'static str,
    junction: &'static str,
    right: &'static str,
}

impl Line {
    fn new(left: &'static str, junction: &'static str, right: &'static str) -> Self {
        Self {
            left,
            junction,
            right,
        }
    }

    pub fn join(&self, segments: impl IntoIterator<Item = String>) -> String {
        let inner = segments.into_iter().collect::<Vec<_>>().join(self.junction);

        format!("{}{inner}{}", self.left, self.right)
    }

    pub fn border_width(&self, columns: usize) -> usize {
        let junctions = columns.saturating_sub(1);

        measure_text_width(self.left)
            + measure_text_width(self.junction) * junctions
            + measure_text_width(self.right)
    }
}

pub struct Rule {
    line: Line,
    fill: char,
}

impl Rule {
    pub fn draw(&self, widths: &[usize]) -> String {
        let segments = widths
            .iter()
            .map(|width| self.fill.to_string().repeat(width + 2));

        self.line.join(segments)
    }
}
