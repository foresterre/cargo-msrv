use crate::errors::TResult;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub fn command_with_output<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
) -> TResult<Child> {
    command_impl(commands, None)
        .pipe_output()
        .spawn()
        .map_err(From::from)
}

pub fn command<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
    dir: Option<&Path>,
) -> TResult<Child> {
    command_impl(commands, dir)
        .nullify_output()
        .spawn()
        .map_err(From::from)
}

trait PipeCliOutput {
    fn pipe_output(&mut self) -> &mut Command;
}

impl PipeCliOutput for Command {
    fn pipe_output(&mut self) -> &mut Command {
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped())
    }
}

trait NullifyCliOutput {
    fn nullify_output(&mut self) -> &mut Command;
}

impl NullifyCliOutput for Command {
    fn nullify_output(&mut self) -> &mut Command {
        self.stdout(Stdio::null());
        self.stderr(Stdio::null())
    }
}

fn command_impl<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
    current_dir: Option<&Path>,
) -> Command {
    let mut cmd = Command::new("rustup");
    let _ = cmd.args(commands);

    if let Some(dir) = current_dir {
        let _ = cmd.current_dir(dir);
    }

    cmd
}
