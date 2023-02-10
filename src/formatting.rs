use once_cell::sync::OnceCell;
use tabled::object::Segment;
use tabled::{Alignment, Modify, Table, Tabled, Width};

static TERM_WIDTH: OnceCell<usize> = OnceCell::new();

pub struct TermWidth;

impl TermWidth {
    pub fn width() -> usize {
        *TERM_WIDTH.get_or_init(|| {
            terminal_size::terminal_size()
                .map(|(w, _)| w.0)
                .unwrap_or(80) as usize
        })
    }
}

pub fn table<T: Tabled>(iter: impl IntoIterator<Item = T>) -> Table {
    let width = TermWidth::width();
    let width = width - (width / 5);

    Table::new(iter)
        .with(
            Modify::new(Segment::all()) // A compromise between 'works usually' and still dynamically sizing based on the term width.
                // Tabled and dynamic sizing are very funky together...
                .with(Width::wrap(width).keep_words())
                .with(Alignment::left())
                .with(Alignment::top()),
        )
        .to_owned()
}
