//! Verifies explicit installation, ownership, and rollback behavior.

#![cfg(unix)]

use std::fs;
use std::process::Command;

use codex_latex_package::InstallOptionsV1;
use codex_latex_package::StageOptionsV1;
use codex_latex_package::UninstallOptionsV1;
use codex_latex_package::install_bundle_v1;
use codex_latex_package::stage_bundle_v1;
use codex_latex_package::uninstall_bundle_v1;

use support::TestDirectory;

mod support;

#[test]
fn install_and_uninstall_preserve_the_normal_codex_entrypoint() {
    let fixture = TestDirectory::new();
    let bundle = stage_fixture(&fixture, "bundle");
    let prefix = fixture.directory("prefix");
    fixture.write("prefix/bin/codex", b"normal-codex");

    let installed = install_bundle_v1(&InstallOptionsV1 {
        bundle,
        prefix: prefix.clone(),
    })
    .expect("bundle installs");

    assert_eq!(fs::read(prefix.join("bin/codex")).unwrap(), b"normal-codex");
    assert_eq!(
        fs::canonicalize(&installed.codex_entrypoint).unwrap(),
        fs::canonicalize(installed.installed_root.join("bin/codex-latex")).unwrap()
    );
    assert_eq!(
        fs::canonicalize(&installed.renderer_entrypoint).unwrap(),
        fs::canonicalize(installed.installed_root.join("bin/latex-render")).unwrap()
    );
    assert_eq!(
        fs::canonicalize(&installed.worker_entrypoint).unwrap(),
        fs::canonicalize(
            installed
                .installed_root
                .join("share/latex-render/mathjax-worker")
        )
        .unwrap()
    );

    let removed = uninstall_bundle_v1(&UninstallOptionsV1 {
        prefix: prefix.clone(),
    })
    .expect("bundle uninstalls");
    assert_eq!(removed, installed.installed_root);
    assert!(!removed.exists());
    assert!(!installed.codex_entrypoint.exists());
    assert!(!installed.renderer_entrypoint.exists());
    assert!(fs::symlink_metadata(&installed.worker_entrypoint).is_err());
    assert_eq!(fs::read(prefix.join("bin/codex")).unwrap(), b"normal-codex");
}

#[test]
fn install_refuses_existing_entrypoints_without_mutation() {
    let fixture = TestDirectory::new();
    let bundle = stage_fixture(&fixture, "bundle");
    let prefix = fixture.directory("prefix");
    fixture.write("prefix/bin/codex-latex", b"keep-existing");

    let result = install_bundle_v1(&InstallOptionsV1 {
        bundle,
        prefix: prefix.clone(),
    });

    assert!(result.is_err());
    assert_eq!(
        fs::read(prefix.join("bin/codex-latex")).unwrap(),
        b"keep-existing"
    );
    assert!(!prefix.join("libexec/codex-latex").exists());
}

#[test]
fn install_refuses_an_existing_worker_activation_without_mutation() {
    let fixture = TestDirectory::new();
    let bundle = stage_fixture(&fixture, "bundle");
    let prefix = fixture.directory("prefix");
    fixture.write("prefix/share/latex-render/mathjax-worker", b"keep-existing");

    let result = install_bundle_v1(&InstallOptionsV1 {
        bundle,
        prefix: prefix.clone(),
    });

    assert!(result.is_err());
    assert_eq!(
        fs::read(prefix.join("share/latex-render/mathjax-worker")).unwrap(),
        b"keep-existing"
    );
    assert!(!prefix.join("libexec/codex-latex").exists());
    assert!(!prefix.join("bin/codex-latex").exists());
}

#[test]
fn tampered_or_unowned_files_block_installation_and_rollback() {
    let fixture = TestDirectory::new();
    let bundle = stage_fixture(&fixture, "bundle");
    fs::write(bundle.join("bin/latex-render"), b"tampered").unwrap();
    let prefix = fixture.path.join("tampered-prefix");
    assert!(
        install_bundle_v1(&InstallOptionsV1 {
            bundle,
            prefix: prefix.clone(),
        })
        .is_err()
    );
    assert!(!prefix.exists());

    let clean_bundle = stage_fixture(&fixture, "clean-bundle");
    let installed = install_bundle_v1(&InstallOptionsV1 {
        bundle: clean_bundle,
        prefix: prefix.clone(),
    })
    .expect("clean bundle installs");
    fs::write(installed.installed_root.join("unowned.txt"), b"keep").unwrap();
    assert!(
        uninstall_bundle_v1(&UninstallOptionsV1 {
            prefix: prefix.clone(),
        })
        .is_err()
    );
    assert!(installed.codex_entrypoint.exists());
    assert_eq!(
        fs::read(installed.installed_root.join("unowned.txt")).unwrap(),
        b"keep"
    );
}

