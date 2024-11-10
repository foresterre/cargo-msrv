use crate::rust::Toolchain;

mod rustup_toolchain_check;
#[cfg(test)]
mod testing;

use crate::{Compatibility, TResult};
pub use rustup_toolchain_check::{RunCommand, RustupToolchainCheck};

#[cfg(test)]
pub use testing::TestRunner;

/// Implementers of this trait must determine whether a Rust toolchain is _supported_
/// for a Rust project. This is a step in the process of determining the _minimally
/// supported_ Rust version; the MSRV.
pub trait IsCompatible {
    fn before(&self, _toolchain: &Toolchain) -> TResult<()> {
        Ok(())
    }

    fn is_compatible(&self, toolchain: &Toolchain) -> TResult<Compatibility>;

    fn after(&self, _toolchain: &Toolchain) -> TResult<()> {
        Ok(())
    }
}
