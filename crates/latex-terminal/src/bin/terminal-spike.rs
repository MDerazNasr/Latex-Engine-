//! Runnable terminal image lifecycle feasibility experiment.

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::io::IsTerminal as _;
use std::io::Write as _;
use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use latex_terminal::ImageDraw;
use latex_terminal::ImageRenderState;
use latex_terminal::ImageSource;
use latex_terminal::PlacementSize;
use latex_terminal::TerminalBackend;
use latex_terminal::TerminalEnvironment;
use latex_terminal::detect_terminal_support;

const SOURCE_FALLBACK: &str = r"\[x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}\]";
const IMAGE_ID: NonZeroU32 = NonZeroU32::new(0xC0E1).expect("image ID is nonzero");
const MAX_HOLD_MILLIS: u64 = 60_000;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("terminal spike failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse(env::args().skip(1))?;
    let detected = detect_terminal_support(&TerminalEnvironment::from_current_process(
        io::stdout().is_terminal(),
    ));
    let backend = arguments.selection.resolve(detected.backend);
    if backend == TerminalBackend::Text {
        println!("{SOURCE_FALLBACK}");
        return Ok(());
    }

    let mut session = ScreenSession::start(backend, arguments.png)?;
    session.draw(arguments.columns, arguments.rows)?;
    if arguments.resize {
        let first_wait = arguments.hold_millis / 2;
        thread::sleep(Duration::from_millis(first_wait));
        session.draw(arguments.columns.saturating_sub(12).max(1), arguments.rows)?;
        thread::sleep(Duration::from_millis(arguments.hold_millis - first_wait));
    } else {
        thread::sleep(Duration::from_millis(arguments.hold_millis));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendSelection {
    Auto,
    Kitty,
    Iterm2,
    Text,
}

impl BackendSelection {
    fn resolve(self, detected: TerminalBackend) -> TerminalBackend {
        match self {
            Self::Auto => detected,
            Self::Kitty => TerminalBackend::KittyDirect,
            Self::Iterm2 => TerminalBackend::KittyLocalFile,
            Self::Text => TerminalBackend::Text,
        }
    }
}

#[derive(Debug)]
struct Arguments {
    selection: BackendSelection,
    png: PathBuf,
    columns: u16,
    rows: u16,
    hold_millis: u64,
    resize: bool,
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = String>) -> Result<Self, ArgumentError> {
        let mut arguments = Self {
            selection: BackendSelection::Auto,
            png: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/terminal/quadratic-formula.png"),
            columns: 56,
            rows: 6,
            hold_millis: 2_500,
            resize: true,
        };
        let mut values = values.into_iter();
        while let Some(argument) = values.next() {
            match argument.as_str() {
                "--backend" => {
                    arguments.selection = parse_backend(next_value(&mut values, "--backend")?)?;
                }
                "--png" => arguments.png = PathBuf::from(next_value(&mut values, "--png")?),
                "--columns" => {
                    arguments.columns = parse_nonzero(next_value(&mut values, "--columns")?)?;
                }
                "--rows" => {
                    arguments.rows = parse_nonzero(next_value(&mut values, "--rows")?)?;
                }
                "--hold-ms" => {
                    arguments.hold_millis = next_value(&mut values, "--hold-ms")?
                        .parse()
                        .map_err(|_| ArgumentError::new("--hold-ms must be an integer"))?;
                    if arguments.hold_millis > MAX_HOLD_MILLIS {
                        return Err(ArgumentError::new("--hold-ms exceeds 60000"));
                    }
                }
                "--no-resize" => arguments.resize = false,
                _ => return Err(ArgumentError::new(format!("unknown argument {argument}"))),
            }
        }
        Ok(arguments)
    }
}

fn parse_backend(value: String) -> Result<BackendSelection, ArgumentError> {
    match value.as_str() {
        "auto" => Ok(BackendSelection::Auto),
        "kitty" => Ok(BackendSelection::Kitty),
        "iterm2" => Ok(BackendSelection::Iterm2),
        "text" => Ok(BackendSelection::Text),
        _ => Err(ArgumentError::new(
            "--backend must be auto, kitty, iterm2, or text",
        )),
    }
}

fn parse_nonzero(value: String) -> Result<u16, ArgumentError> {
    value
        .parse::<NonZeroU16>()
        .map(NonZeroU16::get)
        .map_err(|_| ArgumentError::new("terminal dimensions must be nonzero integers"))
}

fn next_value(
    values: &mut impl Iterator<Item = String>,
    argument: &str,
) -> Result<String, ArgumentError> {
    values
        .next()
        .ok_or_else(|| ArgumentError::new(format!("{argument} requires a value")))
}

#[derive(Debug)]
struct ArgumentError(String);

impl ArgumentError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Error for ArgumentError {}

struct ScreenSession {
    stdout: io::Stdout,
    state: ImageRenderState,
    backend: TerminalBackend,
    png: PathBuf,
}

impl ScreenSession {
    fn start(backend: TerminalBackend, png: PathBuf) -> Result<Self, Box<dyn Error>> {
        let mut stdout = io::stdout();
        stdout.write_all(b"\x1b[?1049h\x1b[2J\x1b[?25l")?;
        stdout.flush()?;
        Ok(Self {
            stdout,
            state: ImageRenderState::default(),
            backend,
            png,
        })
    }

    fn draw(&mut self, columns: u16, rows: u16) -> Result<(), Box<dyn Error>> {
        let source = match self.backend {
            TerminalBackend::KittyDirect => ImageSource::PngBytes(fs::read(&self.png)?),
            TerminalBackend::KittyLocalFile => ImageSource::LocalPng(self.png.clone()),
            TerminalBackend::Text => unreachable!("text mode does not open a screen session"),
        };
        let command = self.state.render(
            self.backend,
            Some(ImageDraw {
                image_id: IMAGE_ID,
                x: 2,
                y: 1,
                size: PlacementSize::new(
                    NonZeroU16::new(columns).expect("columns were validated"),
                    NonZeroU16::new(rows).expect("rows were validated"),
                ),
                source,
            }),
        )?;
        self.stdout.write_all(&command)?;
        write!(
            self.stdout,
            "\x1b[{};3HRendered with {:?} at {columns} by {rows} cells",
            u32::from(rows) + 4,
            self.backend,
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

impl Drop for ScreenSession {
    fn drop(&mut self) {
        let _ = self.stdout.write_all(&self.state.clear());
        let _ = self.stdout.write_all(b"\x1b[?25h\x1b[?1049l");
        let _ = self.stdout.flush();
    }
}
