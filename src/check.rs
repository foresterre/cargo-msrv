use crate::toolchain::ToolchainSpec;

mod rustup_toolchain_check;
#[cfg(test)]
mod testing;

use crate::{Outcome, TResult};
pub use rustup_toolchain_check::{RunCommand, RustupToolchainCheck};

#[cfg(test)]
pub use testing::TestRunner;

pub trait Check {
    fn check(&self, toolchain: &ToolchainSpec) -> TResult<Outcome>;
}
