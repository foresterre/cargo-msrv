use rust_releases::Release;

pub use {bisect::Bisect, linear::Linear};

use crate::result::MinimalCompatibility;
use crate::storyteller::Reporter;
use crate::{Config, TResult};

/// Use a bisection method to find the MSRV. By using a binary search, we halve our search space each
/// step, making this an efficient search function.
pub(crate) mod bisect;
/// Find the MSRV by stepping through the most-recent to least-recent version, one-by-one. This is
/// not very efficient, but is useful as a baseline, or if you're certain the MSRV is very close to
/// the head.
pub(crate) mod linear;

pub trait FindMinimalCapableToolchain {
    /// Method to find the minimum capable toolchain.
    ///
    /// The search space must be ordered from most to least recent.
    ///
    /// This method returns TODO desc error variants, success case
    fn find_toolchain(
        &self,
        search_space: &[Release],
        config: &Config,
        reporter: &impl Reporter,
    ) -> TResult<MinimalCompatibility>;
}
