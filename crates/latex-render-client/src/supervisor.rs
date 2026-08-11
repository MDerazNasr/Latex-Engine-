//! Serialized queue consumer and worker restart policy.

use std::sync::Arc;
use std::time::Instant;

use latex_render_core::{RenderError, RenderErrorCode, RenderRequest, RenderedMath};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::timeout;

use crate::WorkerClientConfig;
use crate::health::{WorkerHealth, WorkerState};
use crate::process::WorkerProcess;

pub(crate) struct RenderJob {
    pub(crate) id: String,
    pub(crate) request: RenderRequest,
    pub(crate) cache_key: String,
    pub(crate) reply: oneshot::Sender<Result<RenderedMath, RenderError>>,
}

pub(crate) async fn run_supervisor(
    config: WorkerClientConfig,
    mut jobs: mpsc::Receiver<RenderJob>,
    health: Arc<Mutex<WorkerHealth>>,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), RenderError> {
    let mut worker: Option<WorkerProcess> = None;
    let mut restart_gate = RestartGate::new(config.restart_interval);

    loop {
        let job = tokio::select! {
            _ = &mut shutdown => break,
            job = jobs.recv() => {
                let Some(job) = job else {
                    break;
                };
                job
            }
        };
        if job.reply.is_closed() {
            continue;
        }
        let outcome = tokio::select! {
            _ = &mut shutdown => {
                let _ = job.reply.send(Err(cancelled_error()));
                break;
            }
            outcome = run_job(
                &config,
                &health,
                &mut worker,
                &mut restart_gate,
                &job,
            ) => outcome,
        };
        let _ = job.reply.send(outcome);
    }

    let shutdown_result = match worker {
        Some(mut process) => process.terminate(config.shutdown_timeout).await,
        None => Ok(()),
    };
    let mut current = health.lock().await;
    current.state = WorkerState::Stopped;
    if let Err(error) = &shutdown_result {
        current.last_error = Some(error.code);
    }
    shutdown_result
}

async fn run_job(
    config: &WorkerClientConfig,
    health: &Arc<Mutex<WorkerHealth>>,
    worker: &mut Option<WorkerProcess>,
    restart_gate: &mut RestartGate,
    job: &RenderJob,
) -> Result<RenderedMath, RenderError> {
    for attempt in 0..=1 {
        if worker.is_none() {
            if attempt == 0 && !restart_gate.can_start() {
                let error = backoff_error();
                record_failure(health, &error, WorkerState::Backoff).await;
                return Err(error);
            }
            set_state(health, WorkerState::Starting).await;
            match WorkerProcess::spawn(config).await {
                Ok((process, version)) => {
                    *worker = Some(process);
                    record_ready(health, version).await;
                }
                Err(error) => {
                    record_failure(health, &error, WorkerState::Backoff).await;
                    if attempt == 0 && restart_gate.allow_restart() {
                        record_restart(health).await;
                        continue;
                    }
                    restart_gate.block();
                    return Err(error);
                }
            }
        }

        let process = worker.as_mut().expect("worker was initialized");
        let outcome = timeout(
            config.render_timeout,
            process.render(&job.id, &job.request, job.cache_key.clone(), config),
        )
        .await;
        let result = match outcome {
            Ok(result) => result,
            Err(_) => Err(RenderError::new(
                RenderErrorCode::Timeout,
                "Renderer worker exceeded the render deadline",
                true,
            )),
        };

        match result {
            Ok(rendered) => return Ok(rendered),
            Err(error) if must_recycle(&error) => {
                if let Some(mut process) = worker.take() {
                    process.terminate(config.shutdown_timeout).await?;
                }
                record_failure(health, &error, WorkerState::Backoff).await;
                if attempt == 0 && error.retryable && restart_gate.allow_restart() {
                    record_restart(health).await;
                    continue;
                }
                restart_gate.block();
                return Err(error);
            }
            Err(error) => {
                record_failure(health, &error, WorkerState::Ready).await;
                return Err(error);
            }
        }
    }
    Err(backoff_error())
}

fn must_recycle(error: &RenderError) -> bool {
    matches!(
        error.code,
        RenderErrorCode::Protocol
            | RenderErrorCode::WorkerUnavailable
            | RenderErrorCode::Timeout
            | RenderErrorCode::OutputLimitExceeded
            | RenderErrorCode::UnsafeOutput
    )
}

async fn set_state(health: &Arc<Mutex<WorkerHealth>>, state: WorkerState) {
    health.lock().await.state = state;
}

async fn record_ready(health: &Arc<Mutex<WorkerHealth>>, version: String) {
    let mut current = health.lock().await;
    current.state = WorkerState::Ready;
    current.renderer_version = Some(version);
    current.last_error = None;
}

async fn record_failure(
    health: &Arc<Mutex<WorkerHealth>>,
    error: &RenderError,
    state: WorkerState,
) {
    let mut current = health.lock().await;
    current.state = state;
    current.last_error = Some(error.code);
}

async fn record_restart(health: &Arc<Mutex<WorkerHealth>>) {
    let mut current = health.lock().await;
    current.restart_count = current.restart_count.saturating_add(1);
}

fn backoff_error() -> RenderError {
    RenderError::new(
        RenderErrorCode::WorkerUnavailable,
        "Renderer worker restart is temporarily throttled",
        true,
    )
}

fn cancelled_error() -> RenderError {
    RenderError::new(
        RenderErrorCode::Cancelled,
        "Renderer request was cancelled during shutdown",
        false,
    )
}

struct RestartGate {
    interval: std::time::Duration,
    last_restart: Option<Instant>,
    blocked_until: Option<Instant>,
}

impl RestartGate {
    fn new(interval: std::time::Duration) -> Self {
        Self {
            interval,
            last_restart: None,
            blocked_until: None,
        }
    }

    fn can_start(&self) -> bool {
        self.blocked_until
            .is_none_or(|deadline| Instant::now() >= deadline)
    }

    fn allow_restart(&mut self) -> bool {
        let now = Instant::now();
        if self
            .last_restart
            .is_some_and(|previous| now.duration_since(previous) < self.interval)
        {
            return false;
        }
        self.last_restart = Some(now);
        self.blocked_until = None;
        true
    }

    fn block(&mut self) {
        self.blocked_until = Some(Instant::now() + self.interval);
    }
}
