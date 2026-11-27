use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::UnableToConfirmValidReleaseVersion;
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, UnableToConfirmValidReleaseVersion> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&status(
            Lead::Info,
            "Unable to verify if provided version is an existing Rust release version",
        ))
    }
}
