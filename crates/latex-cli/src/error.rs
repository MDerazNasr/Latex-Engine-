//! Stable source free command errors and exit statuses.

use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliErrorKind {
    Usage,
    Internal,
}

impl CliErrorKind {
    pub(crate) const fn exit_code(self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::Internal => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliError {
    kind: CliErrorKind,
    message: String,
}

impl CliError {
    pub(crate) fn new(kind: CliErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) const fn kind(&self) -> CliErrorKind {
        self.kind
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
