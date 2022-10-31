use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{IoError, IoErrorSource, TResult};

pub struct RustupCommand {
    command: Command,
    args: Vec<OsString>,
    stdout: Stdio,
    stderr: Stdio,
}

impl RustupCommand {
    pub fn new() -> Self {
        Self {
            command: Command::new("rustup"),
            args: Vec::new(),
            stdout: Stdio::null(),
            stderr: Stdio::null(),
        }
    }

    pub fn with_dir(mut self, path: impl AsRef<Path>) -> Self {
        let _ = self.command.current_dir(path);
        self
    }

    pub fn with_optional_dir(self, path: Option<impl AsRef<Path>>) -> Self {
        if let Some(dir) = path {
            return self.with_dir(dir);
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

    /// Execute a given `rustup` command.
    ///
    /// See also:
    /// * [RustupCommand::run](RustupCommand::run)
    /// * [RustupCommand::install](RustupCommand::run)
    /// * [RustupCommand::show](RustupCommand::run)
    pub fn execute(mut self, cmd: &OsStr) -> TResult<RustupOutput> {
        debug!(
            cmd = ?cmd,
            args = ?self.args.as_slice()
        );

        self.command.arg(&cmd);
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

        Ok(RustupOutput {
            output,
            stdout: once_cell::sync::OnceCell::new(),
            stderr: once_cell::sync::OnceCell::new(),
        })
    }
}

pub struct RustupOutput {
    output: std::process::Output,
    stdout: once_cell::sync::OnceCell<String>,
    stderr: once_cell::sync::OnceCell<String>,
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