#[test]
fn changed_worker_activation_blocks_rollback_without_removing_the_bundle() {
    let fixture = TestDirectory::new();
    let bundle = stage_fixture(&fixture, "bundle");
    let prefix = fixture.directory("prefix");
    let installed = install_bundle_v1(&InstallOptionsV1 {
        bundle,
        prefix: prefix.clone(),
    })
    .expect("bundle installs");
    fs::remove_file(&installed.worker_entrypoint).unwrap();
    let unrelated_worker = fixture.directory("unrelated-worker");
    std::os::unix::fs::symlink(&unrelated_worker, &installed.worker_entrypoint).unwrap();

    let result = uninstall_bundle_v1(&UninstallOptionsV1 { prefix });

    assert!(result.is_err());
    assert!(installed.codex_entrypoint.exists());
    assert!(installed.renderer_entrypoint.exists());
    assert!(installed.installed_root.exists());
    assert_eq!(
        fs::canonicalize(&installed.worker_entrypoint).unwrap(),
        fs::canonicalize(unrelated_worker).unwrap()
    );
}

#[test]
fn command_line_dispatches_stage_install_and_uninstall() {
    let fixture = TestDirectory::new();
    let codex = fixture.write("cli-inputs/codex", b"codex-binary");
    let renderer = fixture.write("cli-inputs/latex-render", b"renderer-binary");
    let worker = fixture.directory("cli-inputs/worker");
    fixture.write("cli-inputs/worker/server.js", b"import 'mathjax';\n");
    let mathjax = fixture.directory("cli-inputs/node_modules/mathjax");
    fixture.write("cli-inputs/node_modules/mathjax/package.json", b"{}\n");
    fixture.write(
        "cli-inputs/node_modules/@mathjax/mathjax-newcm-font/package.json",
        b"{}\n",
    );
    let bundle = fixture.path.join("cli-bundle");
    let prefix = fixture.path.join("cli-prefix");
    let executable = env!("CARGO_BIN_EXE_codex-latex-package");

    let stage = Command::new(executable)
        .args([
            "stage",
            "--codex-binary",
            codex.to_str().unwrap(),
            "--renderer-binary",
            renderer.to_str().unwrap(),
            "--worker-dist",
            worker.to_str().unwrap(),
            "--mathjax-module",
            mathjax.to_str().unwrap(),
            "--output",
            bundle.to_str().unwrap(),
            "--version",
            "0.1.0",
            "--target",
            "test-target",
        ])
        .output()
        .expect("stage process");
    assert!(
        stage.status.success(),
        "{}",
        String::from_utf8_lossy(&stage.stderr)
    );

    let install = Command::new(executable)
        .args([
            "install",
            "--bundle",
            bundle.to_str().unwrap(),
            "--prefix",
            prefix.to_str().unwrap(),
        ])
        .output()
        .expect("install process");
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(prefix.join("bin/codex-latex").exists());
    assert!(prefix.join("share/latex-render/mathjax-worker").exists());

    let uninstall = Command::new(executable)
        .args(["uninstall", "--prefix", prefix.to_str().unwrap()])
        .output()
        .expect("uninstall process");
    assert!(
        uninstall.status.success(),
        "{}",
        String::from_utf8_lossy(&uninstall.stderr)
    );
    assert!(fs::symlink_metadata(prefix.join("bin/codex-latex")).is_err());
    assert!(fs::symlink_metadata(prefix.join("share/latex-render/mathjax-worker")).is_err());
}

fn stage_fixture(fixture: &TestDirectory, name: &str) -> std::path::PathBuf {
    let input = format!("{name}-inputs");
    let codex = fixture.write(format!("{input}/codex"), b"codex-binary");
    let renderer = fixture.write(format!("{input}/latex-render"), b"renderer-binary");
    let worker = fixture.directory(format!("{input}/worker"));
    fixture.write(format!("{input}/worker/server.js"), b"import 'mathjax';\n");
    let mathjax = fixture.directory(format!("{input}/node_modules/mathjax"));
    fixture.write(
        format!("{input}/node_modules/mathjax/package.json"),
        b"{}\n",
    );
    fixture.write(
        format!("{input}/node_modules/mathjax/index.js"),
        b"export {};\n",
    );
    fixture.write(
        format!("{input}/node_modules/@mathjax/mathjax-newcm-font/package.json"),
        b"{}\n",
    );
    let output = fixture.path.join(name);
    stage_bundle_v1(&StageOptionsV1 {
        codex_binary: codex,
        renderer_binary: renderer,
        worker_dist: worker,
        mathjax_module: mathjax,
        output: output.clone(),
        version: "0.1.0".to_string(),
        target: "test-target".to_string(),
    })
    .expect("fixture bundle stages");
    output
}
