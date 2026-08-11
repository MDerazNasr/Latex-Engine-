//! Output writing that preserves existing files by default.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{CliError, CliErrorKind};

static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(1);

pub(crate) struct CommandOutput {
    pub(crate) bytes: Vec<u8>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) force: bool,
}

pub(crate) fn write_output(output: CommandOutput) -> Result<(), CliError> {
    let Some(path) = output.path else {
        return write_stdout(&output.bytes);
    };
    write_file(&path, &output.bytes, output.force)
}

fn write_file(path: &Path, bytes: &[u8], force: bool) -> Result<(), CliError> {
    let (temporary_path, mut file) = create_temporary(path)?;
    if file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(output_error("Output file could not be written completely"));
    }
    drop(file);
    if force {
        return fs::rename(&temporary_path, path).map_err(|_| {
            let _ = fs::remove_file(&temporary_path);
            output_error("Output file could not be replaced atomically")
        });
    }
    if fs::hard_link(&temporary_path, path).is_err() {
        let _ = fs::remove_file(&temporary_path);
        return Err(output_error(
            "Output file already exists or cannot be linked",
        ));
    }
    fs::remove_file(temporary_path)
        .map_err(|_| output_error("Output completed but temporary cleanup failed"))
}

fn create_temporary(path: &Path) -> Result<(PathBuf, File), CliError> {
    let file_name = path
        .file_name()
        .ok_or_else(|| output_error("Output path must name a file"))?;
    for _ in 0..64 {
        let id = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(file_name);
        temporary_name.push(format!(".latex-render.{}.{id}.tmp", std::process::id()));
        let temporary_path = path.with_file_name(temporary_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(_) => {
                return Err(output_error(
                    "Temporary output file could not be created safely",
                ));
            }
        }
    }
    Err(output_error(
        "A unique temporary output file could not be created",
    ))
}

pub(crate) fn write_stdout(bytes: &[u8]) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    locked
        .write_all(bytes)
        .and_then(|()| locked.flush())
        .map_err(|_| output_error("Standard output could not be written"))
}

fn output_error(message: impl Into<String>) -> CliError {
    CliError::new(CliErrorKind::Output, message)
}
