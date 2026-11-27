use cargo_msrv_reporter::event::CheckToolchain;
use owo_colors::OwoColorize;
use std::fmt::{self, Display, Formatter};

pub struct CheckHeader<'a> {
    pub check: &'a CheckToolchain,
    pub nth: u32,
}

impl Display for CheckHeader<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let header = format!(
            "\nCompatibility Check #{}: Rust {}",
            self.nth,
            self.check.toolchain.version(),
        );

        write!(f, "{}", header.bold())
    }
}
