//! Strict arguments for the terminal acceptance tool.

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use latex_terminal::ThemeMode;

const MAX_HOLD_MILLIS: u64 = 60_000;

pub(crate) const HELP: &str = "\
Render one equation through the complete terminal presentation path\n\n\
Usage:\n\
  latex-terminal-smoke [OPTIONS] [SOURCE]\n\n\
Options:\n\
  --backend auto|kitty|iterm2|text  Select transport, default auto\n\
  --geometry COLSxROWS@WIDTHxHEIGHT Set measured cells and pixels\n\
  --resize-geometry VALUE           Redraw once with new measurements\n\
  --theme auto|light|dark           Select equation theme, default auto\n\
  --inline                          Use inline math layout\n\
  --worker PATH                     Select the MathJax worker script\n\
  --node PROGRAM                    Select Node.js, default node\n\
  --hold-ms NUMBER                  Hold the screen through 60000, default 2500\n\
  -h, --help                        Show this help\n";

const DEFAULT_SOURCE: &str = r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackendSelection {
    Auto,
    Kitty,
    Iterm2,
    Text,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometrySpec {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Arguments {
    pub(crate) backend: BackendSelection,
    pub(crate) geometry: Option<GeometrySpec>,
    pub(crate) resize_geometry: Option<GeometrySpec>,
    pub(crate) theme: ThemeMode,
    pub(crate) display_mode: bool,
    pub(crate) worker: PathBuf,
    pub(crate) node: PathBuf,
    pub(crate) hold_millis: u64,
    pub(crate) source: String,
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            backend: BackendSelection::Auto,
            geometry: None,
            resize_geometry: None,
            theme: ThemeMode::Auto,
            display_mode: true,
            worker: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../renderer/mathjax-worker/dist/src/server.js"),
            node: PathBuf::from("node"),
            hold_millis: 2_500,
            source: DEFAULT_SOURCE.to_owned(),
        }
    }
}

impl Arguments {
    pub(crate) fn parse(values: impl IntoIterator<Item = String>) -> Result<ParseResult, ArgError> {
        let mut arguments = Self::default();
        let mut values = values.into_iter();
        let mut seen = Seen::default();
        let mut source_seen = false;
        let mut options_enabled = true;
        while let Some(value) = values.next() {
            if options_enabled && value == "--" {
                options_enabled = false;
                continue;
            }
            if !options_enabled {
                if source_seen {
                    return Err(ArgError::new("at most one source is accepted"));
                }
                arguments.source = value;
                source_seen = true;
                continue;
            }
            match value.as_str() {
                "-h" | "--help" => return Ok(ParseResult::Help),
                "--backend" => {
                    set_once(&mut seen.backend, "backend")?;
                    arguments.backend = parse_backend(next(&mut values, &value)?)?;
                }
                "--geometry" => {
                    set_once(&mut seen.geometry, "geometry")?;
                    arguments.geometry = Some(parse_geometry(next(&mut values, &value)?)?)
                }
                "--resize-geometry" => {
                    set_once(&mut seen.resize_geometry, "resize geometry")?;
                    arguments.resize_geometry = Some(parse_geometry(next(&mut values, &value)?)?)
                }
                "--theme" => {
                    set_once(&mut seen.theme, "theme")?;
                    arguments.theme = parse_theme(next(&mut values, &value)?)?;
                }
                "--inline" => {
                    set_once(&mut seen.inline, "inline")?;
                    arguments.display_mode = false;
                }
                "--worker" => {
                    set_once(&mut seen.worker, "worker")?;
                    arguments.worker = nonempty_path(next(&mut values, &value)?, &value)?;
                }
                "--node" => {
                    set_once(&mut seen.node, "node")?;
                    arguments.node = nonempty_path(next(&mut values, &value)?, &value)?;
                }
                "--hold-ms" => {
                    set_once(&mut seen.hold, "hold milliseconds")?;
                    arguments.hold_millis = next(&mut values, &value)?
                        .parse()
                        .map_err(|_| ArgError::new("hold milliseconds must be an integer"))?;
                    if arguments.hold_millis > MAX_HOLD_MILLIS {
                        return Err(ArgError::new("hold milliseconds exceed 60000"));
                    }
                }
                option if option.starts_with('-') => {
                    return Err(ArgError::new(format!("unknown option {option}")));
                }
                source => {
                    if source_seen {
                        return Err(ArgError::new("at most one source is accepted"));
                    }
                    arguments.source = source.to_owned();
                    source_seen = true;
                }
            }
        }
        if arguments.source.is_empty() {
            return Err(ArgError::new("source must not be empty"));
        }
        if arguments.resize_geometry.is_some() && arguments.geometry.is_none() {
            return Err(ArgError::new("resize geometry requires initial geometry"));
        }
        Ok(ParseResult::Run(arguments))
    }
}

