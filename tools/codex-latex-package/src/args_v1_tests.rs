use std::ffi::OsString;

use super::*;

#[test]
fn stage_command_requires_every_value_once() {
    let command = parse_command_v1(words(&[
        "stage",
        "--codex-binary",
        "codex",
        "--renderer-binary",
        "latex-render",
        "--worker-dist",
        "dist",
        "--mathjax-module",
        "mathjax",
        "--output",
        "bundle",
        "--version",
        "0.1.0",
        "--target",
        "aarch64-apple-darwin",
    ]))
    .expect("valid command");
    let CommandV1::Stage(options) = command;
    assert_eq!(options.output, PathBuf::from("bundle"));
    assert_eq!(options.version, "0.1.0");
}

#[test]
fn missing_duplicate_and_unknown_options_fail_closed() {
    for arguments in [
        words(&["stage", "--output"]),
        words(&["stage", "--output", "one", "--output", "two"]),
        words(&["stage", "--unknown", "value"]),
        words(&["unknown"]),
    ] {
        assert!(parse_command_v1(arguments).is_err());
    }
}

fn words(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
