use crate::draw_target::term_width;
use crate::state::{Outcome, Summary, SummaryRow};
use crate::view::Human;
use cargo_msrv_table::{Margin, Style, Table, Width};
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, Summary> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut table = Table::new(Style::Blank, Width::AtMost(term_width()));
        table.header([
            format!("{}", "Package".dimmed()),
            format!("{}", "MSRV".dimmed()),
        ]);

        for row in self.0.rows() {
            table.row([row.package.name.clone(), Human(row).to_string()]);
        }

        let table = table.margin(Margin::new(2, 0, 1)).to_string();

        write!(f, "\n{}\n{}", "Summary:".bold(), table)
    }
}

impl Display for Human<'_, SummaryRow> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let row = self.0;

        match &row.outcome {
            Outcome::Msrv(Some(msrv)) => write!(f, "{}", msrv.green())?,
            Outcome::Msrv(None) => write!(f, "{}", "N/A".red())?,
            Outcome::Verified {
                version,
                compatible: true,
            } => write!(f, "{} {}", version, "(compatible)".green())?,
            Outcome::Verified {
                version,
                compatible: false,
            } => write!(f, "{} {}", version, "(incompatible)".red())?,
            Outcome::Version(version) => write!(f, "{version}")?,
        }

        if row.written {
            write!(f, " {}", "✓ written".green())?;
        }

        Ok(())
    }
}
