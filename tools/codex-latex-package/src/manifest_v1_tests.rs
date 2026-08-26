use super::*;

#[test]
fn unsupported_contracts_hashes_and_sizes_fail_closed() {
    let mut manifest = valid_manifest();
    assert!(validate_manifest_v1(&manifest).is_ok());

    manifest.schema = 2;
    assert!(validate_manifest_v1(&manifest).is_err());
    manifest = valid_manifest();
    manifest.files[0].sha256 = "unsafe".to_string();
    assert!(validate_manifest_v1(&manifest).is_err());
    manifest = valid_manifest();
    manifest.files[0].bytes = MAX_BUNDLE_FILE_BYTES_V1 + 1;
    assert!(validate_manifest_v1(&manifest).is_err());
}

fn valid_manifest() -> BundleManifestV1 {
    BundleManifestV1 {
        schema: 1,
        version: "0.1.0".to_string(),
        target: "test-target".to_string(),
        daemon_protocol: 1,
        node_minimum: 22,
        files: vec![BundleFileV1 {
            path: "bin/codex-latex".to_string(),
            bytes: 1,
            sha256: "a".repeat(64),
            executable: true,
        }],
    }
}
