use crate::compatibility::IsCompatible;
use crate::context::EnvironmentContext;
use crate::error::{IoError, IoErrorSource};
use crate::external_command::cargo_command::CargoCommand;
use crate::external_command::rustup_command::RustupCommand;
use crate::lockfile::LockfileHandler;
use crate::outcome::Incompatible;
use crate::reporter::event::{CheckMethod, CheckResult, CheckToolchain, Method};
use crate::rust::setup_toolchain::{SetupRustupToolchain, SetupToolchain};
use crate::rust::Toolchain;
use crate::{lockfile, CargoMSRVError, Compatibility, Reporter, TResult};
use camino::{Utf8Path, Utf8PathBuf};
use std::fmt;
use std::fmt::Formatter;

pub struct RustupToolchainCheck<'reporter, 'env, R: Reporter> {
    reporter: &'reporter R,
    settings: Settings<'env>,
}

impl<'reporter, 'env, R: Reporter> RustupToolchainCheck<'reporter, 'env, R> {
    pub fn new(
        reporter: &'reporter R,
        ignore_lockfile: bool,
        no_check_feedback: bool,
        skip_unavailable_toolchains: bool,
        environment: &'env EnvironmentContext,
        run_command: RunCommand,
    ) -> Self {
        Self {
            reporter,
            settings: Settings {
                ignore_lockfile,
                no_check_feedback,
                skip_unavailable_toolchains,
                environment,
                check_cmd: run_command,
            },
        }
    }
}

impl<R: Reporter> IsCompatible for RustupToolchainCheck<'_, '_, R> {
    fn is_compatible(&self, toolchain: &Toolchain) -> TResult<Compatibility> {
        let settings = &self.settings;

        self.reporter
            .run_scoped_event(CheckToolchain::new(toolchain.to_owned()), || {
                info!(ignore_lockfile_enabled = settings.ignore_lockfile());

                // temporarily move the lockfile if the user opted to ignore it, and it exists
                let ignore_lockfile = settings.ignore_lockfile();
                let handle_wrap = create_lockfile_handle(ignore_lockfile, settings.environment)
                    .map(|handle| handle.move_lockfile())
                    .transpose()?;

                // Exit early, while marking this version as incompatible, when `skip_unavailable_toolchains`
                // is set and the setup failed.
                match setup_toolchain(self.reporter, toolchain) {
                    Ok(()) => Ok(()),
                    Err(err) if settings.skip_unavailable_toolchains() => {
                        return Ok(Compatibility::Incompatible(Incompatible {
                            toolchain_spec: toolchain.clone(),
                            error_message: err.to_string(),
                        }));
                    }
                    Err(err) => Err(err),
                }?;

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

impl<R: Reporter> fmt::Debug for RustupToolchainCheck<'_, '_, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.settings))
    }
}

fn setup_toolchain(reporter: &impl Reporter, toolchain: &Toolchain) -> TResult<()> {
    let downloader = SetupRustupToolchain::new(reporter);
    downloader.download(toolchain)?;

    Ok(())
}

fn run_check_command_via_rustup(
    reporter: &impl Reporter,
    toolchain: &Toolchain,
    dir: &Utf8Path,
    check: &[String],
) -> TResult<Compatibility> {
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
        Ok(Compatibility::new_success(toolchain.to_owned()))
    } else {
        let stderr = rustup_output.stderr();
        let command = cmd.join(" ");

        info!(
            ?toolchain,
            stderr,
            cmd = command.as_str(),
            "try_building run failed"
        );

        Ok(Compatibility::new_failure(
            toolchain.to_owned(),
            stderr.to_string(),
        ))
    }
}

fn report_outcome(
    reporter: &impl Reporter,
    outcome: &Compatibility,
    no_error_report: bool,
) -> TResult<()> {
    match outcome {
        Compatibility::Compatible(outcome) => {
            // report compatibility with this toolchain
            reporter.report_event(CheckResult::compatible(outcome.toolchain_spec.to_owned()))?
        }
        Compatibility::Incompatible(outcome) if no_error_report => {
            // report incompatibility with this toolchain
            reporter.report_event(CheckResult::incompatible(
                outcome.toolchain_spec.to_owned(),
                None,
            ))?
        }
        Compatibility::Incompatible(outcome) => {
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

#[derive(Debug)]
struct Settings<'env> {
    ignore_lockfile: bool,
    no_check_feedback: bool,
    skip_unavailable_toolchains: bool,

    environment: &'env EnvironmentContext,
    check_cmd: RunCommand,
}

impl Settings<'_> {
    pub fn ignore_lockfile(&self) -> bool {
        self.ignore_lockfile
    }

    pub fn no_check_feedback(&self) -> bool {
        self.no_check_feedback
    }

    pub fn skip_unavailable_toolchains(&self) -> bool {
        self.skip_unavailable_toolchains
    }

    pub fn crate_root_path(&self) -> &Utf8Path {
        self.environment.root()
    }

    pub fn lockfile_path(&self) -> Utf8PathBuf {
        self.environment.lock()
    }
}

#[derive(Debug)]
pub struct RunCommand {
    command: Vec<String>,
}

impl RunCommand {
    pub fn from_cargo_command(cargo_command: CargoCommand) -> Self {
        Self {
            command: cargo_command.into_args(),
        }
    }

    pub fn custom(command: Vec<String>) -> Self {
        Self { command }
    }

    pub fn components(&self) -> &[String] {
        self.command.as_ref()
    }
}
