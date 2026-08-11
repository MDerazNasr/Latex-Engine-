use std::ffi::OsString;
use std::path::PathBuf;

use latex_render_core::Rgba;

use crate::args::{CliCommand, OutputFormat, ParsedArgs, parse_args};
use crate::error::CliErrorKind;

fn values(input: &[&str]) -> Vec<OsString> {
    input.iter().map(OsString::from).collect()
}

#[test]
fn root_help_and_version_are_successful_text() {
    let help = parse_args(values(&[])).expect("empty arguments should show help");
    let version = parse_args(values(&["--version"])).expect("version should parse");

    assert!(matches!(help, ParsedArgs::Text(text) if text.contains("Usage:")));
    assert!(matches!(version, ParsedArgs::Text(text) if text == "latex-render 0.1.0\n"));
}

#[test]
fn render_defaults_are_deterministic() {
    let parsed = parse_args(values(&["render", "x^2"])).expect("render should parse");
    let ParsedArgs::Command(CliCommand::Render(options)) = parsed else {
        panic!("render command should be returned");
    };

    assert_eq!(options.source.as_deref(), Some("x^2"));
    assert!(!options.display_mode);
    assert_eq!(options.format, OutputFormat::Svg);
    assert_eq!(options.output, None);
    assert!(!options.force);
    assert_eq!(options.foreground, Rgba::opaque(230, 237, 243));
    assert_eq!(options.background, None);
    assert_eq!(options.scale, 2.0);
    assert_eq!(options.max_width_px, 1200);
    assert_eq!(options.worker.worker, None);
    assert_eq!(options.worker.node, PathBuf::from("node"));
}

#[test]
fn render_accepts_every_declared_option() {
    let parsed = parse_args(values(&[
        "render",
        "--display",
        "--format",
        "png",
        "--output",
        "equation.png",
        "--force",
        "--foreground",
        "#A0b1C2",
        "--background",
        "#010203",
        "--scale",
        "1.5",
        "--max-width",
        "800",
        "--worker",
        "worker.js",
        "--node",
        "nodejs",
        "x+y",
    ]))
    .expect("all render options should parse");
    let ParsedArgs::Command(CliCommand::Render(options)) = parsed else {
        panic!("render command should be returned");
    };

    assert!(options.display_mode);
    assert_eq!(options.format, OutputFormat::Png);
    assert_eq!(options.output, Some(PathBuf::from("equation.png")));
    assert!(options.force);
    assert_eq!(options.foreground, Rgba::opaque(160, 177, 194));
    assert_eq!(options.background, Some(Rgba::opaque(1, 2, 3)));
    assert_eq!(options.scale, 1.5);
    assert_eq!(options.max_width_px, 800);
    assert_eq!(options.worker.worker, Some(PathBuf::from("worker.js")));
    assert_eq!(options.worker.node, PathBuf::from("nodejs"));
}

#[test]
fn missing_source_is_reserved_for_redirected_stdin() {
    let parsed = parse_args(values(&["render"])).expect("stdin render should parse");
    let ParsedArgs::Command(CliCommand::Render(options)) = parsed else {
        panic!("render command should be returned");
    };
    assert_eq!(options.source, None);
}

#[test]
fn double_dash_allows_source_that_starts_with_a_hyphen() {
    let parsed = parse_args(values(&["render", "--", "-x"])).expect("source should parse");
    let ParsedArgs::Command(CliCommand::Render(options)) = parsed else {
        panic!("render command should be returned");
    };
    assert_eq!(options.source.as_deref(), Some("-x"));
}

#[test]
fn check_accepts_only_worker_selection() {
    let parsed = parse_args(values(&[
        "check",
        "--worker",
        "worker.js",
        "--node",
        "nodejs",
    ]))
    .expect("check should parse");
    let ParsedArgs::Command(CliCommand::Check(options)) = parsed else {
        panic!("check command should be returned");
    };
    assert_eq!(options.worker, Some(PathBuf::from("worker.js")));
    assert_eq!(options.node, PathBuf::from("nodejs"));
}

#[test]
fn malformed_null_and_duplicate_values_are_usage_errors() {
    let cases = [
        values(&["unknown"]),
        values(&["render", "x", "y"]),
        values(&["render", "--format"]),
        values(&["render", "--format", "pdf"]),
        values(&["render", "--foreground", "red"]),
        values(&["render", "--background", "#fffffg"]),
        values(&["render", "--scale", "NaN"]),
        values(&["render", "--scale", "5"]),
        values(&["render", "--max-width", "0"]),
        values(&["render", "--force", "x"]),
        values(&["render", "--display", "--display", "x"]),
        values(&["check", "source"]),
        values(&["check", "--worker", "a", "--worker", "b"]),
        values(&["--version", "extra"]),
    ];

    for arguments in cases {
        let error = parse_args(arguments).expect_err("arguments should fail");
        assert_eq!(error.kind(), CliErrorKind::Usage);
        assert!(!error.to_string().contains("x^2"));
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_source_is_a_usage_error() {
    use std::os::unix::ffi::OsStringExt;

    let error = parse_args(vec![
        OsString::from("render"),
        OsString::from_vec(vec![0xff]),
    ])
    .expect_err("non UTF 8 source should fail");

    assert_eq!(error.kind(), CliErrorKind::Usage);
}
