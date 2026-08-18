//! Pure conversion from equation pixels to reserved terminal cells.

use std::fmt::{Display, Formatter};
use std::num::NonZeroU16;

use crate::PlacementSize;

/// Validated terminal cell and pixel measurements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalGeometry {
    columns: NonZeroU16,
    rows: NonZeroU16,
    width_px: u32,
    height_px: u32,
    cell_width_px: u32,
    cell_height_px: u32,
}

impl TerminalGeometry {
    /// Creates geometry only when every cell has at least one measured pixel.
    pub fn new(
        columns: u16,
        rows: u16,
        width_px: u32,
        height_px: u32,
    ) -> Result<Self, LayoutError> {
        let columns = NonZeroU16::new(columns).ok_or(LayoutError::InvalidTerminalGeometry)?;
        let rows = NonZeroU16::new(rows).ok_or(LayoutError::InvalidTerminalGeometry)?;
        let cell_width_px = width_px / u32::from(columns.get());
        let cell_height_px = height_px / u32::from(rows.get());
        if cell_width_px == 0 || cell_height_px == 0 {
            return Err(LayoutError::InvalidTerminalGeometry);
        }
        Ok(Self {
            columns,
            rows,
            width_px,
            height_px,
            cell_width_px,
            cell_height_px,
        })
    }

    /// Returns the measured terminal column count.
    pub const fn columns(self) -> NonZeroU16 {
        self.columns
    }

    /// Returns the measured terminal row count.
    pub const fn rows(self) -> NonZeroU16 {
        self.rows
    }

    /// Returns the measured terminal pixel width.
    pub const fn width_px(self) -> u32 {
        self.width_px
    }

    /// Returns the measured terminal pixel height.
    pub const fn height_px(self) -> u32 {
        self.height_px
    }

    /// Returns the whole pixel width used for one cell.
    pub const fn cell_width_px(self) -> u32 {
        self.cell_width_px
    }

    /// Returns the whole pixel height used for one cell.
    pub const fn cell_height_px(self) -> u32 {
        self.cell_height_px
    }
}

/// Validated pixel geometry returned by the math worker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MathGeometry {
    width_px: u32,
    height_px: u32,
    baseline_px: Option<f32>,
    display_mode: bool,
}

impl MathGeometry {
    /// Creates geometry with an optional baseline inside the image height.
    pub fn new(
        width_px: u32,
        height_px: u32,
        baseline_px: Option<f32>,
        display_mode: bool,
    ) -> Result<Self, LayoutError> {
        if width_px == 0
            || height_px == 0
            || baseline_px
                .is_some_and(|value| !value.is_finite() || value < 0.0 || value > height_px as f32)
        {
            return Err(LayoutError::InvalidMathGeometry);
        }
        Ok(Self {
            width_px,
            height_px,
            baseline_px,
            display_mode,
        })
    }
}

/// Bounded user policy applied during terminal layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutPolicy {
    max_width_percent: u8,
    max_height_rows: NonZeroU16,
    inline_baseline_percent: u8,
}

impl LayoutPolicy {
    /// Creates policy values within their stable public bounds.
    pub fn new(
        max_width_percent: u8,
        max_height_rows: u16,
        inline_baseline_percent: u8,
    ) -> Result<Self, LayoutError> {
        if !(1..=100).contains(&max_width_percent) || !(1..100).contains(&inline_baseline_percent) {
            return Err(LayoutError::InvalidPolicy);
        }
        Ok(Self {
            max_width_percent,
            max_height_rows: NonZeroU16::new(max_height_rows).ok_or(LayoutError::InvalidPolicy)?,
            inline_baseline_percent,
        })
    }

    /// Returns the maximum block width as a viewport percentage.
    pub const fn max_width_percent(self) -> u8 {
        self.max_width_percent
    }

    /// Returns the maximum reserved block rows.
    pub const fn max_height_rows(self) -> NonZeroU16 {
        self.max_height_rows
    }

