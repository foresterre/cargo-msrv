/// Allows users to check whether the MSRV of a crate is proper.
///
/// Use case:
///
/// * Run `cargo msrv verify` on the CI, to verify the crates MSRV is acceptable.
pub use {find::Find, list::List, set::Set, show::Show};

use crate::reporter::Reporter;
use crate::TResult;

pub mod find;
pub mod list;
pub mod set;
pub mod show;
pub mod verify;

/// A sub-command of `cargo-msrv`.
///
/// It takes a set of inputs, from the `config`, and reports it's results via the `reporter`.
pub trait SubCommand {
    type Context;
    type Output;

    /// Run the sub-command
    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output>;
}
