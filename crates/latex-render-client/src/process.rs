//! Lifecycle owner for one worker child process.

use std::process::Stdio;
use std::time::Duration;

use latex_render_core::{RenderError, RenderErrorCode, RenderRequest, RenderedMath};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::WorkerClientConfig;
use crate::line_reader::read_bounded_line;
use crate::protocol::{decode_ready, decode_response, encode_request, response_line_limit};

pub(crate) struct WorkerProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    line: Vec<u8>,
}

impl WorkerProcess {
    pub(crate) async fn spawn(config: &WorkerClientConfig) -> Result<(Self, String), RenderError> {
        let mut command = Command::new(&config.command.program);
        command
            .args(&config.command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if let Some(current_dir) = &config.command.current_dir {
            command.current_dir(current_dir);
        }

        let mut child = command.spawn().map_err(|_| {
            RenderError::new(
                RenderErrorCode::WorkerUnavailable,
                "Renderer worker could not be started",
                true,
            )
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            RenderError::new(
                RenderErrorCode::WorkerUnavailable,
                "Renderer worker stdin is unavailable",
                true,
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            RenderError::new(
                RenderErrorCode::WorkerUnavailable,
                "Renderer worker stdout is unavailable",
                true,
            )
        })?;
        let mut process = Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            line: Vec::with_capacity(4096),
        };

        let ready = timeout(config.startup_timeout, process.read_ready(config)).await;
        match ready {
            Ok(Ok(version)) => Ok((process, version)),
            Ok(Err(error)) => {
                process.terminate(config.shutdown_timeout).await?;
                Err(error)
            }
            Err(_) => {
                process.terminate(config.shutdown_timeout).await?;
                Err(RenderError::new(
                    RenderErrorCode::Timeout,
                    "Renderer worker did not become ready before its deadline",
                    true,
                ))
            }
        }
    }

    pub(crate) async fn render(
        &mut self,
        id: &str,
        request: &RenderRequest,
        cache_key: String,
        config: &WorkerClientConfig,
    ) -> Result<RenderedMath, RenderError> {
        let encoded = encode_request(id, request, &config.render_limits)?;
        let stdin = self.stdin.as_mut().ok_or_else(worker_closed)?;
        stdin
            .write_all(&encoded)
            .await
            .map_err(|_| worker_closed())?;
        stdin.write_all(b"\n").await.map_err(|_| worker_closed())?;
        stdin.flush().await.map_err(|_| worker_closed())?;

        read_bounded_line(
            &mut self.stdout,
            &mut self.line,
            response_line_limit(&config.render_limits),
        )
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::InvalidData {
                RenderError::new(
                    RenderErrorCode::OutputLimitExceeded,
                    "Renderer worker response exceeded its byte limit",
                    false,
                )
            } else {
                worker_closed()
            }
        })?;
        decode_response(&self.line, id, cache_key, &config.render_limits)
    }

    pub(crate) async fn terminate(
        &mut self,
        shutdown_timeout: Duration,
    ) -> Result<(), RenderError> {
        self.stdin.take();
        match timeout(shutdown_timeout, self.child.wait()).await {
            Ok(Ok(_)) => return Ok(()),
            Ok(Err(_)) => return Err(reap_failed()),
            Err(_) => {}
        }
        self.child.start_kill().map_err(|_| reap_failed())?;
        match timeout(shutdown_timeout, self.child.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) | Err(_) => Err(reap_failed()),
        }
    }

    async fn read_ready(&mut self, config: &WorkerClientConfig) -> Result<String, RenderError> {
        read_bounded_line(
            &mut self.stdout,
            &mut self.line,
            config.render_limits.max_json_line_bytes,
        )
        .await
        .map_err(|_| worker_closed())?;
        decode_ready(&self.line, config)
    }
}

fn worker_closed() -> RenderError {
    RenderError::new(
        RenderErrorCode::WorkerUnavailable,
        "Renderer worker closed its protocol stream",
        true,
    )
}

fn reap_failed() -> RenderError {
    RenderError::new(
        RenderErrorCode::WorkerUnavailable,
        "Renderer worker could not be reaped",
        false,
    )
}
