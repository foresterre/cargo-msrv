use crate::errors::TResult;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub fn command_with_output<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
) -> TResult<Child> {
    command_impl(commands, StreamType::Piped, None)
        .spawn()
        .map_err(From::from)
}

pub fn command<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
    dir: Option<&Path>,
) -> TResult<Child> {
    command_impl(commands, StreamType::Null, dir)
        .spawn()
        .map_err(From::from)
}

pub enum StreamType {
    Piped,
    Null,
}

fn command_impl<I: IntoIterator<Item = V>, V: AsRef<OsStr>>(
    commands: I,
    io: StreamType,
    current_dir: Option<&Path>,
) -> Command {
    let mut cmd = Command::new("rustup");
    let _ = cmd.args(commands);

    match io {
        StreamType::Piped => {
            let _ = cmd.stdout(Stdio::piped());
            let _ = cmd.stderr(Stdio::piped());
        }
        StreamType::Null => {
            let _ = cmd.stdout(Stdio::null());
            let _ = cmd.stderr(Stdio::null());
        }
    }

    if let Some(dir) = current_dir {
        let _ = cmd.current_dir(dir);
    }

    cmd
}
