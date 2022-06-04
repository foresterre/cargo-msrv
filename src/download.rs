use crate::command::RustupCommand;
use crate::toolchain::ToolchainSpec;
use crate::{CargoMSRVError, TResult};
use storyteller::Reporter;

pub trait DownloadToolchain {
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()>;
}

#[derive(Debug)]
pub struct ToolchainDownloader<'reporter, R: Reporter> {
    reporter: &'reporter R,
}

impl<'reporter, R: Reporter> ToolchainDownloader<'reporter, R> {
    pub fn new(reporter: &'reporter R) -> Self {
        Self { reporter }
    }
}

impl<'reporter, R: Reporter> DownloadToolchain for ToolchainDownloader<'reporter, R> {
    // todo!
    // #[instrument(skip(toolchain))]
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()> {
        // todo!
        // self.reporter
        //     .progress(ProgressAction::Installing(toolchain.version()));

        info!(toolchain = toolchain.spec(), "installing toolchain");

        let rustup = RustupCommand::new()
            .with_stdout()
            .with_stderr()
            .with_args(&["--profile", "minimal", toolchain.spec()])
            .install()?;

        let status = rustup.exit_status();

        if !status.success() {
            error!(
                toolchain = toolchain.spec(),
                stdout = rustup.stdout(),
                stderr = rustup.stderr(),
                "rustup failed to install toolchain"
            );

            return Err(CargoMSRVError::RustupInstallFailed(
                toolchain.spec().to_string(),
            ));
        }

        Ok(())
    }
}
