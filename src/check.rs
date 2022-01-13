use std::path::Path;

use crate::command::RustupCommand;
use crate::config::Config;
use crate::download::{DownloadToolchain, ToolchainDownloader};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::outcome::Outcome;
use crate::paths::crate_root_folder;
use crate::reporter::{Output, ProgressAction};
use crate::toolchain::ToolchainSpec;

pub trait Check {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome>;
}

pub struct RunCheck<'reporter, R: Output> {
    reporter: &'reporter R,
}

impl<'reporter, R: Output> Check for RunCheck<'reporter, R> {
    fn check(&self, config: &Config, toolchain: &ToolchainSpec) -> TResult<Outcome> {
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

        // move the lockfile back
        if let Some(handle) = handle_wrap {
            handle.move_lockfile_back()?;
        }

        Ok(outcome)
    }
}

impl<'reporter, R: Output> RunCheck<'reporter, R> {
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

        self.reporter
            .progress(ProgressAction::Checking(toolchain.version()));

        let rustup_output = RustupCommand::new()
            .with_args(cmd.iter())
            .with_optional_dir(dir)
            .with_stderr()
            .run()
            .map_err(|_| CargoMSRVError::UnableToRunCheck)?;

        let status = rustup_output.exit_status();

        self.reporter
            .complete_step(toolchain.version(), status.success());

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
                rustup_output.take_stderr(),
            ))
        }
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
