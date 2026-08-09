//! Stable source free command errors and exit statuses.

use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliErrorKind {
    Usage,
    Worker,
    Render,
    Output,
    Internal,
}

impl CliErrorKind {
    pub(crate) const fn exit_code(self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::Worker => 3,
            Self::Render => 4,
            Self::Output => 5,
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
            message: message
                .into()
                .chars()
                .map(|character| {
                    if character.is_control() {
                        '\u{fffd}'
                    } else {
                        character
                    }
                })
                .collect(),
        }
    }

    pub(crate) const fn kind(&self) -> CliErrorKind {
        self.kind
    }

    pub(crate) fn from_render(error: latex_render_core::RenderError) -> Self {
        use latex_render_core::RenderErrorCode;

        let kind = match error.code {
            RenderErrorCode::InvalidRequest | RenderErrorCode::InputLimitExceeded => {
                CliErrorKind::Usage
            }
            RenderErrorCode::Protocol
            | RenderErrorCode::WorkerUnavailable
            | RenderErrorCode::Timeout => CliErrorKind::Worker,
            _ => CliErrorKind::Render,
        };
        Self::new(kind, error.message)
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
