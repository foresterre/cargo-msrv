use camino::Utf8PathBuf;
use cargo_msrv_context::{CheckCommandContext, EnvironmentContext, Package, ToolchainContext};
use cargo_msrv_rust_tools::CargoCommand;
use cargo_msrv_types::Toolchain;

mod rustup_toolchain_check;
#[cfg(any(test, feature = "testing"))]
mod testing;

use crate::{Compatibility, TResult};
pub use rustup_toolchain_check::{RunCommand, RustupToolchainCheck};

#[cfg(any(test, feature = "testing"))]
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

impl<C: IsCompatible> IsCompatible for &C {
    fn before(&self, toolchain: &Toolchain) -> TResult<()> {
        (*self).before(toolchain)
    }

    fn is_compatible(&self, toolchain: &Toolchain) -> TResult<Compatibility> {
        (*self).is_compatible(toolchain)
    }

    fn after(&self, toolchain: &Toolchain) -> TResult<()> {
        (*self).after(toolchain)
    }
}

#[derive(Debug)]
pub struct CheckTarget {
    pub dir: Utf8PathBuf,
    pub run_command: RunCommand,
}

impl CheckTarget {
    pub fn new(
        check_cmd: &CheckCommandContext,
        toolchain: &ToolchainContext,
        env: &EnvironmentContext,
        package: Option<&Package>,
    ) -> Self {
        Self {
            dir: env.root().to_path_buf(),
            run_command: run_command(check_cmd, toolchain, package),
        }
    }
}

fn run_command(
    check_cmd: &CheckCommandContext,
    toolchain: &ToolchainContext,
    package: Option<&Package>,
) -> RunCommand {
    if let Some(custom) = &check_cmd.rustup_command {
        RunCommand::custom(custom.clone())
    } else {
        let cargo_command = CargoCommand::default()
            .package(package.map(|p| &p.name))
            .target(Some(toolchain.target))
            .features(check_cmd.cargo_features.clone())
            .all_features(check_cmd.cargo_all_features)
            .no_default_features(check_cmd.cargo_no_default_features);

        RunCommand::from_cargo_command(cargo_command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_msrv_context::Project;
    use cargo_msrv_context::types::Edition;

    fn check_cmd(custom: Option<Vec<String>>) -> CheckCommandContext {
        CheckCommandContext {
            cargo_features: None,
            cargo_all_features: false,
            cargo_no_default_features: false,
            rustup_command: custom,
        }
    }

    fn toolchain() -> ToolchainContext {
        ToolchainContext {
            host: "x",
            target: "x",
            components: &[],
        }
    }

    fn env() -> EnvironmentContext {
        EnvironmentContext {
            root_crate_path: Utf8PathBuf::from("root"),
            project: Project::Bare,
        }
    }

    fn package() -> Package {
        Package {
            name: "a".to_string(),
            manifest_path: Utf8PathBuf::from("root/a/Cargo.toml"),
            rust_version: None,
            edition: Edition::Edition2021,
        }
    }

    #[test]
    fn package_is_passed_to_cargo() {
        let target = CheckTarget::new(&check_cmd(None), &toolchain(), &env(), Some(&package()));

        assert_eq!(target.dir, "root");
        assert_eq!(
            target.run_command.components().join(" "),
            "cargo check --package a --target x"
        );
    }

    #[test]
    fn without_package() {
        let target = CheckTarget::new(&check_cmd(None), &toolchain(), &env(), None);

        assert_eq!(
            target.run_command.components().join(" "),
            "cargo check --target x"
        );
    }

    #[test]
    fn custom_command_is_kept_as_is() {
        let custom = vec!["cargo".to_string(), "test".to_string()];
        let target = CheckTarget::new(
            &check_cmd(Some(custom)),
            &toolchain(),
            &env(),
            Some(&package()),
        );

        assert_eq!(target.run_command.components().join(" "), "cargo test");
    }
}
