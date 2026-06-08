use crate::paths;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use track_host_wit::track::host::project_lock::{Error, ErrorCode};

pub struct HeldLock {
    file: File,
}

pub fn acquire(project_root: &Path, blocking: bool) -> Result<HeldLock, Error> {
    let lock_dir = project_root.join(".track");
    std::fs::create_dir_all(&lock_dir).map_err(io_error)?;
    let lock_path = paths::state_lock_path(project_root);
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(io_error)?;

    if blocking {
        file.lock_exclusive().map_err(map_lock_error)?;
    } else {
        file.try_lock_exclusive().map_err(map_try_lock_error)?;
    }

    file.set_len(0).map_err(io_error)?;
    writeln!(file, "{}", std::process::id()).map_err(io_error)?;

    Ok(HeldLock { file })
}

impl HeldLock {
    pub fn release(self) -> Result<(), Error> {
        self.file.unlock().map_err(io_error)?;
        Ok(())
    }
}

fn map_try_lock_error(err: std::io::Error) -> Error {
    if err.kind() == std::io::ErrorKind::WouldBlock {
        Error {
            code: ErrorCode::Unavailable,
            message: "project state lock is held by another process".into(),
        }
    } else {
        io_error(err)
    }
}

fn map_lock_error(err: std::io::Error) -> Error {
    Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    }
}

fn io_error(err: impl std::fmt::Display) -> Error {
    Error {
        code: ErrorCode::IoError,
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn exclusive_lock_blocks_second_nonblocking_acquire() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let _first = acquire(root, false).unwrap();
        match acquire(root, false) {
            Err(err) => assert_eq!(err.code, ErrorCode::Unavailable),
            Ok(_) => panic!("expected second acquire to fail"),
        }
    }

    #[test]
    fn release_allows_reacquire() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let lock = acquire(root, false).unwrap();
        lock.release().unwrap();
        assert!(acquire(root, false).is_ok());
    }

    #[test]
    fn blocking_acquire_waits_for_release() {
        let dir = Arc::new(TempDir::new().unwrap());
        let root = dir.path().to_path_buf();
        let _first = acquire(&root, false).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let barrier_child = barrier.clone();
        let root_child = root.clone();
        let handle = thread::spawn(move || {
            barrier_child.wait();
            let lock = acquire(&root_child, true).unwrap();
            lock.release().unwrap();
        });
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(_first);
        barrier.wait();
        handle.join().unwrap();
    }
}
