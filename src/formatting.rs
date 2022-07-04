use tabled::object::Segment;
use tabled::{Alignment, MaxWidth, Modify, Table, Tabled};

pub fn table<T: Tabled>(iter: impl IntoIterator<Item = T>) -> Table {
    let max_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(60);

    Table::new(iter).with(
        Modify::new(Segment::all())
            // A compromise between 'works usually' and still dynamically sizing based on the term width.
            // Tabled and dynamic sizing are very funky together...
            .with(MaxWidth::wrapping(max_width / 2).keep_words())
            .with(Alignment::left())
            .with(Alignment::top()),
    )
}
