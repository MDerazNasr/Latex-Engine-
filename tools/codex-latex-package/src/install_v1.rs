//! Installs and removes only manifest owned experimental bundles.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::error_v1::PackageErrorV1;
use crate::filesystem_v1::copy_file_v1;
use crate::manifest_v1::BundleManifestV1;
use crate::manifest_v1::MANIFEST_NAME_V1;
use crate::manifest_v1::read_manifest_v1;
use crate::manifest_v1::verify_manifest_files_v1;

const INSTALL_BASE_V1: &str = "libexec/codex-latex";

#[derive(Clone, Debug, Eq, PartialEq)]
/// Selects a verified bundle and an explicit installation prefix.
pub struct InstallOptionsV1 {
    /// Points to a complete staged bundle.
    pub bundle: PathBuf,
    /// Selects the installation prefix.
    pub prefix: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Reports the paths created by a successful installation.
pub struct InstallResultV1 {
    /// Identifies the isolated installed bundle root.
    pub installed_root: PathBuf,
    /// Identifies the experimental Codex entry point.
    pub codex_entrypoint: PathBuf,
    /// Identifies the renderer entry point.
    pub renderer_entrypoint: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Selects an explicit prefix for manifest owned rollback.
pub struct UninstallOptionsV1 {
    /// Selects the installation prefix.
    pub prefix: PathBuf,
}

/// Installs one verified bundle without replacing existing paths.
pub fn install_bundle_v1(options: &InstallOptionsV1) -> Result<InstallResultV1, PackageErrorV1> {
    validate_prefix_v1(&options.prefix)?;
    let manifest = read_manifest_v1(&options.bundle)?;
    verify_manifest_files_v1(&options.bundle, &manifest)?;
    let install_root = install_root_v1(&options.prefix, &manifest);
    let codex_entrypoint = options.prefix.join("bin/codex-latex");
    let renderer_entrypoint = options.prefix.join("bin/latex-render");
    require_absent_v1(&install_root, "installed bundle")?;
    require_absent_v1(&codex_entrypoint, "Codex LaTeX entry point")?;
    require_absent_v1(&renderer_entrypoint, "renderer entry point")?;

    fs::create_dir_all(options.prefix.join(INSTALL_BASE_V1))
        .map_err(|error| PackageErrorV1::io("installation base creation failed", error))?;
    fs::create_dir_all(options.prefix.join("bin"))
        .map_err(|error| PackageErrorV1::io("entry point directory creation failed", error))?;
    fs::create_dir(&install_root)
        .map_err(|error| PackageErrorV1::io("installed bundle reservation failed", error))?;
    let install_guard = IncompleteInstallV1::new(install_root.clone());
    copy_manifest_files_v1(&options.bundle, &install_root, &manifest)?;
    copy_file_v1(
        &options.bundle.join(MANIFEST_NAME_V1),
        &install_root.join(MANIFEST_NAME_V1),
        false,
    )?;
    let installed_manifest = read_manifest_v1(&install_root)?;
    verify_manifest_files_v1(&install_root, &installed_manifest)?;

    let key = bundle_key_v1(&manifest);
    let codex_target = PathBuf::from(format!("../{INSTALL_BASE_V1}/{key}/bin/codex-latex"));
    let renderer_target = PathBuf::from(format!("../{INSTALL_BASE_V1}/{key}/bin/latex-render"));
    create_symlink_v1(&codex_target, &codex_entrypoint)?;
    let codex_guard = EntryPointGuardV1::new(codex_entrypoint.clone(), codex_target);
    create_symlink_v1(&renderer_target, &renderer_entrypoint)?;
    let renderer_guard = EntryPointGuardV1::new(renderer_entrypoint.clone(), renderer_target);
    renderer_guard.keep();
    codex_guard.keep();
    install_guard.keep();

    Ok(InstallResultV1 {
        installed_root: install_root,
        codex_entrypoint,
        renderer_entrypoint,
    })
}

/// Removes only entry points and files owned by the active bundle manifest.
pub fn uninstall_bundle_v1(options: &UninstallOptionsV1) -> Result<PathBuf, PackageErrorV1> {
    validate_prefix_v1(&options.prefix)?;
    let codex_entrypoint = options.prefix.join("bin/codex-latex");
    let renderer_entrypoint = options.prefix.join("bin/latex-render");
    let codex_target = resolve_owned_entrypoint_v1(&codex_entrypoint)?;
    let renderer_target = resolve_owned_entrypoint_v1(&renderer_entrypoint)?;
    let install_root = codex_target
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| PackageErrorV1::new("Codex LaTeX entry point target is invalid"))?
        .to_path_buf();
    if renderer_target != install_root.join("bin/latex-render")
        || codex_target != install_root.join("bin/codex-latex")
    {
        return Err(PackageErrorV1::new(
            "entry points do not identify one installed bundle",
        ));
    }
    let base = fs::canonicalize(options.prefix.join(INSTALL_BASE_V1))
        .map_err(|error| PackageErrorV1::io("installation base resolution failed", error))?;
    if install_root.parent() != Some(base.as_path()) {
        return Err(PackageErrorV1::new(
            "entry point target is outside the installation base",
        ));
    }
    let manifest = read_manifest_v1(&install_root)?;
    verify_manifest_files_v1(&install_root, &manifest)?;
    let reported_root = install_root_v1(&options.prefix, &manifest);
    verify_entrypoint_v1(&codex_entrypoint, &codex_target)?;
    verify_entrypoint_v1(&renderer_entrypoint, &renderer_target)?;

