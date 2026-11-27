use crate::view::{Human, Lead, message_box, status};
use cargo_msrv_reporter::event::CheckResult;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, CheckResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let compatibility = &self.0.compatibility;

        if compatibility.is_compatible() {
            return f.write_str(&status(Lead::Ok, "Is compatible"));
        }

        f.write_str(&status(Lead::Fail, "Is incompatible"))?;

        match compatibility.error() {
            Some(error) => write!(f, "\n{}", message_box(error)),
            None => Ok(()),
        }
    }
}
