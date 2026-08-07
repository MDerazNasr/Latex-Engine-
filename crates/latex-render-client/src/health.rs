//! Redacted worker health exposed to diagnostics.

use latex_render_core::RenderErrorCode;

/// Current lifecycle state of the worker supervisor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerState {
    /// No render has required a worker yet.
    Idle,
    /// A worker is starting and awaiting its handshake.
    Starting,
    /// A compatible worker is ready.
    Ready,
    /// Restart throttling is active after a failure.
    Backoff,
    /// The supervisor has shut down.
    Stopped,
}

/// Source free health information for status commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerHealth {
    /// Current lifecycle state.
    pub state: WorkerState,
    /// Renderer version reported by the accepted handshake.
    pub renderer_version: Option<String>,
    /// Number of supervised restart attempts.
    pub restart_count: u64,
    /// Most recent public error category.
    pub last_error: Option<RenderErrorCode>,
}

impl Default for WorkerHealth {
    fn default() -> Self {
        Self {
            state: WorkerState::Idle,
            renderer_version: None,
            restart_count: 0,
            last_error: None,
        }
    }
}
