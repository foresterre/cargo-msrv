use crate::draw_target::term_width;
use crate::view::Human;
use cargo_msrv_reporter::event::FindResult;
use cargo_msrv_table::{Margin, Style, Table, Width};
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, FindResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let result = self.0;
        let title = match result.package() {
            Some(package) => format!("Result for '{}':", package.name),
            None => "Result:".to_string(),
        };

        write!(f, "\n{}\n{}\n", title.bold(), result_table(result))
    }
}

fn result_table(result: &FindResult) -> String {
    fn msrv(result: &FindResult) -> String {
        result
            .msrv()
            .map(|version| format!("{}", version.green().bold().underline()))
            .unwrap_or_else(|| format!("{}", "N/A".red()))
    }

    let target = result.target.as_str();
    let search_method: &str = result.search_method.into();

    let content = [
        [
            format!("Considered ({} … {}):", "min".cyan(), "max".yellow()),
            format!(
                "Rust {} … Rust {}",
                result.minimum_version.cyan(),
                result.maximum_version.yellow()
            ),
        ],
        [
            "Search method:".to_string(),
            format!("{}", search_method.bright_purple()),
        ],
        ["MSRV:".to_string(), msrv(result)],
        [
            format!("{}", "Target:".dimmed()),
            format!("{}", target.dimmed()),
        ],
    ];

    let mut table = Table::new(Style::Blank, Width::AtMost(term_width()));
    table.rows(content).margin(Margin::new(2, 0, 1));

    table.to_string()
}
