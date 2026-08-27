//! Builds locked source inputs before invoking the verified stager.

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use crate::bundle_v1::StageOptionsV1;
use crate::bundle_v1::stage_bundle_v1;
use crate::error_v1::PackageErrorV1;
use crate::validation_v1::validate_label_v1;

const PINNED_CODEX_COMMIT_V1: &str = "b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46";
const REQUIRED_CODEX_INTEGRATION_COMMIT_V1: &str = "9bf8b63da2";

#[derive(Clone, Debug, Eq, PartialEq)]
/// Selects clean source checkouts and a new package output.
pub struct BuildOptionsV1 {
    /// Points to the renderer source checkout.
    pub engine_root: PathBuf,
    /// Points to the experimental Codex source checkout.
    pub codex_checkout: PathBuf,
    /// Selects a new versioned bundle directory.
    pub output: PathBuf,
    /// Records the renderer bundle version.
    pub version: String,
}

/// Builds locked release artifacts and stages one verified bundle.
pub fn build_bundle_v1(options: &BuildOptionsV1) -> Result<PathBuf, PackageErrorV1> {
    validate_build_inputs_v1(options)?;
    require_clean_checkout_v1(&options.engine_root, "renderer")?;
    require_clean_checkout_v1(&options.codex_checkout, "Codex")?;
    require_codex_ancestor_v1(
        &options.codex_checkout,
        PINNED_CODEX_COMMIT_V1,
        "pinned Codex ancestry check",
    )?;
    require_codex_ancestor_v1(
        &options.codex_checkout,
        REQUIRED_CODEX_INTEGRATION_COMMIT_V1,
        "LaTeX integration ancestry check",
    )?;
    require_node_v1(&options.engine_root)?;

    let worker_root = options.engine_root.join("renderer/mathjax-worker");
    run_status_v1(
        "worker dependency installation",
        command_v1(
            "corepack",
            &worker_root,
            ["pnpm", "install", "--frozen-lockfile"],
        ),
    )?;
    run_status_v1(
        "worker build",
        command_v1("corepack", &worker_root, ["pnpm", "build"]),
    )?;
    run_status_v1(
        "renderer release build",
        command_v1(
            "cargo",
            &options.engine_root,
            [
                "build",
                "--release",
                "--locked",
                "-p",
                "latex-cli",
                "--bin",
                "latex-render",
            ],
        ),
    )?;
    let codex_rust = options.codex_checkout.join("codex-rs");
    run_status_v1(
        "Codex release build",
        command_v1(
            "cargo",
            &codex_rust,
            [
                "build",
                "--release",
                "--locked",
                "-p",
                "codex-cli",
                "--bin",
                "codex",
            ],
        ),
    )?;
    require_clean_checkout_v1(&options.engine_root, "renderer")?;
    require_clean_checkout_v1(&options.codex_checkout, "Codex")?;

    let target = detect_host_target_v1(&options.engine_root)?;
    stage_bundle_v1(&StageOptionsV1 {
        codex_binary: codex_rust.join("target/release/codex"),
        renderer_binary: options.engine_root.join("target/release/latex-render"),
        worker_dist: worker_root.join("dist/src"),
        mathjax_module: worker_root.join("node_modules/mathjax"),
        output: options.output.clone(),
        version: options.version.clone(),
        target,
    })
}

fn validate_build_inputs_v1(options: &BuildOptionsV1) -> Result<(), PackageErrorV1> {
    validate_label_v1(&options.version, "version")?;
    require_file_v1(
        &options.engine_root.join("Cargo.toml"),
        "renderer Cargo workspace",
    )?;
    require_file_v1(
        &options
            .engine_root
            .join("renderer/mathjax-worker/pnpm-lock.yaml"),
        "worker lockfile",
    )?;
    require_file_v1(
        &options.codex_checkout.join("codex-rs/Cargo.toml"),
        "Codex Cargo workspace",
    )?;
    if options.output.exists() {
        return Err(PackageErrorV1::new("bundle output already exists"));
    }
    Ok(())
}

fn require_file_v1(path: &Path, label: &str) -> Result<(), PackageErrorV1> {
    if path.is_file() {
        Ok(())
    } else {
        Err(PackageErrorV1::new(format!("{label} was not found")))
    }
}

fn require_clean_checkout_v1(root: &Path, label: &str) -> Result<(), PackageErrorV1> {
    let output = run_output_v1(
        &format!("{label} status check"),
        command_v1("git", root, ["status", "--porcelain"]),
    )?;
    if output.stdout.is_empty() {
        Ok(())
    } else {
        Err(PackageErrorV1::new(format!(
            "{label} checkout must be clean before packaging"
        )))
    }
}

fn require_codex_ancestor_v1(
    root: &Path,
    revision: &str,
    label: &str,
) -> Result<(), PackageErrorV1> {
    run_status_v1(
        label,
        command_v1(
            "git",
            root,
            ["merge-base", "--is-ancestor", revision, "HEAD"],
        ),
    )
}

fn require_node_v1(current_dir: &Path) -> Result<(), PackageErrorV1> {
    let output = run_output_v1(
        "Node.js version check",
        command_v1("node", current_dir, ["--version"]),
    )?;
    let version = String::from_utf8(output.stdout)
        .map_err(|_| PackageErrorV1::new("Node.js version is not UTF8"))?;
    let major = parse_node_major_v1(&version)?;
    if major >= 22 {
        Ok(())
    } else {
        Err(PackageErrorV1::new("Node.js 22 or newer is required"))
    }
}

fn detect_host_target_v1(current_dir: &Path) -> Result<String, PackageErrorV1> {
    let output = run_output_v1(
        "Rust host target check",
        command_v1("rustc", current_dir, ["-vV"]),
    )?;
    let version = String::from_utf8(output.stdout)
        .map_err(|_| PackageErrorV1::new("Rust version output is not UTF8"))?;
    let target = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| PackageErrorV1::new("Rust host target was not reported"))?
        .to_string();
    validate_label_v1(&target, "target")?;
    Ok(target)
}

fn parse_node_major_v1(version: &str) -> Result<u32, PackageErrorV1> {
    version
        .trim()
        .strip_prefix('v')
        .and_then(|version| version.split('.').next())
        .and_then(|major| major.parse().ok())
        .ok_or_else(|| PackageErrorV1::new("Node.js version could not be parsed"))
}

fn command_v1<I, S>(program: &str, current_dir: &Path, arguments: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(current_dir)
        .env_remove("CARGO_TARGET_DIR");
    command
}

fn run_status_v1(label: &str, mut command: Command) -> Result<(), PackageErrorV1> {
    let status = command
        .status()
        .map_err(|error| PackageErrorV1::io(&format!("{label} could not start"), error))?;
    if status.success() {
        Ok(())
    } else {
        Err(PackageErrorV1::new(format!(
            "{label} failed with status {status}"
        )))
    }
}

fn run_output_v1(label: &str, mut command: Command) -> Result<Output, PackageErrorV1> {
    let output = command
        .output()
        .map_err(|error| PackageErrorV1::io(&format!("{label} could not start"), error))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(PackageErrorV1::new(format!(
            "{label} failed with status {}",
            output.status
        )))
    }
}

#[cfg(test)]
#[path = "build_v1_tests.rs"]
mod tests;
