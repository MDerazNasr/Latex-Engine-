//! Defines the deterministic bundle ownership manifest.

use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

use crate::error_v1::PackageErrorV1;

pub(crate) const MANIFEST_NAME_V1: &str = "manifest-v1.json";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
pub struct BundleFileV1 {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub executable: bool,
}

pub(crate) fn build_manifest_v1(
    root: &Path,
    version: &str,
    target: &str,
) -> Result<BundleManifestV1, PackageErrorV1> {
    let mut files = Vec::new();
    collect_files_v1(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(BundleManifestV1 {
        schema: 1,
        version: version.to_string(),
        target: target.to_string(),
        daemon_protocol: 1,
        node_minimum: 22,
        files,
    })
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

fn digest_file_v1(path: &Path) -> Result<String, PackageErrorV1> {
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
