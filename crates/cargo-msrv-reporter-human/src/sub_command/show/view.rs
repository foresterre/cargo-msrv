use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::ShowResult;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, ShowResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let result = self.0;
        let message = match result.package() {
            Some(package) => status(
                Lead::Command("Show"),
                format_args!("MSRV of '{}' is Rust {}", package.name, result.version()),
            ),
            None => status(
                Lead::Command("Show"),
                format_args!("MSRV is Rust {}", result.version()),
            ),
        };

        f.write_str(&message)
    }
}
