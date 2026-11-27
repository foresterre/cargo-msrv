use crate::state::{Running, Unexpected};
use cargo_msrv_reporter::event::SubcommandResult;

pub use find::Find;
pub use list::List;
pub use set::Set;
pub use show::Show;
pub use verify::Verify;

mod find;
mod list;
mod set;
mod show;
mod verify;

pub trait SubCommand: Sized {
    const SPINNER: bool;

    fn result(run: &mut Running<Self>, result: &SubcommandResult) -> Result<(), Unexpected>;
}

pub trait CompatibilityChecks: SubCommand {}

pub trait ResultOnly: SubCommand {}
