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
    pub fn execute(mut self, cmd: &OsStr) -> TResult<RustupOutput> {
        debug!(
            cmd = ?cmd,
            args = ?self.args.as_slice()
        );

        self.command.arg(cmd);
        self.command.args(self.args);

        self.command.stdout(self.stdout);
        self.command.stderr(self.stderr);

        // todo:
        //     cwd: Some(
        //         "/home/a/cargo-msrv/tests/fixtures/1.35.0",
        //     ),
        // however:
        //     error: failed to parse manifest at `/home/a/cargo-msrv/Cargo.toml
        // whaaat?
        // -> Turns out, if I copy the fixtures outside of cargo-msrv it works fine
        // -> For some reason, the cwd is getting overwritten.
        // -> I checked the edition notes, but they do not mention a change
        //
        // It happens in all these places:
        // - running cargo test (in some integration tests in the tests directory, ... where edition of test crate < cargo-msrv's (or top level Cargo.toml)
        // - running cargo run -- msrv --manifest-path tests/fixtures/1.35.0, ... where edition of test crate < cargo-msrv's (or top level Cargo.toml)
        // - running cargo msrv (after installing the version which uses edition 2021), ... where edition crate < cargo-msrv's (or top level Cargo.toml)
        // -> ... where edition crate < cargo-msrv's (or top level Cargo.toml)
        // -> -> Running rustup run <toolchain> <cargo cmd> with cwd = (an inner crate within an outer crate) just reads the outer crate it seems
        // -> -> But this only seems to happen from edition 2021 forward
        // -> -> Didn't see anything about this in the Cargo changelog or edition notes...
        //
        // Whether it is a smart thing to add crates within crates (which aren't part of the workspace), 🤷
        // But it used to work...
        //
        // -> Confirmed with 'cargo new --lib outer && cd outer && cargo new --lib inner' and setting inner edition to 2018
        // -> Then running cargo msrv inside 'inner'
        //
        // When using cargo-msrv 0.15.1 (latest on crates.io), and testing the above, it seems like it
        // used to work for tests, not normal runs...?
        // TODO: remove dbg
        dbg!(&self.command);

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