    /// Returns the text baseline target as a cell height percentage.
    pub const fn inline_baseline_percent(self) -> u8 {
        self.inline_baseline_percent
    }
}

impl Default for LayoutPolicy {
    fn default() -> Self {
        Self {
            max_width_percent: 90,
            max_height_rows: NonZeroU16::new(18).expect("default rows are nonzero"),
            inline_baseline_percent: 80,
        }
    }
}

/// Whether an equation remains in prose or occupies a centered block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutMode {
    /// A one row image beginning at the current prose column.
    Inline,
    /// A centered image with independently reserved rows.
    Block,
}

/// A pixel rectangle inside the transparent cell aligned canvas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelRect {
    /// Horizontal pixel offset inside the canvas.
    pub x: u32,
    /// Vertical pixel offset inside the canvas.
    pub y: u32,
    /// Uniformly scaled content width.
    pub width: u32,
    /// Uniformly scaled content height.
    pub height: u32,
}

/// Complete terminal layout for one rendered equation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageLayout {
    /// Inline or promoted block presentation.
    pub mode: LayoutMode,
    /// Zero based terminal column for the placement.
    pub column: u16,
    /// Exact cell rectangle reserved by the transcript.
    pub placement: PlacementSize,
    /// Exact transparent canvas width in pixels.
    pub canvas_width_px: u32,
    /// Exact transparent canvas height in pixels.
    pub canvas_height_px: u32,
    /// Equation content rectangle inside the canvas.
    pub content: PixelRect,
    /// Uniform content scale relative to worker output.
    pub scale: f32,
    /// Scaled baseline measured from the canvas top.
    pub baseline_px: Option<f32>,
}

/// Invalid terminal, equation, or policy inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutError {
    /// Terminal cells or pixels were missing or incoherent.
    InvalidTerminalGeometry,
    /// Equation dimensions or baseline were invalid.
    InvalidMathGeometry,
    /// A layout limit was zero or outside its allowed percentage.
    InvalidPolicy,
}

impl Display for LayoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTerminalGeometry => {
                formatter.write_str("terminal pixel geometry is unavailable")
            }
            Self::InvalidMathGeometry => formatter.write_str("math geometry is invalid"),
            Self::InvalidPolicy => formatter.write_str("terminal layout policy is invalid"),
        }
    }
}

impl std::error::Error for LayoutError {}

/// Computes a stable cell and pixel layout without terminal side effects.
pub fn layout_math(
    terminal: TerminalGeometry,
    math: MathGeometry,
    current_column: u16,
    policy: LayoutPolicy,
) -> ImageLayout {
    if !math.display_mode
        && let Some(layout) = inline_layout(terminal, math, current_column, policy)
    {
        return layout;
    }
    block_layout(terminal, math, policy)
}

fn inline_layout(
    terminal: TerminalGeometry,
    math: MathGeometry,
    current_column: u16,
    policy: LayoutPolicy,
) -> Option<ImageLayout> {
    let remaining = terminal.columns.get().saturating_sub(current_column);
    if remaining == 0 {
        return None;
    }
    let maximum_columns = maximum_block_columns(terminal, policy).min(remaining);
    let target_baseline =
        terminal.cell_height_px as f32 * f32::from(policy.inline_baseline_percent) / 100.0;
    let scale = inline_scale(math, terminal.cell_height_px, target_baseline);
    let content_width = scaled_length(math.width_px, scale);
    let content_height = scaled_length(math.height_px, scale);
    let columns = cells_for(content_width, terminal.cell_width_px)?;
    if columns.get() > maximum_columns {
        return None;
    }
    let canvas_width = u32::from(columns.get()) * terminal.cell_width_px;
    let canvas_height = terminal.cell_height_px;
    let content_y = inline_y(math, scale, target_baseline, canvas_height, content_height);
    Some(ImageLayout {
        mode: LayoutMode::Inline,
        column: current_column,
        placement: PlacementSize::new(columns, NonZeroU16::MIN),
        canvas_width_px: canvas_width,
        canvas_height_px: canvas_height,
        content: PixelRect {
            x: 0,
            y: content_y,
            width: content_width,
            height: content_height,
        },
        scale,
        baseline_px: math
            .baseline_px
            .map(|baseline| content_y as f32 + baseline * scale),
    })
}

