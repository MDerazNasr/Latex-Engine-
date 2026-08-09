//! Output writing that preserves existing files by default.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::error::{CliError, CliErrorKind};

pub(crate) struct CommandOutput {
    pub(crate) bytes: Vec<u8>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) force: bool,
}

pub(crate) fn write_output(output: CommandOutput) -> Result<(), CliError> {
    let Some(path) = output.path else {
        return write_stdout(&output.bytes);
    };
    let mut options = OpenOptions::new();
    options.write(true);
    if output.force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options
        .open(path)
        .map_err(|_| output_error("Output file could not be created without data loss"))?;
    file.write_all(&output.bytes)
        .and_then(|()| file.flush())
        .map_err(|_| output_error("Output file could not be written"))
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
