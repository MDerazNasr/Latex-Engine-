//! Copies package inputs without allowing links to escape their owned root.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::error_v1::PackageErrorV1;

pub(crate) fn require_regular_file_v1(path: &Path, label: &str) -> Result<(), PackageErrorV1> {
    let metadata = fs::metadata(path)
        .map_err(|error| PackageErrorV1::io(&format!("{label} metadata failed"), error))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(PackageErrorV1::new(format!(
            "{label} must be a nonempty regular file"
        )));
    }
    Ok(())
}

pub(crate) fn copy_file_v1(
    source: &Path,
    destination: &Path,
    executable: bool,
) -> Result<(), PackageErrorV1> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| PackageErrorV1::io("destination creation failed", error))?;
    }
    fs::copy(source, destination)
        .map_err(|error| PackageErrorV1::io("package file copy failed", error))?;
    set_executable_v1(destination, executable)
}

pub(crate) fn copy_tree_v1(
    source: &Path,
    destination: &Path,
    allowed_root: &Path,
) -> Result<(), PackageErrorV1> {
    let allowed_root = fs::canonicalize(allowed_root)
        .map_err(|error| PackageErrorV1::io("allowed root resolution failed", error))?;
    let mut active = HashSet::new();
    copy_entry_v1(source, destination, &allowed_root, &mut active)
}

fn copy_entry_v1(
    source: &Path,
    destination: &Path,
    allowed_root: &Path,
    active: &mut HashSet<PathBuf>,
) -> Result<(), PackageErrorV1> {
    let resolved = fs::canonicalize(source)
        .map_err(|error| PackageErrorV1::io("package source resolution failed", error))?;
    if !resolved.starts_with(allowed_root) {
        return Err(PackageErrorV1::new(
            "package source link escaped its owned root",
        ));
    }
    let metadata = fs::metadata(&resolved)
        .map_err(|error| PackageErrorV1::io("package source metadata failed", error))?;
    if metadata.is_file() {
        return copy_file_v1(&resolved, destination, false);
    }
    if !metadata.is_dir() {
        return Err(PackageErrorV1::new(
            "package source contains a special file",
        ));
    }
    if !active.insert(resolved.clone()) {
        return Err(PackageErrorV1::new("package source contains a link cycle"));
    }
    fs::create_dir_all(destination)
        .map_err(|error| PackageErrorV1::io("package directory creation failed", error))?;
    let entries = fs::read_dir(&resolved)
        .map_err(|error| PackageErrorV1::io("package directory read failed", error))?;
    for entry in entries {
        let entry = entry.map_err(|error| PackageErrorV1::io("package entry failed", error))?;
        copy_entry_v1(
            &entry.path(),
            &destination.join(entry.file_name()),
            allowed_root,
            active,
        )?;
    }
    active.remove(&resolved);
    Ok(())
}

#[cfg(unix)]
fn set_executable_v1(path: &Path, executable: bool) -> Result<(), PackageErrorV1> {
    use std::os::unix::fs::PermissionsExt;

    let mode = if executable { 0o755 } else { 0o644 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| PackageErrorV1::io("package permissions failed", error))
}

#[cfg(not(unix))]
fn set_executable_v1(_path: &Path, _executable: bool) -> Result<(), PackageErrorV1> {
    Err(PackageErrorV1::new(
        "experimental packaging currently supports macOS and Linux",
    ))
}
