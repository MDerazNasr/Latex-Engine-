//! Verifies the complete versioned bundle staging boundary.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[cfg(unix)]
use std::os::unix::fs::symlink;

use codex_latex_package::BundleManifestV1;
use codex_latex_package::StageOptionsV1;
use codex_latex_package::stage_bundle_v1;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

#[test]
fn stage_builds_a_hashed_source_safe_runtime_layout() {
    let fixture = TestDirectory::new();
    let codex = fixture.write("inputs/codex", b"codex-binary");
    let renderer = fixture.write("inputs/latex-render", b"renderer-binary");
    let worker = fixture.directory("inputs/worker");
    fixture.write("inputs/worker/server.js", b"import 'mathjax';\n");
    fixture.write("inputs/worker/renderer.js", b"export const ok = true;\n");
    fixture.directory("inputs/node_modules");
    let mathjax = fixture.directory("inputs/node_modules/mathjax");
    fixture.write("inputs/node_modules/mathjax/package.json", b"{}\n");
    fixture.write("inputs/node_modules/mathjax/index.js", b"export {};\n");
    let output = fixture.path.join("codex-latex-0.1.0-test");

    stage_bundle_v1(&StageOptionsV1 {
        codex_binary: codex,
        renderer_binary: renderer,
        worker_dist: worker,
        mathjax_module: mathjax,
        output: output.clone(),
        version: "0.1.0".to_string(),
        target: "test-target".to_string(),
    })
    .expect("bundle stages");

    assert_eq!(
        fs::read(output.join("bin/codex-latex")).unwrap(),
        b"codex-binary"
    );
    assert_eq!(
        fs::read(output.join("share/latex-render/mathjax-worker/server.js")).unwrap(),
        b"import 'mathjax';\n"
    );
    let manifest: BundleManifestV1 =
        serde_json::from_slice(&fs::read(output.join("manifest-v1.json")).expect("manifest bytes"))
            .expect("valid manifest");
    assert_eq!(manifest.schema, 1);
    assert_eq!(manifest.daemon_protocol, 1);
    assert_eq!(manifest.node_minimum, 22);
    assert!(
        manifest
            .files
            .windows(2)
            .all(|files| files[0].path < files[1].path)
    );
    assert!(manifest.files.iter().all(|file| file.sha256.len() == 64));
}

#[test]
fn existing_outputs_and_missing_inputs_are_never_replaced() {
    let fixture = TestDirectory::new();
    let output = fixture.directory("existing");
    fixture.write("existing/owned.txt", b"keep me");
    let result = stage_bundle_v1(&StageOptionsV1 {
        codex_binary: fixture.path.join("missing-codex"),
        renderer_binary: fixture.path.join("missing-renderer"),
        worker_dist: fixture.path.join("missing-worker"),
        mathjax_module: fixture.path.join("missing-mathjax"),
        output,
        version: "../bad".to_string(),
        target: "test".to_string(),
    });
    assert!(result.is_err());
    assert_eq!(
        fs::read(fixture.path.join("existing/owned.txt")).unwrap(),
        b"keep me"
    );
}

#[cfg(unix)]
#[test]
fn module_links_inside_the_owned_root_are_materialized() {
    let fixture = TestDirectory::new();
    let codex = fixture.write("inputs/codex", b"codex-binary");
    let renderer = fixture.write("inputs/latex-render", b"renderer-binary");
    let worker = fixture.directory("inputs/worker");
    fixture.write("inputs/worker/server.js", b"import 'mathjax';\n");
    let modules = fixture.directory("inputs/node_modules");
    fixture.directory("inputs/node_modules/.store/mathjax");
    fixture.write("inputs/node_modules/.store/mathjax/package.json", b"{}\n");
    fixture.write(
        "inputs/node_modules/.store/mathjax/index.js",
        b"export {};\n",
    );
    let mathjax = modules.join("mathjax");
    symlink(".store/mathjax", &mathjax).expect("owned module link");
    let output = fixture.path.join("linked-bundle");

    stage_bundle_v1(&StageOptionsV1 {
        codex_binary: codex,
        renderer_binary: renderer,
        worker_dist: worker,
        mathjax_module: mathjax,
        output: output.clone(),
        version: "0.1.0".to_string(),
        target: "test-target".to_string(),
    })
    .expect("linked module stages");

    let installed = output.join("share/latex-render/mathjax-worker/node_modules/mathjax");
    assert!(installed.is_dir());
    assert!(
        !fs::symlink_metadata(installed)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[cfg(unix)]
#[test]
fn module_links_must_resolve_within_the_owned_node_modules_root() {
    let fixture = TestDirectory::new();
    let codex = fixture.write("inputs/codex", b"codex-binary");
    let renderer = fixture.write("inputs/latex-render", b"renderer-binary");
    let worker = fixture.directory("inputs/worker");
    fixture.write("inputs/worker/server.js", b"import 'mathjax';\n");
    let modules = fixture.directory("inputs/node_modules");
    fixture.directory("outside/mathjax");
    fixture.write("outside/mathjax/package.json", b"{}\n");
    let mathjax = modules.join("mathjax");
    symlink(fixture.path.join("outside/mathjax"), &mathjax).expect("escaping module link");
    let output = fixture.path.join("escaped-bundle");

    let result = stage_bundle_v1(&StageOptionsV1 {
        codex_binary: codex,
        renderer_binary: renderer,
        worker_dist: worker,
        mathjax_module: mathjax,
        output: output.clone(),
        version: "0.1.0".to_string(),
        target: "test-target".to_string(),
    });

    assert!(result.is_err());
    assert!(!output.exists());
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "codex-latex-package-stage-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("unique test directory");
        Self { path }
    }

    fn directory(&self, relative: impl AsRef<Path>) -> PathBuf {
        let path = self.path.join(relative);
        fs::create_dir_all(&path).expect("fixture directory");
        path
    }

    fn write(&self, relative: impl AsRef<Path>, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent");
        }
        fs::write(&path, bytes).expect("fixture file");
        path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
