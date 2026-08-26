//! Stages one immutable developer bundle and publishes it atomically.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_json::json;

use crate::error_v1::PackageErrorV1;
use crate::filesystem_v1::copy_file_v1;
use crate::filesystem_v1::copy_tree_v1;
use crate::filesystem_v1::require_regular_file_v1;
use crate::manifest_v1::BundleManifestV1;
use crate::manifest_v1::build_manifest_v1;
use crate::manifest_v1::write_manifest_v1;

const WORKER_DESTINATION_V1: &str = "share/latex-render/mathjax-worker";

#[derive(Clone, Debug, Eq, PartialEq)]
/// Identifies every input required to stage one bundle.
pub struct StageOptionsV1 {
    /// Points to the tested experimental Codex executable.
    pub codex_binary: PathBuf,
    /// Points to the tested renderer daemon executable.
    pub renderer_binary: PathBuf,
    /// Points to the compiled worker JavaScript directory.
    pub worker_dist: PathBuf,
    /// Points to the installed locked MathJax module.
    pub mathjax_module: PathBuf,
    /// Selects a new versioned bundle directory.
    pub output: PathBuf,
    /// Records the renderer bundle version.
    pub version: String,
    /// Records the Rust host target used for both binaries.
    pub target: String,
}

/// Stages a complete bundle and publishes its manifest last.
pub fn stage_bundle_v1(options: &StageOptionsV1) -> Result<PathBuf, PackageErrorV1> {
    validate_options_v1(options)?;
    let parent = options
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| PackageErrorV1::io("bundle parent creation failed", error))?;
    fs::create_dir(&options.output)
        .map_err(|error| PackageErrorV1::io("bundle reservation failed", error))?;
    let guard = IncompleteBundleV1::new(options.output.clone());

    copy_file_v1(
        &options.codex_binary,
        &options.output.join("bin/codex-latex"),
        true,
    )?;
    copy_file_v1(
        &options.renderer_binary,
        &options.output.join("bin/latex-render"),
        true,
    )?;
    let worker = options.output.join(WORKER_DESTINATION_V1);
    copy_tree_v1(&options.worker_dist, &worker, &options.worker_dist)?;
    let module_root = options
        .mathjax_module
        .parent()
        .ok_or_else(|| PackageErrorV1::new("MathJax module has no owned root"))?;
    copy_tree_v1(
        &options.mathjax_module,
        &worker.join("node_modules/mathjax"),
        module_root,
    )?;
    let package = serde_json::to_vec_pretty(&json!({
        "name": "@codex-latex/mathjax-worker-runtime",
        "private": true,
        "type": "module",
        "engines": { "node": ">=22" }
    }))
    .map_err(|error| PackageErrorV1::new(format!("worker package encoding failed: {error}")))?;
    let mut package_with_newline = package;
    package_with_newline.push(b'\n');
    fs::write(worker.join("package.json"), package_with_newline)
        .map_err(|error| PackageErrorV1::io("worker package write failed", error))?;

    let manifest = build_manifest_v1(&options.output, &options.version, &options.target)?;
    validate_manifest_shape_v1(&manifest)?;
    write_manifest_v1(&options.output, &manifest)?;
    guard.keep();
    Ok(options.output.clone())
}

fn validate_options_v1(options: &StageOptionsV1) -> Result<(), PackageErrorV1> {
    if options.output.exists() {
        return Err(PackageErrorV1::new("bundle output already exists"));
    }
    require_regular_file_v1(&options.codex_binary, "Codex binary")?;
    require_regular_file_v1(&options.renderer_binary, "renderer binary")?;
    require_regular_file_v1(&options.worker_dist.join("server.js"), "worker server")?;
    require_regular_file_v1(
        &options.mathjax_module.join("package.json"),
        "MathJax module",
    )?;
    validate_label_v1(&options.version, "version")?;
    validate_label_v1(&options.target, "target")
}

fn validate_label_v1(value: &str, label: &str) -> Result<(), PackageErrorV1> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(PackageErrorV1::new(format!("bundle {label} is invalid")));
    }
    Ok(())
}

fn validate_manifest_shape_v1(manifest: &BundleManifestV1) -> Result<(), PackageErrorV1> {
    let required = [
        "bin/codex-latex",
        "bin/latex-render",
        "share/latex-render/mathjax-worker/server.js",
        "share/latex-render/mathjax-worker/node_modules/mathjax/package.json",
    ];
    if required.iter().all(|required| {
        manifest
            .files
            .iter()
            .any(|file| file.path == *required && file.bytes > 0)
    }) {
        Ok(())
    } else {
        Err(PackageErrorV1::new(
            "bundle is missing a required runtime file",
        ))
    }
}

struct IncompleteBundleV1 {
    path: PathBuf,
    keep: std::cell::Cell<bool>,
}

impl IncompleteBundleV1 {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            keep: std::cell::Cell::new(false),
        }
    }

    fn keep(&self) {
        self.keep.set(true);
    }
}

impl Drop for IncompleteBundleV1 {
    fn drop(&mut self) {
        if !self.keep.get() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
#[path = "bundle_v1_tests.rs"]
mod tests;
