#![doc = "Standalone Codex LaTeX renderer command entry point."]

mod args;
mod error;

#[cfg(test)]
mod args_tests;

use std::ffi::OsString;

use args::ParsedArgs;
use error::{CliError, CliErrorKind};

fn main() {
    let arguments = std::env::args_os().skip(1).collect::<Vec<OsString>>();
    let result = match args::parse_args(arguments) {
        Ok(ParsedArgs::Text(text)) => {
            print!("{text}");
            Ok(())
        }
        Ok(ParsedArgs::Command(_)) => Err(CliError::new(
            CliErrorKind::Internal,
            "Command execution is not available in this build",
        )),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        eprintln!("latex-render: {error}");
        std::process::exit(error.kind().exit_code());
    }
}
