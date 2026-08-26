//! Defines the deterministic bundle ownership manifest.

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

use crate::error_v1::PackageErrorV1;
use crate::validation_v1::validate_label_v1;
use crate::validation_v1::validate_relative_path_v1;

pub(crate) const MANIFEST_NAME_V1: &str = "manifest-v1.json";
const MAX_MANIFEST_BYTES_V1: u64 = 8 * 1024 * 1024;
const MAX_MANIFEST_FILES_V1: usize = 20_000;
const MAX_BUNDLE_FILE_BYTES_V1: u64 = 1024 * 1024 * 1024;
const MAX_BUNDLE_BYTES_V1: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Records the owned files and runtime contract of one bundle.
pub struct BundleManifestV1 {
    /// Identifies the manifest schema.
    pub schema: u32,
    /// Identifies the renderer bundle version.
    pub version: String,
    /// Identifies the build target.
    pub target: String,
    /// Identifies the renderer daemon protocol.
    pub daemon_protocol: u32,
    /// Identifies the minimum supported Node.js major version.
    pub node_minimum: u32,
    /// Lists every owned runtime file in path order.
    pub files: Vec<BundleFileV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Records one regular file owned by the bundle.
pub struct BundleFileV1 {
    /// Stores a portable relative path.
    pub path: String,
    /// Stores the exact file size.
    pub bytes: u64,
    /// Stores the lowercase SHA256 digest.
    pub sha256: String,
    /// Records whether installation must retain executable permissions.
    pub executable: bool,
}

pub(crate) fn read_manifest_v1(root: &Path) -> Result<BundleManifestV1, PackageErrorV1> {
    let path = root.join(MANIFEST_NAME_V1);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| PackageErrorV1::io("manifest metadata failed", error))?;
    if !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_MANIFEST_BYTES_V1
    {
        return Err(PackageErrorV1::new("manifest file is invalid or oversized"));
    }
    let bytes =
        fs::read(path).map_err(|error| PackageErrorV1::io("manifest read failed", error))?;
    let manifest: BundleManifestV1 = serde_json::from_slice(&bytes)
        .map_err(|error| PackageErrorV1::new(format!("manifest decoding failed: {error}")))?;
    validate_manifest_v1(&manifest)?;
    Ok(manifest)
}

pub(crate) fn verify_manifest_files_v1(
    root: &Path,
    manifest: &BundleManifestV1,
) -> Result<(), PackageErrorV1> {
    let mut expected = BTreeSet::new();
    for file in &manifest.files {
        validate_relative_path_v1(&file.path)?;
        if !expected.insert(file.path.as_str()) {
            return Err(PackageErrorV1::new("manifest contains a duplicate path"));
        }
        let path = root.join(&file.path);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| PackageErrorV1::io("owned file metadata failed", error))?;
        if !metadata.file_type().is_file()
            || metadata.len() != file.bytes
            || digest_file_v1(&path)? != file.sha256
        {
            return Err(PackageErrorV1::new(
                "owned file does not match its manifest",
            ));
        }
    }
    let mut actual = Vec::new();
    collect_paths_v1(root, root, &mut actual)?;
    actual.retain(|path| path != MANIFEST_NAME_V1);
    actual.sort();
    if actual.len() != expected.len()
        || actual
            .iter()
            .map(String::as_str)
            .ne(expected.iter().copied())
    {
        return Err(PackageErrorV1::new(
            "bundle contains an unowned or missing file",
        ));
    }
    Ok(())
}

