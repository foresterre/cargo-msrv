use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::SetResult;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, SetResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let result = self.0;
        let message = match result.package() {
            Some(package) => status(
                Lead::Command("Set"),
                format_args!("Rust {} for '{}'", result.version(), package.name),
            ),
            None => status(
                Lead::Command("Set"),
                format_args!("Rust {}", result.version()),
            ),
        };

        f.write_str(&message)
    }
}
