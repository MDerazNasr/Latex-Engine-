//! Generation checked bridge from terminal layout to image placement.

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read as _;
use std::num::NonZeroU32;

use latex_render_svg::{
    FittedRasterRequest, PngImage, RasterLimits, RasterRect, SanitizedSvg, rasterize_svg_fitted,
    validate_png,
};

use crate::{
    ImageDraw, ImageLayout, ImageRenderState, ImageSource, PlacementError, TerminalBackend,
};

/// Immutable work issued for one terminal layout generation.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationJob {
    generation: u64,
    backend: TerminalBackend,
    image_id: NonZeroU32,
    row: u16,
    layout: ImageLayout,
}

impl PresentationJob {
    /// Returns the fitted raster request derived from the reserved cells.
    pub fn raster_request(&self) -> FittedRasterRequest {
        FittedRasterRequest {
            canvas_width_px: self.layout.canvas_width_px,
            canvas_height_px: self.layout.canvas_height_px,
            content: RasterRect {
                x_px: self.layout.content.x,
                y_px: self.layout.content.y,
                width_px: self.layout.content.width,
                height_px: self.layout.content.height,
            },
        }
    }

    /// Returns the generation captured when the job began.
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the reserved layout carried by this job.
    pub const fn layout(&self) -> ImageLayout {
        self.layout
    }
}

/// PNG output correlated with the job that requested it.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterizedPresentation {
    job: PresentationJob,
    image: PngImage,
}

impl RasterizedPresentation {
    /// Correlates output only when it exactly matches the reserved canvas.
    pub fn new(job: PresentationJob, image: PngImage) -> Result<Self, PresentationError> {
        if image.width_px != job.layout.canvas_width_px
            || image.height_px != job.layout.canvas_height_px
        {
            return Err(PresentationError::RasterDimensionsMismatch);
        }
        let decoded = validate_png(&image.bytes, RasterLimits::default())
            .map_err(|_| PresentationError::InvalidRasterBytes)?;
        if decoded.width_px != image.width_px || decoded.height_px != image.height_px {
            return Err(PresentationError::InvalidRasterBytes);
        }
        Ok(Self { job, image })
    }

    /// Returns the bounded PNG bytes for backend preparation.
    pub fn png_bytes(&self) -> &[u8] {
        &self.image.bytes
    }

    /// Creates the source used by direct Kitty protocol transfer.
    pub fn direct_source(&self) -> ImageSource {
        ImageSource::PngBytes(self.image.bytes.clone())
    }
}

/// Result of attempting to publish a correlated raster.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishOutcome {
    /// The current raster produced these terminal protocol bytes.
    Published(Vec<u8>),
    /// The raster belongs to a superseded generation or backend.
    Stale,
}

/// Failure while coordinating terminal presentation.
#[derive(Debug)]
pub enum PresentationError {
    /// No further unique generation could be allocated.
    GenerationExhausted,
    /// Raster output did not match its reserved cell canvas.
    RasterDimensionsMismatch,
    /// Image source bytes did not match the correlated raster.
    RasterSourceMismatch,
    /// Raster bytes were empty, malformed, or exceeded their global bound.
    InvalidRasterBytes,
    /// A local PNG source could not be read for correlation.
    LocalFile(std::io::Error),
    /// Fitted SVG rasterization failed.
    Raster(latex_render_core::RenderError),
    /// Terminal placement encoding failed.
    Placement(PlacementError),
}

impl Display for PresentationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerationExhausted => formatter.write_str("presentation generation exhausted"),
            Self::RasterDimensionsMismatch => {
                formatter.write_str("raster dimensions do not match the reserved canvas")
            }
            Self::RasterSourceMismatch => {
                formatter.write_str("image source does not match the correlated raster")
            }
            Self::InvalidRasterBytes => formatter.write_str("raster PNG bytes are invalid"),
            Self::LocalFile(error) => write!(formatter, "local PNG source is unavailable: {error}"),
            Self::Raster(error) => error.fmt(formatter),
            Self::Placement(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for PresentationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Raster(error) => Some(error),
            Self::Placement(error) => Some(error),
            Self::LocalFile(error) => Some(error),
            Self::GenerationExhausted
            | Self::RasterDimensionsMismatch
            | Self::RasterSourceMismatch
            | Self::InvalidRasterBytes => None,
        }
    }
}

