use super::*;

#[test]
fn node_major_requires_a_canonical_version() {
    assert_eq!(parse_node_major_v1("v22.18.0\n").unwrap(), 22);
    assert_eq!(parse_node_major_v1("v100.0.0").unwrap(), 100);
    for invalid in ["", "22.0.0", "version 22", "vnext"] {
        assert!(parse_node_major_v1(invalid).is_err());
    }
}

#[test]
fn build_inputs_require_cleanly_named_locked_source_roots() {
    let missing = BuildOptionsV1 {
        engine_root: PathBuf::from("missing-engine"),
        codex_checkout: PathBuf::from("missing-codex"),
        output: PathBuf::from("bundle"),
        version: "../unsafe".to_string(),
    };
    assert!(validate_build_inputs_v1(&missing).is_err());
}
