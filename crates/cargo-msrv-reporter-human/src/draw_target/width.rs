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