#[derive(Default)]
struct Seen {
    backend: bool,
    geometry: bool,
    resize_geometry: bool,
    theme: bool,
    inline: bool,
    worker: bool,
    node: bool,
    hold: bool,
}

#[derive(Debug)]
pub(crate) enum ParseResult {
    Help,
    Run(Arguments),
}

#[derive(Debug)]
pub(crate) struct ArgError(String);

impl ArgError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ArgError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ArgError {}

fn parse_backend(value: String) -> Result<BackendSelection, ArgError> {
    match value.as_str() {
        "auto" => Ok(BackendSelection::Auto),
        "kitty" => Ok(BackendSelection::Kitty),
        "iterm2" => Ok(BackendSelection::Iterm2),
        "text" => Ok(BackendSelection::Text),
        _ => Err(ArgError::new(
            "backend must be auto, kitty, iterm2, or text",
        )),
    }
}

fn parse_theme(value: String) -> Result<ThemeMode, ArgError> {
    match value.as_str() {
        "auto" => Ok(ThemeMode::Auto),
        "light" => Ok(ThemeMode::Light),
        "dark" => Ok(ThemeMode::Dark),
        _ => Err(ArgError::new("theme must be auto, light, or dark")),
    }
}

fn parse_geometry(value: String) -> Result<GeometrySpec, ArgError> {
    let (cells, pixels) = value
        .split_once('@')
        .ok_or_else(|| ArgError::new("geometry must use COLSxROWS@WIDTHxHEIGHT"))?;
    let (columns, rows) = dimensions(cells)?;
    let (width_px, height_px) = dimensions(pixels)?;
    let columns =
        u16::try_from(columns).map_err(|_| ArgError::new("terminal columns exceed their limit"))?;
    let rows =
        u16::try_from(rows).map_err(|_| ArgError::new("terminal rows exceed their limit"))?;
    Ok(GeometrySpec {
        columns,
        rows,
        width_px,
        height_px,
    })
}

fn dimensions(value: &str) -> Result<(u32, u32), ArgError> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| ArgError::new("geometry dimensions must be separated by x"))?;
    let width = width
        .parse::<u32>()
        .map_err(|_| ArgError::new("geometry dimensions must be integers"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| ArgError::new("geometry dimensions must be integers"))?;
    if width == 0 || height == 0 {
        return Err(ArgError::new("geometry dimensions must be nonzero"));
    }
    Ok((width, height))
}

fn next(values: &mut impl Iterator<Item = String>, option: &str) -> Result<String, ArgError> {
    values
        .next()
        .ok_or_else(|| ArgError::new(format!("{option} requires a value")))
}

fn nonempty_path(value: String, option: &str) -> Result<PathBuf, ArgError> {
    if value.is_empty() {
        Err(ArgError::new(format!("{option} requires a nonempty path")))
    } else {
        Ok(PathBuf::from(value))
    }
}

fn set_once(seen: &mut bool, name: &str) -> Result<(), ArgError> {
    if *seen {
        Err(ArgError::new(format!(
            "option {name} may be supplied only once"
        )))
    } else {
        *seen = true;
        Ok(())
    }
}
