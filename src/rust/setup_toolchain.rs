use crate::error::{
    RustupAddComponentError, RustupAddTargetError, RustupError, RustupInstallError,
};
use crate::external_command::rustup_command::RustupCommand;
use crate::reporter::event::SetupToolchain as SetupToolchainEvent;
use crate::rust::Toolchain;
use crate::{CargoMSRVError, Reporter, TResult};

pub trait SetupToolchain {
    fn download(&self, toolchain: &Toolchain) -> TResult<()>;
}

#[derive(Debug)]
pub struct SetupRustupToolchain<'reporter, R: Reporter> {
    reporter: &'reporter R,
}

impl<'reporter, R: Reporter> SetupRustupToolchain<'reporter, R> {
    pub fn new(reporter: &'reporter R) -> Self {
        Self { reporter }
    }
}

impl<'reporter, R: Reporter> SetupToolchain for SetupRustupToolchain<'reporter, R> {
    #[instrument(skip(self, toolchain))]
    fn download(&self, toolchain: &Toolchain) -> TResult<()> {
        self.reporter
            .run_scoped_event(SetupToolchainEvent::new(toolchain.to_owned()), || {
                install_toolchain(toolchain)
                    .and_then(|_| add_target(toolchain))
                    .and_then(|_| {
                        if !toolchain.components().is_empty() {
                            add_components(toolchain)
                        } else {
                            Ok(())
                        }
                    })
            })
    }
}

#[instrument(skip(toolchain))]
fn install_toolchain(toolchain: &Toolchain) -> TResult<()> {
    info!(toolchain = toolchain.spec(), "installing host toolchain");

    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args(["--profile", "minimal", &format!("{}", toolchain.version())])
        .install()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain.spec(),
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to install toolchain"
        );

        return Err(CargoMSRVError::RustupError(RustupError::Install(
            RustupInstallError {
                toolchain_spec: toolchain.spec().to_string(),
                stderr: rustup.stderr().to_string(),
            },
        )));
    }

    Ok(())
}

#[instrument(skip(toolchain))]
fn add_target(toolchain: &Toolchain) -> TResult<()> {
    info!(
        toolchain = toolchain.spec(),
        target = toolchain.target(),
        "adding target to toolchain"
    );

    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args([
            "add",
            "--toolchain",
            &format!("{}", toolchain.version()),
            toolchain.target(),
        ])
        .target()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain.spec(),
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to add target to toolchain"
        );

        return Err(CargoMSRVError::RustupError(RustupError::AddTarget(
            RustupAddTargetError {
                targets: toolchain.target().to_string(),
                toolchain_spec: toolchain.spec().to_string(),
                stderr: rustup.stderr().to_string(),
            },
        )));
    }

    Ok(())
}

#[instrument(skip(toolchain))]
fn add_components(toolchain: &Toolchain) -> TResult<()> {
    info!(
        toolchain = toolchain.spec(),
        target = toolchain.target(),
        components = toolchain.components().join(","),
        "adding components to toolchain"
    );

    let base_arguments = [
        "add",
        "--toolchain",
        &format!("{}", toolchain.version()),
        "--target",
        toolchain.target(),
    ];

    let rustup = RustupCommand::new()
        .with_stdout()
        .with_stderr()
        .with_args(base_arguments.iter().chain(toolchain.components().iter()))
        .component()?;

    let status = rustup.exit_status();

    if !status.success() {
        error!(
            toolchain = toolchain.spec(),
            stdout = rustup.stdout(),
            stderr = rustup.stderr(),
            "rustup failed to add component(s) to toolchain"
        );

        return Err(CargoMSRVError::RustupError(RustupError::AddComponent(
            RustupAddComponentError {
                components: toolchain.components().join(", "),
                toolchain_spec: toolchain.spec().to_string(),
                stderr: rustup.stderr().to_string(),
            },
        )));
    }

    Ok(())
}