impl From<latex_render_core::RenderError> for PresentationError {
    fn from(error: latex_render_core::RenderError) -> Self {
        Self::Raster(error)
    }
}

impl From<PlacementError> for PresentationError {
    fn from(error: PlacementError) -> Self {
        Self::Placement(error)
    }
}

/// Owns terminal backend, generation, and active image state.
#[derive(Debug)]
pub struct TerminalPresenter {
    backend: TerminalBackend,
    generation: u64,
    state: ImageRenderState,
}

impl TerminalPresenter {
    /// Creates a presenter for one detected terminal backend.
    pub fn new(backend: TerminalBackend) -> Self {
        Self {
            backend,
            generation: 0,
            state: ImageRenderState::default(),
        }
    }

    /// Starts one generation or selects immediate source fallback for text mode.
    pub fn begin(
        &mut self,
        image_id: NonZeroU32,
        row: u16,
        layout: ImageLayout,
    ) -> Result<Option<PresentationJob>, PresentationError> {
        let generation = self.advance_generation()?;
        if self.backend == TerminalBackend::Text {
            return Ok(None);
        }
        Ok(Some(PresentationJob {
            generation,
            backend: self.backend,
            image_id,
            row,
            layout,
        }))
    }

    /// Publishes only a completion from the current generation and backend.
    pub fn publish(
        &mut self,
        raster: RasterizedPresentation,
        source: ImageSource,
    ) -> Result<PublishOutcome, PresentationError> {
        if raster.job.generation != self.generation || raster.job.backend != self.backend {
            return Ok(PublishOutcome::Stale);
        }
        validate_source(self.backend, &source, &raster.image.bytes)?;
        let command = self.state.render(
            self.backend,
            Some(ImageDraw {
                image_id: raster.job.image_id,
                x: raster.job.layout.column,
                y: raster.job.row,
                size: raster.job.layout.placement,
                source,
            }),
        )?;
        Ok(PublishOutcome::Published(command))
    }

    /// Invalidates pending work and returns cleanup bytes for source fallback.
    pub fn fallback(&mut self) -> Result<Vec<u8>, PresentationError> {
        self.advance_generation()?;
        Ok(self.state.clear())
    }

    /// Changes transport while invalidating work and clearing the prior image.
    pub fn set_backend(&mut self, backend: TerminalBackend) -> Result<Vec<u8>, PresentationError> {
        if backend == self.backend {
            return Ok(Vec::new());
        }
        self.advance_generation()?;
        self.backend = backend;
        Ok(self.state.clear())
    }

    /// Reports whether this presenter owns an active terminal placement.
    pub fn has_active_image(&self) -> bool {
        self.state.has_active_image()
    }

    fn advance_generation(&mut self) -> Result<u64, PresentationError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(PresentationError::GenerationExhausted)?;
        Ok(self.generation)
    }
}

fn validate_source(
    backend: TerminalBackend,
    source: &ImageSource,
    expected: &[u8],
) -> Result<(), PresentationError> {
    if !matches!(
        (backend, source),
        (TerminalBackend::KittyDirect, ImageSource::PngBytes(_))
            | (TerminalBackend::KittyLocalFile, ImageSource::LocalPng(_))
    ) {
        return Err(PlacementError::SourceMismatch { backend }.into());
    }
    let matches = match source {
        ImageSource::PngBytes(bytes) => bytes == expected,
        ImageSource::LocalPng(path) => {
            let file = File::open(path).map_err(PresentationError::LocalFile)?;
            let mut bytes = Vec::with_capacity(expected.len());
            file.take(expected.len() as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(PresentationError::LocalFile)?;
            bytes == expected
        }
    };
    if matches {
        Ok(())
    } else {
        Err(PresentationError::RasterSourceMismatch)
    }
}

/// Runs deterministic fitted rasterization for an immutable presentation job.
pub fn rasterize_presentation(
    svg: &SanitizedSvg,
    job: PresentationJob,
    limits: RasterLimits,
) -> Result<RasterizedPresentation, PresentationError> {
    let image = rasterize_svg_fitted(svg, job.raster_request(), limits)?;
    RasterizedPresentation::new(job, image)
}
