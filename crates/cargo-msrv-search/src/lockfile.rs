use crate::error::{IoError, IoErrorSource, LockfileHandlerError, TResult};
use camino::{Utf8Path, Utf8PathBuf};
use std::sync::Mutex;

static RESTORE: Mutex<Option<(Utf8PathBuf, Utf8PathBuf)>> = Mutex::new(None);

/// Must be called once to set up the lockfile restoration handler
pub fn init_lockfile_cleanup_handler() -> Result<(), LockfileHandlerError> {
    match ctrlc::set_handler(|| {
        restore();
    }) {
        Ok(()) => Ok(()),
        // only init once, if there are multiple processes playing with the lockfile we could have a problem
        Err(ctrlc::Error::MultipleHandlers) => Ok(()),
        Err(_) => Err(LockfileHandlerError),
    }
}

fn with_lock<F, R>(f: F) -> R
where
    F: FnOnce(&mut Option<(Utf8PathBuf, Utf8PathBuf)>) -> R,
{
    let mut guard = RESTORE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    f(&mut guard)
}

fn restore() {
    with_lock(|opt| {
        if let Some((temp, original)) = opt.take() {
            let _ = std::fs::rename(temp, original);
        }
    });
}

fn register(temp: Utf8PathBuf, original: Utf8PathBuf) {
    with_lock(|opt| *opt = Some((temp, original)));
}

fn deregister() {
    with_lock(|opt| *opt = None);
}

const CARGO_LOCK_REPLACEMENT: &str = "Cargo.lock-ignored-for-cargo-msrv";

pub struct LockfileHandler {
    original: Utf8PathBuf,
    temp: Utf8PathBuf,
}

impl LockfileHandler {
    pub fn try_new<P: AsRef<Utf8Path>>(lock_file: P) -> Result<Self, LockfileHandlerError> {
        let original = lock_file.as_ref().to_path_buf();
        let temp = original.parent().unwrap().join(CARGO_LOCK_REPLACEMENT);

        init_lockfile_cleanup_handler()?;

        Ok(Self { original, temp })
    }

    pub fn move_lockfile(self) -> TResult<MovedLockfile> {
        std::fs::rename(&self.original, &self.temp).map_err(|error| IoError {
            error,
            source: IoErrorSource::RenameFile(self.original.clone()),
        })?;

        register(self.temp.clone(), self.original.clone());

        Ok(MovedLockfile {
            original: self.original,
            temp: self.temp,
        })
    }
}

pub struct MovedLockfile {
    original: Utf8PathBuf,
    temp: Utf8PathBuf,
}

impl Drop for MovedLockfile {
    fn drop(&mut self) {
        let _ = std::fs::rename(&self.temp, &self.original);
        deregister();
    }
}
