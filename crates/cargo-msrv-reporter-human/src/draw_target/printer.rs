use std::fmt::Display;
use std::time::Duration;

pub struct Printer {
    bar: indicatif::ProgressBar,
}

#[must_use]
pub struct Ticking {
    _private: (),
}

impl Printer {
    pub fn new() -> Self {
        let bar = indicatif::ProgressBar::new_spinner();
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner} {msg:<16} Elapsed {elapsed}")
                .unwrap()
                .tick_chars("◜◠◝◞◡◟"),
        );
        bar.finish_and_clear(); // Hide the spinner on startup

        Self { bar }
    }

    pub fn line(&mut self, message: impl Display) {
        self.bar.suspend(|| println!("{message}"))
    }

    pub fn idle_spinner(&mut self) {
        self.bar.reset(); // We'll reset here to ensure the steady tick call below works
        self.bar.enable_steady_tick(Duration::from_millis(150));
    }

    pub fn start(&mut self, label: String) -> Ticking {
        self.bar.reset();
        self.bar.set_message(label);

        Ticking { _private: () }
    }

    pub fn stop(&mut self, ticking: Ticking) {
        let Ticking { _private } = ticking;
        self.bar.finish_and_clear();
    }
}
