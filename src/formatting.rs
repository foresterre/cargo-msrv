use tabled::object::Segment;
use tabled::{Alignment, MaxWidth, Modify, Table, Tabled};

pub fn table<T: Tabled>(iter: impl IntoIterator<Item = T>) -> Table {
    let max_width = terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or(60);

    Table::new(iter).with(
        Modify::new(Segment::all())
            // A compromise between 'works usually' and still dynamically sizing based on the term width.
            // Tabled and dynamic sizing are very funky together...
            .with(MaxWidth::wrapping(usize::from(max_width) / 2).keep_words())
            .with(Alignment::left())
            .with(Alignment::top()),
    )
}
