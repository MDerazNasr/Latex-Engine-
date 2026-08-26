//! Runs the experimental Codex LaTeX packaging utility.

use std::process::ExitCode;

use codex_latex_package::CommandV1;
use codex_latex_package::parse_command_v1;
use codex_latex_package::stage_bundle_v1;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("codex-latex-package: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match parse_command_v1(std::env::args_os().skip(1))? {
        CommandV1::Stage(options) => {
            let output = stage_bundle_v1(&options)?;
            println!("{}", output.display());
        }
    }
    Ok(())
}
