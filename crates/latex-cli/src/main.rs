#![doc = "Standalone Codex LaTeX renderer command entry point."]

mod app;
mod args;
#[cfg(test)]
mod daemon_protocol_v1;
#[cfg(test)]
mod daemon_renderer_v1;
mod error;
mod output;
mod worker_path;

#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod args_tests;

use args::ParsedArgs;
use error::CliError;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("latex-render: {error}");
        std::process::exit(error.kind().exit_code());
    }
}

async fn run() -> Result<(), CliError> {
    let arguments = std::env::args_os().skip(1).collect();
    match args::parse_args(arguments) {
        Ok(ParsedArgs::Text(text)) => {
            output::write_stdout(text.as_bytes())?;
        }
        Ok(ParsedArgs::Command(command)) => {
            let output = app::execute(command).await?;
            output::write_output(output)?;
        }
        Err(error) => return Err(error),
    }
    Ok(())
}
