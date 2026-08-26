use super::*;

#[test]
fn labels_and_relative_paths_fail_closed() {
    assert!(validate_label_v1("0.1.0", "version").is_ok());
    assert!(validate_relative_path_v1("bin/codex-latex").is_ok());
    let excessive = "x".repeat(65);
    for invalid in ["", "../escape", "spaces are unsafe", excessive.as_str()] {
        assert!(validate_label_v1(invalid, "label").is_err());
    }
    for invalid in ["", "/absolute", "../escape", "a/../b", "a\\b"] {
        assert!(validate_relative_path_v1(invalid).is_err());
    }
}
