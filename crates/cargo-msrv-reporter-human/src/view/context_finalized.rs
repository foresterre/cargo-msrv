use crate::view::{Human, Lead, status};
use cargo_msrv_reporter::event::{ContextFinalized, ProjectMode};
use std::fmt::{self, Display, Formatter};

impl Display for Human<'_, ContextFinalized> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.0.project {
            ProjectMode::Bare => "Without Cargo (no Cargo.toml found)",
            ProjectMode::Cargo {
                workspace: false, ..
            } => "Cargo package",
            ProjectMode::Cargo {
                workspace: true, ..
            } => "Cargo workspace",
        };

        f.write_str(&status(Lead::Mode, message))
    }
}
