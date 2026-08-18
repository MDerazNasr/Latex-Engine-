use std::path::PathBuf;

use latex_terminal::ThemeMode;

use crate::args::{Arguments, BackendSelection, GeometrySpec, ParseResult};

fn values(input: &[&str]) -> Vec<String> {
    input.iter().map(ToString::to_string).collect()
}

#[test]
fn defaults_are_safe_for_automatic_source_fallback() {
    let ParseResult::Run(arguments) = Arguments::parse(values(&[])).expect("defaults should parse")
    else {
        panic!("run arguments should be returned");
    };

    assert_eq!(arguments.backend, BackendSelection::Auto);
    assert_eq!(arguments.geometry, None);
    assert_eq!(arguments.resize_geometry, None);
    assert_eq!(arguments.theme, ThemeMode::Auto);
    assert!(arguments.display_mode);
    assert_eq!(arguments.node, PathBuf::from("node"));
    assert_eq!(arguments.hold_millis, 2_500);
    assert!(!arguments.source.is_empty());
}

#[test]
fn every_acceptance_option_parses_with_exact_geometry() {
    let ParseResult::Run(arguments) = Arguments::parse(values(&[
        "--backend",
        "iterm2",
        "--geometry",
        "120x40@1200x800",
        "--resize-geometry",
        "80x30@800x600",
        "--theme",
        "light",
        "--inline",
        "--worker",
        "worker.js",
        "--node",
        "nodejs",
        "--hold-ms",
        "0",
        "x+y",
    ]))
    .expect("all options should parse") else {
        panic!("run arguments should be returned");
    };

    assert_eq!(arguments.backend, BackendSelection::Iterm2);
    assert_eq!(
        arguments.geometry,
        Some(GeometrySpec {
            columns: 120,
            rows: 40,
            width_px: 1200,
            height_px: 800,
        })
    );
    assert_eq!(
        arguments.resize_geometry,
        Some(GeometrySpec {
            columns: 80,
            rows: 30,
            width_px: 800,
            height_px: 600,
        })
    );
    assert_eq!(arguments.theme, ThemeMode::Light);
    assert!(!arguments.display_mode);
    assert_eq!(arguments.worker, PathBuf::from("worker.js"));
    assert_eq!(arguments.node, PathBuf::from("nodejs"));
    assert_eq!(arguments.hold_millis, 0);
    assert_eq!(arguments.source, "x+y");
}

#[test]
fn double_dash_allows_a_source_that_starts_with_a_hyphen() {
    let ParseResult::Run(arguments) =
        Arguments::parse(values(&["--", "-x"])).expect("source should parse")
    else {
        panic!("run arguments should be returned");
    };
    assert_eq!(arguments.source, "-x");
}

#[test]
fn malformed_null_duplicate_and_incoherent_values_fail() {
    let cases = [
        values(&["--backend"]),
        values(&["--backend", "sixel"]),
        values(&["--geometry", "80x24"]),
        values(&["--geometry", "0x24@800x480"]),
        values(&["--geometry", "70000x24@800x480"]),
        values(&["--resize-geometry", "80x24@800x480"]),
        values(&["--theme", "blue"]),
        values(&["--hold-ms", "60001"]),
        values(&["--worker", ""]),
        values(&["--inline", "--inline"]),
        values(&["--backend", "auto", "--backend", "text"]),
        values(&["x", "y"]),
        values(&[""]),
    ];

    for arguments in cases {
        Arguments::parse(arguments).expect_err("arguments should fail");
    }
}

#[test]
fn help_short_circuits_without_running_the_pipeline() {
    assert!(matches!(
        Arguments::parse(values(&["--help"])).expect("help should parse"),
        ParseResult::Help
    ));
}
