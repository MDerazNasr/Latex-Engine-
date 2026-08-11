//! Worker command and supervision configuration.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use latex_render_core::{
    CacheLimits, MAX_HEIGHT_PX, MAX_JSON_LINE_BYTES, MAX_SCALE, MAX_SOURCE_BYTES, MAX_SVG_BYTES,
    MAX_WIDTH_PX, MIN_SCALE, RenderError, RenderErrorCode, RenderLimits,
};

/// Maximum queued expressions accepted by the MVP client.
pub const MAX_PENDING_RENDERS: usize = 32;

/// Executable and arguments used to start the local worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerCommand {
    /// Worker executable path or program name.
    pub program: PathBuf,
    /// Arguments passed without shell interpretation.
    pub args: Vec<OsString>,
    /// Optional working directory used by the worker.
    pub current_dir: Option<PathBuf>,
}

impl WorkerCommand {
    /// Creates a command without arguments or a working directory.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
        }
    }

    /// Appends one literal argument.
    pub fn arg(mut self, argument: impl Into<OsString>) -> Self {
        self.args.push(argument.into());
        self
    }

    /// Selects the worker working directory.
    pub fn current_dir(mut self, directory: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(directory.into());
        self
    }
}

/// Runtime policy for one supervised worker session.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerClientConfig {
    /// Executable used to start the worker.
    pub command: WorkerCommand,
    /// Maximum time allowed for the ready handshake.
    pub startup_timeout: Duration,
    /// Maximum time allowed for one render.
    pub render_timeout: Duration,
    /// Maximum time allowed for graceful process shutdown.
    pub shutdown_timeout: Duration,
    /// Minimum interval between worker restarts.
    pub restart_interval: Duration,
    /// Maximum queued render count.
    pub queue_capacity: usize,
    /// Shared request and result limits.
    pub render_limits: RenderLimits,
    /// Memory cache limits.
    pub cache_limits: CacheLimits,
    /// Expected renderer implementation name.
    pub expected_renderer_name: String,
    /// Expected renderer implementation version.
    pub expected_renderer_version: String,
    /// Fixed macro and extension policy version.
    pub macro_policy_version: String,
}

impl WorkerClientConfig {
    /// Creates the default MVP policy for a worker command.
    pub fn new(command: WorkerCommand) -> Self {
        Self {
            command,
            startup_timeout: Duration::from_millis(1500),
            render_timeout: Duration::from_secs(1),
            shutdown_timeout: Duration::from_millis(500),
            restart_interval: Duration::from_secs(5),
            queue_capacity: MAX_PENDING_RENDERS,
            render_limits: RenderLimits::default(),
            cache_limits: CacheLimits::default(),
            expected_renderer_name: "mathjax".to_owned(),
            expected_renderer_version: "0.1.0".to_owned(),
            macro_policy_version: "base-ams-1".to_owned(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), RenderError> {
        if self.command.program.as_os_str().is_empty() {
            return Err(invalid("Worker program must not be empty"));
        }
        if self.queue_capacity == 0 || self.queue_capacity > MAX_PENDING_RENDERS {
            return Err(invalid(format!(
                "Queue capacity must be between 1 and {MAX_PENDING_RENDERS}"
            )));
        }
        if self.startup_timeout.is_zero()
            || self.render_timeout.is_zero()
            || self.shutdown_timeout.is_zero()
            || self.restart_interval.is_zero()
        {
            return Err(invalid("Worker time limits must be greater than zero"));
        }
        if self.expected_renderer_name.is_empty()
            || self.expected_renderer_name.len() > 64
            || self.expected_renderer_version.is_empty()
            || self.expected_renderer_version.len() > 64
            || self.macro_policy_version.is_empty()
            || self.macro_policy_version.len() > 64
        {
            return Err(invalid("Worker version labels must contain 1 to 64 bytes"));
        }
        if self.cache_limits.max_entries == 0 || self.cache_limits.max_bytes == 0 {
            return Err(invalid("Cache limits must be greater than zero"));
        }
        let limits = &self.render_limits;
        if limits.max_source_bytes == 0
            || limits.max_source_bytes > MAX_SOURCE_BYTES
            || limits.max_json_line_bytes == 0
            || limits.max_json_line_bytes > MAX_JSON_LINE_BYTES
            || limits.max_svg_bytes == 0
            || limits.max_svg_bytes > MAX_SVG_BYTES
            || limits.max_width_px == 0
            || limits.max_width_px > MAX_WIDTH_PX
            || limits.max_height_px == 0
            || limits.max_height_px > MAX_HEIGHT_PX
            || !limits.min_scale.is_finite()
            || limits.min_scale < MIN_SCALE
            || !limits.max_scale.is_finite()
            || limits.max_scale > MAX_SCALE
            || limits.min_scale > limits.max_scale
        {
            return Err(invalid("Render limits exceed the worker protocol bounds"));
        }
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::InvalidRequest, message, false)
}
