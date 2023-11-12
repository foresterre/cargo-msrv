use crate::check::Check;
use crate::command::RustupCommand;
use crate::context::EnvironmentContext;
use crate::download::{DownloadToolchain, ToolchainDownloader};
use crate::error::{IoError, IoErrorSource};
use crate::lockfile::LockfileHandler;
use crate::reporter::event::{CheckMethod, CheckResult, CheckToolchain, Method};
use crate::toolchain::ToolchainSpec;
use crate::{lockfile, CargoMSRVError, Outcome, Reporter, TResult};
use camino::{Utf8Path, Utf8PathBuf};

pub struct RustupToolchainCheck<'reporter, 'env, R: Reporter> {
    reporter: &'reporter R,
    settings: Settings<'env>,
}

impl<'reporter, 'env, R: Reporter> RustupToolchainCheck<'reporter, 'env, R> {
    pub fn new(
        reporter: &'reporter R,
        ignore_lockfile: bool,
        no_check_feedback: bool,
        environment: &'env EnvironmentContext,
        run_command: RunCommand,
    ) -> Self {
        Self {
            reporter,
            settings: Settings {
                ignore_lockfile,
                no_check_feedback,
                environment,
                check_cmd: run_command,
            },
        }
    }
}

impl<'reporter, 'env, R: Reporter> Check for RustupToolchainCheck<'reporter, 'env, R> {
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

                setup_toolchain(self.reporter, toolchain)?;

                if handle_wrap.is_some() {
                    remove_lockfile(&settings.lockfile_path())?;
                }

                let crate_root = settings.crate_root_path();
                let cmd = &self.settings.check_cmd;

                let outcome = run_check_command_via_rustup(
                    self.reporter,
                    toolchain,
                    crate_root,
                    cmd.components(),
                )?;

                // report outcome to UI
                report_outcome(self.reporter, &outcome, settings.no_check_feedback())?;

                // move the lockfile back
                if let Some(handle) = handle_wrap {
                    handle.move_lockfile_back()?;
                }

                Ok(outcome)
            })
    }
}

fn setup_toolchain(reporter: &impl Reporter, toolchain: &ToolchainSpec) -> TResult<()> {
    let downloader = ToolchainDownloader::new(reporter);
    downloader.download(toolchain)?;

    Ok(())
}

fn run_check_command_via_rustup(
    reporter: &impl Reporter,
    toolchain: &ToolchainSpec,
    dir: &Utf8Path,
    check: &[String],
) -> TResult<Outcome> {
    let version = format!("{}", toolchain.version());
    let mut cmd = vec![version.as_str()];
    cmd.extend(check.iter().map(|s| s.as_str()));

    reporter.report_event(CheckMethod::new(
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

fn report_outcome(
    reporter: &impl Reporter,
    outcome: &Outcome,
    no_error_report: bool,
) -> TResult<()> {
    match outcome {
        Outcome::Success(outcome) => {
            // report compatibility with this toolchain
            reporter.report_event(CheckResult::compatible(outcome.toolchain_spec.to_owned()))?
        }
        Outcome::Failure(outcome) if no_error_report => {
            // report incompatibility with this toolchain
            reporter.report_event(CheckResult::incompatible(
                outcome.toolchain_spec.to_owned(),
                None,
            ))?
        }
        Outcome::Failure(outcome) => {
            // report incompatibility with this toolchain
            reporter.report_event(CheckResult::incompatible(
                outcome.toolchain_spec.to_owned(),
                Some(outcome.error_message.clone()),
            ))?
        }
    };

    Ok(())
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

struct Settings<'env> {
    ignore_lockfile: bool,
    no_check_feedback: bool,

    environment: &'env EnvironmentContext,
    check_cmd: RunCommand,
}

impl<'env> Settings<'env> {
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

pub struct RunCommand {
    command: Vec<String>,
}

impl RunCommand {
    pub fn default(target: impl ToString) -> Self {
        let command = vec![
            "cargo".to_string(),
            "check".to_string(),
            "--target".to_string(),
            target.to_string(),
        ];

        Self { command }
    }

    pub fn custom(command: Vec<String>) -> Self {
        Self { command }
    }

    pub fn components(&self) -> &[String] {
        self.command.as_ref()
    }
}
