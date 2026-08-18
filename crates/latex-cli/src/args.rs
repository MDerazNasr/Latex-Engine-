//! Strict argument parsing for the standalone commands.

use std::ffi::OsString;
use std::path::PathBuf;

use latex_render_core::{MAX_SCALE, MAX_WIDTH_PX, MIN_SCALE, Rgba};

use crate::error::{CliError, CliErrorKind};

pub(crate) const ROOT_HELP: &str = "\
Codex LaTeX renderer\n\n\
Usage:\n\
  latex-render render [OPTIONS] [SOURCE]\n\
  latex-render check [OPTIONS]\n\
  latex-render doctor [OPTIONS]\n\n\
Commands:\n\
  render  Render one TeX math fragment to SVG or PNG\n\
  check   Validate the worker and native rendering pipeline\n\
  doctor  Diagnose rendering and terminal image support\n\n\
Run 'latex-render COMMAND --help' for command options.\n";

pub(crate) const RENDER_HELP: &str = "\
Render one TeX math fragment\n\n\
Usage:\n\
  latex-render render [OPTIONS] [SOURCE]\n\n\
Options:\n\
  --display                 Use display math layout\n\
  --format svg|png          Select output format, default svg\n\
  --output PATH|-           Write to a file or stdout, default stdout\n\
  --force                   Replace an existing output file\n\
  --foreground #RRGGBB      Set foreground color, default #e6edf3\n\
  --background VALUE        Set #RRGGBB or transparent\n\
  --scale NUMBER            Set scale from 0.5 through 4, default 2\n\
  --max-width PIXELS        Set maximum width through 4096, default 1200\n\
  --worker PATH             Select the MathJax worker script\n\
  --node PROGRAM            Select the Node.js executable, default node\n\
  -h, --help                Show this help\n\n\
When SOURCE is absent, redirected UTF 8 stdin is used.\n";

pub(crate) const CHECK_HELP: &str = "\
Validate the local rendering pipeline\n\n\
Usage:\n\
  latex-render check [OPTIONS]\n\n\
Options:\n\
  --worker PATH   Select the MathJax worker script\n\
  --node PROGRAM  Select the Node.js executable, default node\n\
  -h, --help      Show this help\n";

