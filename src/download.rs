use crate::command::RustupCommand;
use crate::reporter::event::SetupToolchain;
use crate::toolchain::ToolchainSpec;
use crate::{CargoMSRVError, EventReporter, TResult};

pub trait DownloadToolchain {
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()>;
}

#[derive(Debug)]
pub struct ToolchainDownloader<'reporter, R: EventReporter> {
    reporter: &'reporter R,
}

impl<'reporter, R: EventReporter> ToolchainDownloader<'reporter, R> {
    pub fn new(reporter: &'reporter R) -> Self {
        Self { reporter }
    }
}

impl<'reporter, R: EventReporter> DownloadToolchain for ToolchainDownloader<'reporter, R> {
    #[instrument(skip(self, toolchain))]
    fn download(&self, toolchain: &ToolchainSpec) -> TResult<()> {
        info!(toolchain = toolchain.spec(), "installing toolchain");

        self.reporter
            .run_scoped_event(SetupToolchain::new(toolchain.to_owned()), || {
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

                    eprintln!(
                        "Toolchain Download Failed -> \n\n{:?}\n{:?}\n{:?}\n{:?}\n<-\n\n",
                        toolchain.spec(),
                        rustup.stdout(),
                        rustup.stderr(),
                        "rustup failed to install toolchain"
                    );

                    return Err(CargoMSRVError::RustupInstallFailed(
                        toolchain.spec().to_string(),
                    ));
                }

                Ok(())
            })
    }
}
