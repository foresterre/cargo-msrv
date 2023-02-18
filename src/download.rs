use crate::command::RustupCommand;
use crate::error::RustupInstallFailed;
use crate::reporter::event::SetupToolchain;
use crate::toolchain::ToolchainSpec;
use crate::{CargoMSRVError, Reporter, TResult};

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
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()> {
        self.reporter
            .run_scoped_event(SetupToolchain::new(toolchain.to_owned()), || {
                let rustup = RustupCommand::new()
                    .with_stdout()
                    .with_stderr()
                    .with_args(["--profile", "minimal", toolchain.spec()])
                    .install()?;

                let status = rustup.exit_status();

                if !status.success() {
                    return Err(CargoMSRVError::RustupInstallFailed(
                        RustupInstallFailed::new(toolchain.spec(), rustup.stderr()),
                    ));
                }

                Ok(())
            })
    }
}
