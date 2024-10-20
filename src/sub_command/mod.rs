/// Find the MSRV of a Rust package.
///
/// # Example (CLI)
///
/// `cargo msrv`
pub use find::Find;

/// List the MSRV's of libraries you depend on.
///
/// # Example (CLI)
///
/// `cargo msrv list`
pub use list::List;

/// Check whether the MSRV of a crate is valid as an MSRV.
///
/// # Use case
///
/// - Integrate into a continuous integration (CI) pipeline, to check that your
///   crate fulfills its promised minimally supported Rust version.
///
/// # Example (CLI)
///
/// `cargo msrv verify`
pub use verify::Verify;

/// Write a given MSRV to a Cargo manifest
///
/// # Example (CLI)
///
/// `cargo msrv set 1.50`
pub use set::Set;

/// Show the MSRV present in the Cargo manifest
///
/// # Example (CLI)
///
/// `cargo msrv show`
pub use show::Show;

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
