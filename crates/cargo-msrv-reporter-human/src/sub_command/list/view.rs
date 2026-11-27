use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::ListResult;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, ListResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let result = self.0;

        if let Some(package) = result.package() {
            let lead = status(
                Lead::Command("List"),
                format_args!("MSRV's of dependencies of '{}'", package.name),
            );
            writeln!(f, "{lead}")?;
        }

        write!(f, "{result}")
    }
}
