//! Deterministic local MathJax worker discovery.

use std::path::{Path, PathBuf};

use crate::args::WorkerOptions;
use crate::error::{CliError, CliErrorKind};

const WORKER_ENVIRONMENT: &str = "LATEX_RENDER_WORKER";
const DEVELOPMENT_WORKER: &str = "renderer/mathjax-worker/dist/src/server.js";

pub(crate) fn resolve_worker(options: &WorkerOptions) -> Result<PathBuf, CliError> {
    if let Some(explicit) = &options.worker {
        return require_worker(explicit);
    }
    if let Some(environment) = std::env::var_os(WORKER_ENVIRONMENT) {
        if environment.is_empty() {
            return Err(worker_error(
                "LATEX_RENDER_WORKER must not be empty when it is set",
            ));
        }
        return require_worker(Path::new(&environment));
    }

    for candidate in default_candidates() {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(worker_error(
        "MathJax worker was not found; use --worker or LATEX_RENDER_WORKER",
    ))
}

fn require_worker(path: &Path) -> Result<PathBuf, CliError> {
    if path.is_file() {
        Ok(path.to_owned())
    } else {
        Err(worker_error("Configured MathJax worker is not a file"))
    }
}

fn default_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        candidates.push(directory.join("mathjax-worker/server.js"));
        if let Some(prefix) = directory.parent() {
            candidates.push(prefix.join("share/latex-render/mathjax-worker/server.js"));
        }
        for ancestor in directory.ancestors() {
            candidates.push(ancestor.join(DEVELOPMENT_WORKER));
        }
    }
    if let Ok(directory) = std::env::current_dir() {
        candidates.push(directory.join(DEVELOPMENT_WORKER));
    }
    candidates
}

fn worker_error(message: impl Into<String>) -> CliError {
    CliError::new(CliErrorKind::Worker, message)
}
