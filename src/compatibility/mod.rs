use crate::external_command::cargo_command::CargoCommand;
use crate::rust::Toolchain;
use cargo_msrv_context::{CheckCommandContext, FindContext, ToolchainContext, VerifyContext};

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

/// The command which is run to determine whether a Rust toolchain is compatible.
pub trait RunCommandProvider {
    fn provide_run_command(&self) -> RunCommand;
}

impl RunCommandProvider for FindContext {
    fn provide_run_command(&self) -> RunCommand {
        run_command(&self.check_cmd, &self.toolchain)
    }
}

impl RunCommandProvider for VerifyContext {
    fn provide_run_command(&self) -> RunCommand {
        run_command(&self.check_cmd, &self.toolchain)
    }
}

fn run_command(check_cmd: &CheckCommandContext, toolchain: &ToolchainContext) -> RunCommand {
    if let Some(custom) = &check_cmd.rustup_command {
        RunCommand::custom(custom.clone())
    } else {
        let cargo_command = CargoCommand::default()
            .target(Some(toolchain.target))
            .features(check_cmd.cargo_features.clone())
            .all_features(check_cmd.cargo_all_features)
            .no_default_features(check_cmd.cargo_no_default_features);

        RunCommand::from_cargo_command(cargo_command)
    }
}
