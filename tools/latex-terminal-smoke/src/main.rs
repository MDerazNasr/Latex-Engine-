//! End to end terminal equation presentation acceptance tool.

mod args;
#[cfg(test)]
mod args_tests;
mod pipeline;
mod screen;

use std::env;
use std::error::Error;
use std::io::{self, IsTerminal as _};
use std::process::ExitCode;
use std::time::Duration;

use args::{Arguments, BackendSelection, HELP, ParseResult};
use latex_terminal::{
    TerminalBackend, TerminalEnvironment, TerminalPresenter, detect_terminal_support,
};
use pipeline::{publish_equation, render_equation, source_store};
use screen::ScreenSession;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("terminal smoke failed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let arguments = match Arguments::parse(env::args().skip(1))? {
        ParseResult::Help => {
            print!("{HELP}");
            return Ok(());
        }
        ParseResult::Run(arguments) => arguments,
    };
    let backend = resolve_backend(arguments.backend);
    if backend == TerminalBackend::Text {
        println!("{}", arguments.source);
        return Ok(());
    }
    let geometry = arguments
        .geometry
        .ok_or("image backend requires measured terminal geometry")?;
    let equation = match render_equation(&arguments, geometry).await {
        Ok(equation) => equation,
        Err(error) => {
            println!("{}", arguments.source);
            return Err(error);
        }
    };
    if let Err(error) = present(&arguments, backend, geometry, &equation).await {
        println!("{}", arguments.source);
        return Err(error);
    }
    Ok(())
}

async fn present(
    arguments: &Arguments,
    backend: TerminalBackend,
    geometry: args::GeometrySpec,
    equation: &pipeline::RenderedEquation,
) -> Result<(), Box<dyn Error>> {
    let mut presenter = TerminalPresenter::new(backend);
    let mut store = source_store(backend)?;
    let command = publish_equation(&mut presenter, store.as_mut(), equation, geometry, 1)?;
    let mut screen = ScreenSession::start()?;
    screen.draw(&command)?;

    if let Some(resize) = arguments.resize_geometry {
        tokio::time::sleep(Duration::from_millis(arguments.hold_millis / 2)).await;
        let command = publish_equation(&mut presenter, store.as_mut(), equation, resize, 1)?;
        screen.draw(&command)?;
        tokio::time::sleep(Duration::from_millis(
            arguments.hold_millis - arguments.hold_millis / 2,
        ))
        .await;
    } else {
        tokio::time::sleep(Duration::from_millis(arguments.hold_millis)).await;
    }
    Ok(())
}

fn resolve_backend(selection: BackendSelection) -> TerminalBackend {
    match selection {
        BackendSelection::Auto => {
            let environment = TerminalEnvironment::from_current_process(io::stdout().is_terminal());
            detect_terminal_support(&environment).backend
        }
        BackendSelection::Kitty => TerminalBackend::KittyDirect,
        BackendSelection::Iterm2 => TerminalBackend::KittyLocalFile,
        BackendSelection::Text => TerminalBackend::Text,
    }
}