    fs::remove_file(&renderer_entrypoint)
        .map_err(|error| PackageErrorV1::io("renderer entry point removal failed", error))?;
    fs::remove_file(&codex_entrypoint)
        .map_err(|error| PackageErrorV1::io("Codex LaTeX entry point removal failed", error))?;
    remove_manifest_files_v1(&install_root, &manifest)?;
    remove_empty_directory_v1(&options.prefix.join(INSTALL_BASE_V1))?;
    remove_empty_directory_v1(&options.prefix.join("libexec"))?;
    remove_empty_directory_v1(&options.prefix.join("bin"))?;
    Ok(reported_root)
}

fn copy_manifest_files_v1(
    source: &Path,
    destination: &Path,
    manifest: &BundleManifestV1,
) -> Result<(), PackageErrorV1> {
    for file in &manifest.files {
        copy_file_v1(
            &source.join(&file.path),
            &destination.join(&file.path),
            file.executable,
        )?;
    }
    Ok(())
}

fn remove_manifest_files_v1(
    root: &Path,
    manifest: &BundleManifestV1,
) -> Result<(), PackageErrorV1> {
    let mut directories = BTreeSet::new();
    for file in &manifest.files {
        let path = root.join(&file.path);
        fs::remove_file(&path)
            .map_err(|error| PackageErrorV1::io("owned file removal failed", error))?;
        let mut parent = path.parent();
        while let Some(directory) = parent {
            if directory == root {
                break;
            }
            directories.insert(directory.to_path_buf());
            parent = directory.parent();
        }
    }
    fs::remove_file(root.join(MANIFEST_NAME_V1))
        .map_err(|error| PackageErrorV1::io("installed manifest removal failed", error))?;
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        fs::remove_dir(&directory)
            .map_err(|error| PackageErrorV1::io("owned directory removal failed", error))?;
    }
    fs::remove_dir(root)
        .map_err(|error| PackageErrorV1::io("installed bundle removal failed", error))
}

fn install_root_v1(prefix: &Path, manifest: &BundleManifestV1) -> PathBuf {
    prefix.join(INSTALL_BASE_V1).join(bundle_key_v1(manifest))
}

fn bundle_key_v1(manifest: &BundleManifestV1) -> String {
    format!("{}-{}", manifest.version, manifest.target)
}

fn validate_prefix_v1(prefix: &Path) -> Result<(), PackageErrorV1> {
    if prefix.as_os_str().is_empty() {
        Err(PackageErrorV1::new("installation prefix is empty"))
    } else {
        Ok(())
    }
}

fn require_absent_v1(path: &Path, label: &str) -> Result<(), PackageErrorV1> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(PackageErrorV1::new(format!("{label} already exists"))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PackageErrorV1::io(&format!("{label} lookup failed"), error)),
    }
}

fn resolve_owned_entrypoint_v1(path: &Path) -> Result<PathBuf, PackageErrorV1> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| PackageErrorV1::io("entry point metadata failed", error))?;
    if !metadata.file_type().is_symlink() {
        return Err(PackageErrorV1::new(
            "entry point is not an owned symbolic link",
        ));
    }
    fs::canonicalize(path)
        .map_err(|error| PackageErrorV1::io("entry point target resolution failed", error))
}

fn verify_entrypoint_v1(path: &Path, expected: &Path) -> Result<(), PackageErrorV1> {
    if resolve_owned_entrypoint_v1(path)? == expected {
        Ok(())
    } else {
        Err(PackageErrorV1::new(
            "entry point changed during uninstallation",
        ))
    }
}

#[cfg(unix)]
fn create_symlink_v1(target: &Path, link: &Path) -> Result<(), PackageErrorV1> {
    std::os::unix::fs::symlink(target, link)
        .map_err(|error| PackageErrorV1::io("entry point creation failed", error))
}

#[cfg(not(unix))]
fn create_symlink_v1(_target: &Path, _link: &Path) -> Result<(), PackageErrorV1> {
    Err(PackageErrorV1::new(
        "experimental installation currently supports macOS and Linux",
    ))
}

fn remove_empty_directory_v1(path: &Path) -> Result<(), PackageErrorV1> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(PackageErrorV1::io(
            "empty installation directory cleanup failed",
            error,
        )),
    }
}

struct IncompleteInstallV1 {
    root: PathBuf,
    keep: std::cell::Cell<bool>,
}

impl IncompleteInstallV1 {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            keep: std::cell::Cell::new(false),
        }
    }

    fn keep(&self) {
        self.keep.set(true);
    }
}

impl Drop for IncompleteInstallV1 {
    fn drop(&mut self) {
        if !self.keep.get() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

struct EntryPointGuardV1 {
    path: PathBuf,
    target: PathBuf,
    keep: std::cell::Cell<bool>,
}

impl EntryPointGuardV1 {
    fn new(path: PathBuf, target: PathBuf) -> Self {
        Self {
            path,
            target,
            keep: std::cell::Cell::new(false),
        }
    }

    fn keep(&self) {
        self.keep.set(true);
    }
}

impl Drop for EntryPointGuardV1 {
    fn drop(&mut self) {
        if !self.keep.get() && fs::read_link(&self.path).is_ok_and(|target| target == self.target) {
            let _ = fs::remove_file(&self.path);
        }
    }
}
