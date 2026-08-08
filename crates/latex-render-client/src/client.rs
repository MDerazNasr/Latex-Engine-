//! Public render client with queue and cache boundaries.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use latex_render_core::{
    CacheKeyContext, CacheStats, MathRenderer, RenderCache, RenderError, RenderErrorCode,
    RenderFuture, RenderRequest, RenderedMath, derive_cache_key,
};
use latex_render_svg::SVG_POLICY_VERSION_LABEL;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::health::WorkerHealth;
use crate::supervisor::{RenderJob, run_supervisor};
use crate::{WORKER_PROTOCOL_VERSION, WorkerClientConfig};

/// A bounded cached client that owns one worker supervisor task.
pub struct WorkerClient {
    config: WorkerClientConfig,
    jobs: Option<mpsc::Sender<RenderJob>>,
    cache: Arc<Mutex<RenderCache>>,
    health: Arc<Mutex<WorkerHealth>>,
    task: Option<JoinHandle<Result<(), RenderError>>>,
    shutdown: Option<oneshot::Sender<()>>,
    next_id: AtomicU64,
}

impl WorkerClient {
    /// Starts an idle supervisor without launching the worker yet.
    pub fn start(config: WorkerClientConfig) -> Result<Self, RenderError> {
        config.validate()?;
        let runtime = tokio::runtime::Handle::try_current().map_err(|_| {
            RenderError::new(
                RenderErrorCode::WorkerUnavailable,
                "Renderer client requires an active Tokio runtime",
                false,
            )
        })?;
        let cache = RenderCache::new(config.cache_limits).ok_or_else(|| {
            RenderError::new(
                RenderErrorCode::InvalidRequest,
                "Cache limits must be greater than zero",
                false,
            )
        })?;
        let (jobs, receiver) = mpsc::channel(config.queue_capacity);
        let (shutdown, shutdown_receiver) = oneshot::channel();
        let health = Arc::new(Mutex::new(WorkerHealth::default()));
        let task = runtime.spawn(run_supervisor(
            config.clone(),
            receiver,
            Arc::clone(&health),
            shutdown_receiver,
        ));
        Ok(Self {
            config,
            jobs: Some(jobs),
            cache: Arc::new(Mutex::new(cache)),
            health,
            task: Some(task),
            shutdown: Some(shutdown),
            next_id: AtomicU64::new(1),
        })
    }

    /// Renders one request through cache and the bounded worker queue.
    pub async fn render_request(
        &self,
        request: RenderRequest,
    ) -> Result<RenderedMath, RenderError> {
        request.validate(&self.config.render_limits)?;
        let cache_key = self.cache_key(&request);
        if let Some(cached) = self.cache.lock().await.get(&cache_key) {
            return Ok((*cached).clone());
        }

        let id = format!("eq-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let (reply, response) = oneshot::channel();
        let job = RenderJob {
            id,
            request,
            cache_key,
            reply,
        };
        let jobs = self.jobs.as_ref().ok_or_else(client_stopped)?;
        match jobs.try_send(job) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                return Err(RenderError::new(
                    RenderErrorCode::QueueFull,
                    "Renderer queue is full",
                    true,
                ));
            }
            Err(mpsc::error::TrySendError::Closed(_)) => return Err(client_stopped()),
        }

        let rendered = response.await.map_err(|_| client_stopped())??;
        self.cache.lock().await.insert(rendered.clone());
        Ok(rendered)
    }

    /// Returns a redacted worker health snapshot.
    pub async fn health(&self) -> WorkerHealth {
        self.health.lock().await.clone()
    }

    /// Returns a cache usage snapshot.
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.lock().await.stats()
    }

    /// Stops the queue and waits for the worker to be reaped.
    pub async fn shutdown(&mut self) -> Result<(), RenderError> {
        self.jobs.take();
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let Some(mut task) = self.task.take() else {
            return Ok(());
        };
        let deadline = self.config.shutdown_timeout.saturating_mul(3);
        match timeout(deadline, &mut task).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(RenderError::new(
                RenderErrorCode::WorkerUnavailable,
                "Renderer supervisor task failed during shutdown",
                false,
            )),
            Err(_) => {
                task.abort();
                let _ = task.await;
                Err(RenderError::new(
                    RenderErrorCode::Timeout,
                    "Renderer supervisor exceeded the shutdown deadline",
                    false,
                ))
            }
        }
    }

    fn cache_key(&self, request: &RenderRequest) -> String {
        derive_cache_key(
            request,
            CacheKeyContext {
                protocol_version: WORKER_PROTOCOL_VERSION,
                renderer_version: &self.config.expected_renderer_version,
                macro_policy_version: &self.config.macro_policy_version,
                sanitizer_version: SVG_POLICY_VERSION_LABEL,
                rasterizer_version: "none",
            },
        )
    }
}

impl MathRenderer for WorkerClient {
    fn render(&self, request: RenderRequest) -> RenderFuture<'_> {
        Box::pin(self.render_request(request))
    }
}

impl Drop for WorkerClient {
    fn drop(&mut self) {
        self.jobs.take();
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

fn client_stopped() -> RenderError {
    RenderError::new(
        RenderErrorCode::WorkerUnavailable,
        "Renderer supervisor is not available",
        true,
    )
}