pub(crate) const DOCTOR_HELP: &str = "\
Diagnose rendering and terminal image support\n\n\
Usage:\n\
  latex-render doctor [OPTIONS]\n\n\
Options:\n\
  --worker PATH   Select the MathJax worker script\n\
  --node PROGRAM  Select the Node.js executable, default node\n\
  -h, --help      Show this help\n";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParsedArgs {
    Text(String),
    Command(CliCommand),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CliCommand {
    Render(RenderOptions),
    Check(WorkerOptions),
    Doctor(WorkerOptions),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Svg,
    Png,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkerOptions {
    pub(crate) worker: Option<PathBuf>,
    pub(crate) node: PathBuf,
}

impl Default for WorkerOptions {
    fn default() -> Self {
        Self {
            worker: None,
            node: PathBuf::from("node"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderOptions {
    pub(crate) source: Option<String>,
    pub(crate) display_mode: bool,
    pub(crate) format: OutputFormat,
    pub(crate) output: Option<PathBuf>,
    pub(crate) force: bool,
    pub(crate) foreground: Rgba,
    pub(crate) background: Option<Rgba>,
    pub(crate) scale: f32,
    pub(crate) max_width_px: u32,
    pub(crate) worker: WorkerOptions,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            source: None,
            display_mode: false,
            format: OutputFormat::Svg,
            output: None,
            force: false,
            foreground: Rgba::opaque(230, 237, 243),
            background: None,
            scale: 2.0,
            max_width_px: 1200,
            worker: WorkerOptions::default(),
        }
    }
}

pub(crate) fn parse_args(arguments: Vec<OsString>) -> Result<ParsedArgs, CliError> {
    let Some((command, remaining)) = arguments.split_first() else {
        return Ok(ParsedArgs::Text(ROOT_HELP.to_owned()));
    };
    match command.to_str() {
        Some("-h" | "--help") => Ok(ParsedArgs::Text(ROOT_HELP.to_owned())),
        Some("-V" | "--version") if remaining.is_empty() => Ok(ParsedArgs::Text(format!(
            "latex-render {}\n",
            env!("CARGO_PKG_VERSION")
        ))),
        Some("render") => parse_render(remaining),
        Some("check") => parse_check(remaining),
        Some("doctor") => parse_doctor(remaining),
        Some(_) => Err(usage("Expected the render, check, or doctor command")),
        None => Err(usage("Command must contain valid UTF 8")),
    }
}

fn parse_render(arguments: &[OsString]) -> Result<ParsedArgs, CliError> {
    let mut options = RenderOptions::default();
    let mut seen = SeenRender::default();
    let mut index = 0usize;
    let mut options_enabled = true;
    while index < arguments.len() {
        let value = &arguments[index];
        let text = value
            .to_str()
            .ok_or_else(|| usage("TeX source and option names must contain valid UTF 8"))?;
        if options_enabled && text == "--" {
            options_enabled = false;
            index += 1;
            continue;
        }
        if options_enabled && matches!(text, "-h" | "--help") {
            return Ok(ParsedArgs::Text(RENDER_HELP.to_owned()));
        }
        if options_enabled && text.starts_with('-') {
            index = parse_render_option(arguments, index, text, &mut options, &mut seen)?;
        } else {
            if options.source.is_some() {
                return Err(usage("Render accepts at most one source argument"));
            }
            options.source = Some(text.to_owned());
            index += 1;
        }
    }
    if options.force && options.output.is_none() {
        return Err(usage("The force option requires a file output path"));
    }
    Ok(ParsedArgs::Command(CliCommand::Render(options)))
}

fn parse_render_option(
    arguments: &[OsString],
    index: usize,
    option: &str,
    options: &mut RenderOptions,
    seen: &mut SeenRender,
) -> Result<usize, CliError> {
    match option {
        "--display" => {
            set_once(&mut seen.display, "display")?;
            options.display_mode = true;
            Ok(index + 1)
        }
        "--force" => {
            set_once(&mut seen.force, "force")?;
            options.force = true;
            Ok(index + 1)
        }
        "--format" => {
            set_once(&mut seen.format, "format")?;
            let value = next_text(arguments, index, option)?;
            options.format = match value {
                "svg" => OutputFormat::Svg,
                "png" => OutputFormat::Png,
                _ => return Err(usage("Format must be svg or png")),
            };
            Ok(index + 2)
        }
        "--output" => {
            set_once(&mut seen.output, "output")?;
            let value = next_path(arguments, index, option)?;
            options.output = (value.as_os_str() != "-").then_some(value);
            Ok(index + 2)
        }
        "--foreground" => {
            set_once(&mut seen.foreground, "foreground")?;
            options.foreground = parse_color(next_text(arguments, index, option)?)?;
            Ok(index + 2)
        }
        "--background" => {
            set_once(&mut seen.background, "background")?;
            let value = next_text(arguments, index, option)?;
            options.background = if value == "transparent" {
                None
            } else {
                Some(parse_color(value)?)
            };
            Ok(index + 2)
        }
        "--scale" => {
            set_once(&mut seen.scale, "scale")?;
            let value = next_text(arguments, index, option)?;
            options.scale = value
                .parse::<f32>()
                .map_err(|_| usage(format!("Scale must be between {MIN_SCALE} and {MAX_SCALE}")))?;
            if !options.scale.is_finite() || options.scale < MIN_SCALE || options.scale > MAX_SCALE
            {
                return Err(usage(format!(
                    "Scale must be between {MIN_SCALE} and {MAX_SCALE}"
                )));
            }
            Ok(index + 2)
        }
        "--max-width" => {
            set_once(&mut seen.max_width, "max width")?;
            let value = next_text(arguments, index, option)?;
            options.max_width_px = value.parse::<u32>().map_err(|_| {
                usage(format!(
                    "Maximum width must be between 1 and {MAX_WIDTH_PX}"
                ))
            })?;
            if options.max_width_px == 0 || options.max_width_px > MAX_WIDTH_PX {
                return Err(usage(format!(
                    "Maximum width must be between 1 and {MAX_WIDTH_PX}"
                )));
            }
            Ok(index + 2)
        }
        "--worker" => {
            set_once(&mut seen.worker, "worker")?;
            options.worker.worker = Some(next_path(arguments, index, option)?);
            Ok(index + 2)
        }
        "--node" => {
            set_once(&mut seen.node, "node")?;
            options.worker.node = next_path(arguments, index, option)?;
            Ok(index + 2)
        }
        _ => Err(usage(format!("Unknown render option {option}"))),
    }
}

fn parse_check(arguments: &[OsString]) -> Result<ParsedArgs, CliError> {
    parse_worker_command(arguments, "Check", CHECK_HELP, CliCommand::Check)
}

fn parse_doctor(arguments: &[OsString]) -> Result<ParsedArgs, CliError> {
    parse_worker_command(arguments, "Doctor", DOCTOR_HELP, CliCommand::Doctor)
}

fn parse_worker_command(
    arguments: &[OsString],
    command_name: &str,
    help: &str,
    command: impl FnOnce(WorkerOptions) -> CliCommand,
) -> Result<ParsedArgs, CliError> {
    let mut options = WorkerOptions::default();
    let mut seen_worker = false;
    let mut seen_node = false;
    let mut index = 0usize;
    while index < arguments.len() {
        let option = arguments[index].to_str().ok_or_else(|| {
            usage(format!(
                "{command_name} option names must contain valid UTF 8"
            ))
        })?;
        match option {
            "-h" | "--help" => return Ok(ParsedArgs::Text(help.to_owned())),
            "--worker" => {
                set_once(&mut seen_worker, "worker")?;
                options.worker = Some(next_path(arguments, index, option)?);
                index += 2;
            }
            "--node" => {
                set_once(&mut seen_node, "node")?;
                options.node = next_path(arguments, index, option)?;
                index += 2;
            }
            _ => {
                return Err(usage(format!(
                    "Unknown {} option {option}",
                    command_name.to_ascii_lowercase()
                )));
            }
        }
    }
    Ok(ParsedArgs::Command(command(options)))
}

fn next_text<'a>(
    arguments: &'a [OsString],
    index: usize,
    option: &str,
) -> Result<&'a str, CliError> {
    arguments
        .get(index + 1)
        .ok_or_else(|| usage(format!("Option {option} requires a value")))?
        .to_str()
        .ok_or_else(|| usage(format!("Option {option} requires valid UTF 8")))
}

fn next_path(arguments: &[OsString], index: usize, option: &str) -> Result<PathBuf, CliError> {
    let value = arguments
        .get(index + 1)
        .ok_or_else(|| usage(format!("Option {option} requires a value")))?;
    if value.is_empty() {
        return Err(usage(format!("Option {option} requires a nonempty value")));
    }
    Ok(PathBuf::from(value))
}

fn parse_color(value: &str) -> Result<Rgba, CliError> {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes.first() != Some(&b'#') {
        return Err(usage("Color must use the form #RRGGBB"));
    }
    let channel = |start| {
        u8::from_str_radix(&value[start..start + 2], 16)
            .map_err(|_| usage("Color must use the form #RRGGBB"))
    };
    Ok(Rgba::opaque(channel(1)?, channel(3)?, channel(5)?))
}

fn set_once(seen: &mut bool, name: &str) -> Result<(), CliError> {
    if *seen {
        return Err(usage(format!("Option {name} may be supplied only once")));
    }
    *seen = true;
    Ok(())
}

fn usage(message: impl Into<String>) -> CliError {
    CliError::new(CliErrorKind::Usage, message)
}

#[derive(Default)]
struct SeenRender {
    display: bool,
    format: bool,
    output: bool,
    force: bool,
    foreground: bool,
    background: bool,
    scale: bool,
    max_width: bool,
    worker: bool,
    node: bool,
}
