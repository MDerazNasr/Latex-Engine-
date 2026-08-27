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
    let CommandV1::Stage(options) = command else {
        panic!("expected stage command");
    };
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

#[test]
fn install_and_uninstall_commands_require_explicit_paths() {
    let install = parse_command_v1(words(&[
        "install", "--bundle", "bundle", "--prefix", "prefix",
    ]))
    .expect("install command");
    assert!(matches!(install, CommandV1::Install(_)));

    let uninstall =
        parse_command_v1(words(&["uninstall", "--prefix", "prefix"])).expect("uninstall command");
    assert!(matches!(uninstall, CommandV1::Uninstall(_)));

    assert!(parse_command_v1(words(&["install", "--bundle", "bundle"])).is_err());
    assert!(parse_command_v1(words(&["uninstall"])).is_err());
}

#[test]
fn build_command_requires_both_source_roots_and_output_identity() {
    let build = parse_command_v1(words(&[
        "build",
        "--engine-root",
        "engine",
        "--codex-checkout",
        "codex",
        "--output",
        "bundle",
        "--version",
        "0.1.0",
    ]))
    .expect("build command");
    assert!(matches!(build, CommandV1::Build(_)));
    assert!(
        parse_command_v1(words(&[
            "build",
            "--engine-root",
            "engine",
            "--codex-checkout",
            "codex",
        ]))
        .is_err()
    );
}

fn words(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