pub(crate) fn build_manifest_v1(
    root: &Path,
    version: &str,
    target: &str,
) -> Result<BundleManifestV1, PackageErrorV1> {
    let mut files = Vec::new();
    collect_files_v1(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = BundleManifestV1 {
        schema: 1,
        version: version.to_string(),
        target: target.to_string(),
        daemon_protocol: 1,
        node_minimum: 22,
        files,
    };
    validate_manifest_v1(&manifest)?;
    Ok(manifest)
}

pub(crate) fn write_manifest_v1(
    root: &Path,
    manifest: &BundleManifestV1,
) -> Result<(), PackageErrorV1> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| PackageErrorV1::new(format!("manifest encoding failed: {error}")))?;
    let mut bytes_with_newline = bytes;
    bytes_with_newline.push(b'\n');
    let temporary = root.join(".manifest-v1.json.incomplete");
    fs::write(&temporary, bytes_with_newline)
        .map_err(|error| PackageErrorV1::io("manifest write failed", error))?;
    fs::rename(temporary, root.join(MANIFEST_NAME_V1))
        .map_err(|error| PackageErrorV1::io("manifest publication failed", error))
}

fn collect_files_v1(
    root: &Path,
    directory: &Path,
    files: &mut Vec<BundleFileV1>,
) -> Result<(), PackageErrorV1> {
    let entries = fs::read_dir(directory)
        .map_err(|error| PackageErrorV1::io("bundle traversal failed", error))?;
    for entry in entries {
        let entry = entry.map_err(|error| PackageErrorV1::io("bundle entry failed", error))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| PackageErrorV1::io("bundle metadata failed", error))?;
        if metadata.is_dir() {
            collect_files_v1(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| PackageErrorV1::new("bundle path escaped its root"))?;
            files.push(BundleFileV1 {
                path: portable_path_v1(relative)?,
                bytes: metadata.len(),
                sha256: digest_file_v1(&path)?,
                executable: relative.starts_with("bin"),
            });
        } else {
            return Err(PackageErrorV1::new("bundle contains a special file"));
        }
    }
    Ok(())
}

fn portable_path_v1(path: &Path) -> Result<String, PackageErrorV1> {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| PackageErrorV1::new("bundle path is not Unicode"))?;
    Ok(components.join("/"))
}

pub(crate) fn digest_file_v1(path: &Path) -> Result<String, PackageErrorV1> {
    let mut file = fs::File::open(path)
        .map_err(|error| PackageErrorV1::io("bundle file open failed", error))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| PackageErrorV1::io("bundle file read failed", error))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn validate_manifest_v1(manifest: &BundleManifestV1) -> Result<(), PackageErrorV1> {
    let aggregate_bytes = manifest
        .files
        .iter()
        .try_fold(0_u64, |total, file| total.checked_add(file.bytes));
    if manifest.schema != 1
        || manifest.daemon_protocol != 1
        || manifest.node_minimum != 22
        || manifest.files.is_empty()
        || manifest.files.len() > MAX_MANIFEST_FILES_V1
        || aggregate_bytes.is_none_or(|bytes| bytes > MAX_BUNDLE_BYTES_V1)
        || manifest.files.iter().any(|file| {
            file.bytes > MAX_BUNDLE_FILE_BYTES_V1
                || file.sha256.len() != 64
                || !file
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        return Err(PackageErrorV1::new("manifest contract is unsupported"));
    }
    validate_label_v1(&manifest.version, "version")?;
    validate_label_v1(&manifest.target, "target")?;
    if manifest
        .files
        .windows(2)
        .all(|files| files[0].path < files[1].path)
    {
        Ok(())
    } else {
        Err(PackageErrorV1::new(
            "manifest paths are not strictly sorted",
        ))
    }
}

#[cfg(test)]
#[path = "manifest_v1_tests.rs"]
mod tests;

fn collect_paths_v1(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<String>,
) -> Result<(), PackageErrorV1> {
    for entry in fs::read_dir(directory)
        .map_err(|error| PackageErrorV1::io("bundle verification traversal failed", error))?
    {
        let entry =
            entry.map_err(|error| PackageErrorV1::io("bundle verification entry failed", error))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| PackageErrorV1::io("bundle verification metadata failed", error))?;
        if metadata.file_type().is_dir() {
            collect_paths_v1(root, &path, paths)?;
        } else if metadata.file_type().is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| PackageErrorV1::new("bundle verification path escaped its root"))?;
            paths.push(portable_path_v1(relative)?);
        } else {
            return Err(PackageErrorV1::new(
                "bundle contains a link or special file",
            ));
        }
    }
    Ok(())
}
