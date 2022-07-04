use std::path::Path;

use crate::command::RustupCommand;
use crate::config::Config;
use crate::download::{DownloadToolchain, ToolchainDownloader};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::outcome::Outcome;
use crate::paths::crate_root_folder;
use crate::reporter::event::{
    Compatibility, CompatibilityCheckMethod, Method, NewCompatibilityCheck,
};
use crate::reporter::Reporter;
use crate::toolchain::ToolchainSpec;

#[cfg(test)]
mod testing;

#[cfg(test)]
pub use testing::TestRunner;

pub trait Check {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome>;
}

pub struct RustupToolchainCheck<'reporter, R: Reporter> {
    reporter: &'reporter R,
}

impl<'reporter, R: Reporter> Check for RustupToolchainCheck<'reporter, R> {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        self.reporter
            .run_scoped_event(NewCompatibilityCheck::new(toolchain.to_owned()), || {
                info!(ignore_lockfile_enabled = config.ignore_lockfile());

                // temporarily move the lockfile if the user opted to ignore it, and it exists
                let cargo_lock = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;
                let handle_wrap = if config.ignore_lockfile() && cargo_lock.is_file() {
                    let handle = LockfileHandler::new(cargo_lock).move_lockfile()?;

                    Some(handle)
                } else {
                    None
                };

                self.prepare(toolchain, config)?;

                let outcome = self.run_check_command_via_rustup(
                    toolchain,
                    config.crate_path(),
                    config.check_command(),
                )?;

                // report outcome to UI
                self.report_outcome(&outcome)?;

                // move the lockfile back
                if let Some(handle) = handle_wrap {
                    handle.move_lockfile_back()?;
                }

                Ok(outcome)
            })
    }
}

impl<'reporter, R: Reporter> RustupToolchainCheck<'reporter, R> {
    pub fn new(reporter: &'reporter R) -> Self {
        Self { reporter }
    }

    fn prepare(&self, toolchain: &ToolchainSpec, config: &Config) -> TResult<()> {
        let downloader = ToolchainDownloader::new(self.reporter);
        downloader.download(toolchain)?;

        if config.ignore_lockfile() {
            remove_lockfile(config)?;
        }

        Ok(())
    }

    fn run_check_command_via_rustup(
        &self,
        toolchain: &ToolchainSpec,
        dir: Option<&Path>,
        check: &[&str],
    ) -> TResult<Outcome> {
        let mut cmd: Vec<&str> = vec![toolchain.spec()];
        cmd.extend_from_slice(check);

        self.reporter.report_event(CompatibilityCheckMethod::new(
            toolchain.to_owned(),
            Method::rustup_run(&cmd, dir),
        ))?;

        let rustup_output = RustupCommand::new()
            .with_args(cmd.iter())
            .with_optional_dir(dir)
            .with_stderr()
            .run()
            .map_err(|_| CargoMSRVError::UnableToRunCheck)?;

        let status = rustup_output.exit_status();

        if status.success() {
            Ok(Outcome::new_success(toolchain.to_owned()))
        } else {
            let stderr = rustup_output.stderr();
            let command = cmd.join(" ");

            info!(
                ?toolchain,
                stderr,
                cmd = command.as_str(),
                "try_building run failed"
            );

            Ok(Outcome::new_failure(
                toolchain.to_owned(),
                stderr.to_string(),
            ))
        }
    }

    fn report_outcome(&self, outcome: &Outcome) -> TResult<()> {
        match outcome {
            Outcome::Success(outcome) => {
                // report compatibility with this toolchain
                self.reporter
                    .report_event(Compatibility::compatible(outcome.toolchain_spec.to_owned()))?
            }
            Outcome::Failure(outcome) => {
                // report incompatibility with this toolchain
                self.reporter.report_event(Compatibility::incompatible(
                    outcome.toolchain_spec.to_owned(),
                    outcome.error_message.clone(),
                ))?
            }
        };

        Ok(())
    }
}

fn remove_lockfile(config: &Config) -> TResult<()> {
    let lock_file = crate_root_folder(config).map(|p| p.join(CARGO_LOCK))?;

    if lock_file.is_file() {
        std::fs::remove_file(&lock_file).map_err(|error| CargoMSRVError::Io {
            error,
            source: IoErrorSource::RemoveFile(lock_file.clone()),
        })?;
    }

    Ok(())
}