fn block_layout(
    terminal: TerminalGeometry,
    math: MathGeometry,
    policy: LayoutPolicy,
) -> ImageLayout {
    let maximum_columns = maximum_block_columns(terminal, policy);
    let maximum_rows = terminal.rows.get().min(policy.max_height_rows.get());
    let maximum_width = u32::from(maximum_columns) * terminal.cell_width_px;
    let maximum_height = u32::from(maximum_rows) * terminal.cell_height_px;
    let scale = fit_scale(math.width_px, math.height_px, maximum_width, maximum_height);
    let content_width = scaled_length(math.width_px, scale);
    let content_height = scaled_length(math.height_px, scale);
    let columns = cells_for(content_width, terminal.cell_width_px)
        .expect("positive content occupies at least one column");
    let rows = cells_for(content_height, terminal.cell_height_px)
        .expect("positive content occupies at least one row");
    let canvas_width = u32::from(columns.get()) * terminal.cell_width_px;
    let canvas_height = u32::from(rows.get()) * terminal.cell_height_px;
    let x = (canvas_width - content_width) / 2;
    let y = (canvas_height - content_height) / 2;
    ImageLayout {
        mode: LayoutMode::Block,
        column: (terminal.columns.get() - columns.get()) / 2,
        placement: PlacementSize::new(columns, rows),
        canvas_width_px: canvas_width,
        canvas_height_px: canvas_height,
        content: PixelRect {
            x,
            y,
            width: content_width,
            height: content_height,
        },
        scale,
        baseline_px: math.baseline_px.map(|baseline| y as f32 + baseline * scale),
    }
}

fn maximum_block_columns(terminal: TerminalGeometry, policy: LayoutPolicy) -> u16 {
    let columns = u32::from(terminal.columns.get()) * u32::from(policy.max_width_percent) / 100;
    columns.max(1) as u16
}

fn inline_scale(math: MathGeometry, cell_height: u32, target_baseline: f32) -> f32 {
    let mut scale = 1.0f32.min(cell_height as f32 / math.height_px as f32);
    if let Some(baseline) = math.baseline_px {
        if baseline > 0.0 {
            scale = scale.min(target_baseline / baseline);
        }
        let below = math.height_px as f32 - baseline;
        let available_below = cell_height as f32 - target_baseline;
        if below > 0.0 {
            scale = scale.min(available_below / below);
        }
    }
    scale.max(1.0 / math.width_px.max(math.height_px) as f32)
}

fn inline_y(
    math: MathGeometry,
    scale: f32,
    target_baseline: f32,
    canvas_height: u32,
    content_height: u32,
) -> u32 {
    let maximum = canvas_height.saturating_sub(content_height);
    math.baseline_px
        .map(|baseline| {
            (target_baseline - baseline * scale)
                .round()
                .clamp(0.0, maximum as f32)
        })
        .unwrap_or_else(|| (maximum / 2) as f32) as u32
}

fn fit_scale(width: u32, height: u32, maximum_width: u32, maximum_height: u32) -> f32 {
    1.0f32
        .min(maximum_width as f32 / width as f32)
        .min(maximum_height as f32 / height as f32)
}

fn scaled_length(length: u32, scale: f32) -> u32 {
    ((length as f32 * scale).ceil() as u32).max(1)
}

fn cells_for(pixels: u32, cell_pixels: u32) -> Option<NonZeroU16> {
    let cells = pixels.div_ceil(cell_pixels);
    u16::try_from(cells).ok().and_then(NonZeroU16::new)
}
