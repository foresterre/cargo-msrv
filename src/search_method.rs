pub use {bisect::Bisect, linear::Linear};

use crate::msrv::MinimumSupportedRustVersion;
use crate::reporter::Reporter;
use crate::rust_release::RustRelease;
use crate::TResult;

/// Use a bisection method to find the MSRV. By using a binary search, we halve our search space each
/// step, making this an efficient search function.
pub mod bisect;
/// Find the MSRV by stepping through the most-recent to least-recent version, one-by-one. This is
/// not very efficient, but is useful as a baseline, or if you're certain the MSRV is very close to
/// the head.
pub mod linear;

pub trait FindMinimalSupportedRustVersion {
    /// Method to find the minimum capable toolchain.
    ///
    /// The search space must be ordered from most to least recent.
    ///
    /// This method returns TODO desc error variants, success case
    fn find_toolchain(
        &self,
        search_space: &[RustRelease],
        reporter: &impl Reporter,
    ) -> TResult<MinimumSupportedRustVersion>;
}
