//! Defines stable package construction failures.

use std::fmt;
use std::io;

#[derive(Debug)]
/// Reports one source safe packaging failure.
pub struct PackageErrorV1 {
    message: String,
}

impl PackageErrorV1 {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn io(context: &str, error: io::Error) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for PackageErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PackageErrorV1 {}
