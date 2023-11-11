use crate::check::Check;
use crate::command::RustupCommand;
use crate::context::{CheckCmdContext, EnvironmentContext};
use crate::download::{DownloadToolchain, ToolchainDownloader};
use crate::error::{IoError, IoErrorSource};
use crate::lockfile::LockfileHandler;
use crate::reporter::event::{CheckMethod, CheckResult, CheckToolchain, Method};
use crate::toolchain::ToolchainSpec;
use crate::{lockfile, CargoMSRVError, Outcome, Reporter, TResult};
use camino::{Utf8Path, Utf8PathBuf};

pub struct RustupToolchainCheck<'reporter, 'env, 'cc, R: Reporter> {
    reporter: &'reporter R,
    settings: Settings<'env, 'cc>,
}

impl<'reporter, 'env, 'cc, R: Reporter> Check for RustupToolchainCheck<'reporter, 'env, 'cc, R> {
    fn check(&self, toolchain: &ToolchainSpec) -> TResult<Outcome> {
        let settings = &self.settings;

        self.reporter
            .run_scoped_event(CheckToolchain::new(toolchain.to_owned()), || {
                info!(ignore_lockfile_enabled = settings.ignore_lockfile());

                // temporarily move the lockfile if the user opted to ignore it, and it exists
                let ignore_lockfile = settings.ignore_lockfile();
                let handle_wrap = create_lockfile_handle(ignore_lockfile, settings.environment)
                    .map(|handle| handle.move_lockfile())
                    .transpose()?;

                self.setup_toolchain(toolchain)?;

                if handle_wrap.is_some() {
                    remove_lockfile(&settings.lockfile_path())?;
                }

                let crate_root = settings.crate_root_path();

                let outcome = self.run_check_command_via_rustup(
                    toolchain,
                    crate_root,
                    settings.check_cmd.rustup_command.iter().map(|s| s.as_str()),
                )?;

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

impl<'reporter, 'env, 'cc, R: Reporter> RustupToolchainCheck<'reporter, 'env, 'cc, R> {
    pub fn new(
        reporter: &'reporter R,
        ignore_lockfile: bool,
        no_check_feedback: bool,
        environment: &'env EnvironmentContext,
        check_cmd: &'cc CheckCmdContext,
    ) -> Self {
        Self {
            reporter,
            settings: Settings {
                ignore_lockfile,
                no_check_feedback,
                environment,
                check_cmd,
            },
        }
    }

    fn setup_toolchain(&self, toolchain: &ToolchainSpec) -> TResult<()> {
        let downloader = ToolchainDownloader::new(self.reporter);
        downloader.download(toolchain)?;

        Ok(())
    }

    fn run_check_command_via_rustup<'arg>(
        &self,
        toolchain: &'arg ToolchainSpec,
        dir: &Utf8Path,
        check: impl Iterator<Item = &'arg str>,
    ) -> TResult<Outcome> {
        // TODO(#824): Check MSRV against compilation target instead of build machine target

        let mut cmd: Vec<&str> = vec![toolchain.spec()];
        cmd.extend(check);

        self.reporter.report_event(CheckMethod::new(
            toolchain.to_owned(),
            Method::rustup_run(&cmd, dir),
        ))?;

        let rustup_output = RustupCommand::new()
            .with_args(cmd.iter())
            .with_dir(dir)
            .with_stderr()
            .run()
            .map_err(|_| CargoMSRVError::UnableToRunCheck {
                command: cmd[1..].join(" "),
                cwd: dir.to_path_buf(),
            })?;

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
}

/// Creates a lockfile handle, iff the lockfile exists and the user opted
/// to ignore it.
fn create_lockfile_handle(
    ignore_lockfile: bool,
    env: &EnvironmentContext,
) -> Option<LockfileHandler<lockfile::Start>> {
    ignore_lockfile
        .then(|| env.lock())
        .filter(|lockfile| lockfile.is_file())
        .map(LockfileHandler::new)
}

fn remove_lockfile(lock_file: &Utf8Path) -> TResult<()> {
    if lock_file.is_file() {
        std::fs::remove_file(lock_file).map_err(|error| IoError {
            error,
            source: IoErrorSource::RemoveFile(lock_file.to_path_buf()),
        })?;
    }

    Ok(())
}

struct Settings<'env, 'cc> {
    ignore_lockfile: bool,
    no_check_feedback: bool,

    environment: &'env EnvironmentContext,
    check_cmd: &'cc CheckCmdContext,
}

impl<'env, 'cc> Settings<'env, 'cc> {
    pub fn ignore_lockfile(&self) -> bool {
        self.ignore_lockfile
    }

    pub fn no_check_feedback(&self) -> bool {
        self.no_check_feedback
    }

    pub fn crate_root_path(&self) -> &Utf8Path {
        self.environment.root()
    }

    pub fn lockfile_path(&self) -> Utf8PathBuf {
        self.environment.lock()
    }
}
