use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::error::{CargoMSRVError, IoError, IoErrorSource, TResult};

pub struct LockfileHandler<S: LockfileState> {
    state: PathBuf,
    marker: PhantomData<S>,
}

pub struct Start;
pub struct Moved;
pub struct Complete;

pub trait LockfileState {}
impl LockfileState for Start {}
impl LockfileState for Moved {}
impl LockfileState for Complete {}

pub const CARGO_LOCK: &str = "Cargo.lock";
const CARGO_LOCK_REPLACEMENT: &str = "Cargo.lock-ignored-for-cargo-msrv";

impl LockfileHandler<Start> {
    pub fn new<P: AsRef<Path>>(lock_file: P) -> Self {
        Self {
            state: lock_file.as_ref().to_path_buf(),
            marker: PhantomData,
        }
    }

    pub fn move_lockfile(self) -> TResult<LockfileHandler<Moved>> {
        let folder = self.state.parent().unwrap();
        std::fs::rename(self.state.as_path(), folder.join(CARGO_LOCK_REPLACEMENT)).map_err(
            |error| IoError {
                error,
                source: IoErrorSource::RenameFile(self.state.clone()),
            },
        )?;

        Ok(LockfileHandler {
            state: self.state,
            marker: PhantomData,
        })
    }
}

impl LockfileHandler<Moved> {
    pub fn move_lockfile_back(self) -> TResult<LockfileHandler<Complete>> {
        let folder = self.state.parent().unwrap();
        std::fs::rename(folder.join(CARGO_LOCK_REPLACEMENT), self.state.as_path()).map_err(
            |err| IoError {
                error: err,
                source: IoErrorSource::RenameFile(self.state.clone()),
            },
        )?;

        Ok(LockfileHandler {
            state: self.state,
            marker: PhantomData,
        })
    }
}
