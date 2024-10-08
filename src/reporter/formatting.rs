use once_cell::sync::OnceCell;
use tabled::settings::{Margin, Style};
use tabled::{Table, Tabled};

static TERM_WIDTH: OnceCell<usize> = OnceCell::new();

static TABLE_CORRECTION: usize = 4;

pub fn term_width() -> usize {
    let minimum = 40;

    let width = *TERM_WIDTH.get_or_init(|| {
        terminal_size::terminal_size()
            .map(|(w, _)| w.0)
            .unwrap_or(80) as usize
    });

    width.checked_sub(TABLE_CORRECTION).unwrap_or(minimum)
}

// NB: This is only a macro because tabled uses lots of generics, which
// aren't fun to type manually.
#[macro_export]
macro_rules! table_settings {
    () => {{
        let width = $crate::reporter::formatting::term_width();

        tabled::settings::Settings::default()
            .with(
                tabled::settings::Width::wrap(width)
                    .priority(tabled::settings::peaker::PriorityMax),
            )
            .with(tabled::settings::Width::increase(width))
    }};
}

pub fn table<T: Tabled>(iter: impl IntoIterator<Item = T>) -> Table {
    Table::new(iter)
        .with(Style::modern_rounded())
        .with(table_settings!())
        .with(Margin::new(2, 0, 1, 0))
        .to_owned()
}
