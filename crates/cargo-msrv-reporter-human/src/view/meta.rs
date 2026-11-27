use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::Meta;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, Meta> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let meta = self.0;
        let sha_fmt = match meta.sha_short() {
            Some(sha) => format!("({sha})"),
            None => String::new(),
        };

        f.write_str(&status(
            Lead::Meta,
            format_args!("{} {} {}", meta.instance(), meta.version(), sha_fmt.trim()),
        ))
    }
}
