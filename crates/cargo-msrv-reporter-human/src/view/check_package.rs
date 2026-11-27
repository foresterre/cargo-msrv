use crate::view::Human;
use cargo_msrv_reporter::event::CheckPackage;
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, CheckPackage> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let package = &self.0.package;
        let header = format!("\nPackage '{}' ({})", package.name, package.path);

        write!(f, "{}", header.bold())
    }
}
