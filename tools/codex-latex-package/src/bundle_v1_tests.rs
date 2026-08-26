use super::*;

#[test]
fn labels_are_bounded_and_portable() {
    assert!(validate_label_v1("0.1.0", "version").is_ok());
    assert!(validate_label_v1("aarch64-apple-darwin", "target").is_ok());
    let excessive = "x".repeat(65);
    for invalid in ["", "../escape", "spaces are unsafe", excessive.as_str()] {
        assert!(validate_label_v1(invalid, "label").is_err());
    }
}
