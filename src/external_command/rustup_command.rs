use crate::error::{IoError, IoErrorSource, TResult};
use camino::Utf8Path;
use std::ffi::{OsStr, OsString};
use std::process::{Command, Stdio};

pub struct RustupCommand {
    command: Command,
    args: Vec<OsString>,
    stdout: Stdio,
    stderr: Stdio,
    // Span used to trace the path to execution of a RustupCommand
    _span: tracing::Span,
}

impl RustupCommand {
    pub fn new() -> Self {
        Self {
            command: Command::new("rustup"),
            args: Vec::new(),
            stdout: Stdio::null(),
            stderr: Stdio::null(),
            _span: tracing::debug_span!("RustupCommand"),
        }
    }

    pub fn with_dir(mut self, path: impl AsRef<Utf8Path>) -> Self {
        // The block ensures the scope of the _span.enter() will exit before
        // returning self at the end of the method scope.
        {
            let _span = self._span.enter();

            let path = path.as_ref();

            if let Ok(canonical_path) = path.canonicalize() {
                debug!(name: "rustup_command_path", canonicalized = true, path = %canonical_path.display());
                self.command.current_dir(canonical_path);
            } else {
                debug!(name: "rustup_command_path", canonicalized = false, %path);
                self.command.current_dir(path);
            }
        }

        self
    }

    pub fn with_args<T: Into<OsString>>(mut self, args: impl IntoIterator<Item = T>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn with_stdout(mut self) -> Self {
        self.stdout = Stdio::piped();
        self
    }

    pub fn with_stderr(mut self) -> Self {
        self.stderr = Stdio::piped();
        self
    }

    /// Execute `rustup run [...]`
    pub fn run(self) -> TResult<RustupOutput> {
        self.execute(OsStr::new("run"))
    }

    /// Execute `rustup install [...]`
    pub fn install(self) -> TResult<RustupOutput> {
        self.execute(OsStr::new("install"))
    }

    /// Execute `rustup show [...]`
    pub fn show(self) -> TResult<RustupOutput> {
        self.execute(OsStr::new("show"))
    }

    pub fn target(self) -> TResult<RustupOutput> {
        self.execute(OsStr::new("target"))
    }

    pub fn component(self) -> TResult<RustupOutput> {
        self.execute(OsStr::new("component"))
    }

    /// Execute a given `rustup` command.
    ///
    /// See also:
    /// * [RustupCommand::run](RustupCommand::run)
    /// * [RustupCommand::install](RustupCommand::install)
    /// * [RustupCommand::show](RustupCommand::show)
    fn execute(mut self, cmd: &OsStr) -> TResult<RustupOutput> {
        let _span = self._span.enter();

        debug!(
            name: "rustup_command_execute_start",
            cmd = ?cmd,
            args = ?self.args.as_slice(),
            current_dir = ?self.command.get_current_dir(),
        );

        self.command.arg(cmd);
        self.command.args(self.args);

        self.command.stdout(self.stdout);
        self.command.stderr(self.stderr);

        let child = self.command.spawn().map_err(|error| IoError {
            error,
            source: IoErrorSource::SpawnProcess(cmd.to_owned()),
        })?;

        let output = child.wait_with_output().map_err(|error| IoError {
            error,
            source: IoErrorSource::WaitForProcessAndCollectOutput(cmd.to_owned()),
        })?;

        debug!(
            name: "rustup_command_execute_finish",
            cmd = ?cmd,
            success = output.status.success(),
        );

        Ok(RustupOutput {
            output,
            stdout: std::cell::OnceCell::new(),
            stderr: std::cell::OnceCell::new(),
        })
    }
}

pub struct RustupOutput {
    output: std::process::Output,
    stdout: std::cell::OnceCell<String>,
    stderr: std::cell::OnceCell<String>,
}

impl RustupOutput {
    pub fn stdout(&self) -> &str {
        self.stdout
            .get_or_init(|| String::from_utf8_lossy(&self.output.stdout).into_owned())
            .as_str()
    }

    pub fn stderr(&self) -> &str {
        self.stderr
            .get_or_init(|| String::from_utf8_lossy(&self.output.stderr).into_owned())
            .as_str()
    }

    pub fn exit_status(&self) -> std::process::ExitStatus {
        self.output.status
    }
}
