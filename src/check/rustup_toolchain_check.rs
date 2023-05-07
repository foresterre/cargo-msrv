use crate::check::Check;
use crate::command::RustupCommand;
use crate::context::EnvironmentContext;
use crate::download::{DownloadToolchain, ToolchainDownloader};
use crate::error::{IoError, IoErrorSource};
use crate::lockfile::{LockfileHandler, CARGO_LOCK};
use crate::reporter::event::{CheckMethod, CheckResult, CheckToolchain, Method};
use crate::toolchain::ToolchainSpec;
use crate::{CargoMSRVError, Outcome, Reporter, TResult};
use camino::{Utf8Path, Utf8PathBuf};
use std::path::Path;

pub struct RustupToolchainCheck<'reporter, 'env, R: Reporter> {
    reporter: &'reporter R,
    settings: Settings<'env>,
}

impl<'reporter, R: Reporter> Check for RustupToolchainCheck<'reporter, '_, R> {
    fn check(&self, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let settings = &self.settings;

        self.reporter
            .run_scoped_event(CheckToolchain::new(toolchain.to_owned()), || {
                info!(ignore_lockfile_enabled = settings.ignore_lockfile());

                // temporarily move the lockfile if the user opted to ignore it, and it exists
                let cargo_lock = self.lockfile_path(settings)?;

                let handle_wrap = if settings.ignore_lockfile() && cargo_lock.is_file() {
                    let handle = LockfileHandler::new(cargo_lock).move_lockfile()?;

                    Some(handle)
                } else {
                    None
                };

                self.prepare(toolchain, settings)?;

                let path = current_dir_crate_path(settings)?;
                let outcome =
                    self.run_check_command_via_rustup(toolchain, path, settings.check_command())?;

                // report outcome to UI
                self.report_outcome(&outcome, settings.no_check_feedback())?;

                // move the lockfile back
                if let Some(handle) = handle_wrap {
                    handle.move_lockfile_back()?;
                }

                Ok(outcome)
            })
    }
}

impl<'reporter, R: Reporter> RustupToolchainCheck<'reporter, R> {
    pub fn new(
        reporter: &'reporter R,
        ignore_lockfile: bool,
        environment: &EnvironmentContext,
    ) -> Self {
        Self {
            reporter,
            settings: Settings {
                ignore_lockfile,
                environment,
            },
        }
    }

    fn prepare(&self, toolchain: &ToolchainSpec, settings: &Settings) -> TResult<()> {
        let downloader = ToolchainDownloader::new(self.reporter);
        downloader.download(toolchain)?;

        if settings.ignore_lockfile() {
            self.remove_lockfile(&settings.lockfile_path())?;
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

        self.reporter.report_event(CheckMethod::new(
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

    fn report_outcome(&self, outcome: &Outcome, no_error_report: bool) -> TResult<()> {
        match outcome {
            Outcome::Success(outcome) => {
                // report compatibility with this toolchain
                self.reporter
                    .report_event(CheckResult::compatible(outcome.toolchain_spec.to_owned()))?
            }
            Outcome::Failure(outcome) if no_error_report => {
                // report incompatibility with this toolchain
                self.reporter.report_event(CheckResult::incompatible(
                    outcome.toolchain_spec.to_owned(),
                    None,
                ))?
            }
            Outcome::Failure(outcome) => {
                // report incompatibility with this toolchain
                self.reporter.report_event(CheckResult::incompatible(
                    outcome.toolchain_spec.to_owned(),
                    Some(outcome.error_message.clone()),
                ))?
            }
        };

        Ok(())
    }

    fn remove_lockfile(&self, lock_file: &Utf8Path) -> TResult<()> {
        if lock_file.is_file() {
            std::fs::remove_file(lock_file).map_err(|error| IoError {
                error,
                source: IoErrorSource::RemoveFile(lock_file.to_path_buf()),
            })?;
        }

        Ok(())
    }
}

struct Settings<'env> {
    ignore_lockfile: bool,
    environment: &'env EnvironmentContext,
}

impl Settings<'_> {
    fn new(ignore_lockfile: bool, environment: &EnvironmentContext) -> Self {
        Self {
            ignore_lockfile,
            environment,
        }
    }

    fn ignore_lockfile(&self) -> bool {
        self.ignore_lockfile
    }

    fn crate_root_path(&self) -> &Utf8Path {
        self.environment.root()
    }

    fn lockfile_path(&self) -> Utf8PathBuf {
        self.environment.lock()
    }
}

/// If we manually specify the path to a crate (e.g. with --manifest-path or --path),
/// we must supply the custom directory to our Command runner.
fn current_dir_crate_path<'c>(config: &'c Config<'c>) -> TResult<Option<&'c Path>> {
    if config.crate_path().is_some() || config.manifest_path().is_some() {
        config.context().crate_root_path().map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod current_dir_crate_path_tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use crate::SubcommandId;

    #[test]
    fn relative_manifest_path() {
        let config = ConfigBuilder::new(SubcommandId::Verify, "")
            .manifest_path(Some("Cargo.toml"))
            .build();

        let res = current_dir_crate_path(&config).unwrap().unwrap();
        assert!(res.file_name().is_none())
    }

    #[test]
    fn relative_crate_path() {
        let config = ConfigBuilder::new(SubcommandId::Verify, "")
            .crate_path(Some("home"))
            .build();

        let res = current_dir_crate_path(&config).unwrap().unwrap();
        assert!(res.file_name().is_some())
    }

    #[test]
    fn no_paths() {
        let config = ConfigBuilder::new(SubcommandId::Verify, "").build();

        let res = current_dir_crate_path(&config).unwrap();
        assert!(res.is_none())
    }
}
