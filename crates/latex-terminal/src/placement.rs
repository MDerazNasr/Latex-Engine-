use std::fmt;
use std::num::NonZeroU32;
use std::path::PathBuf;

use crate::PlacementSize;
use crate::ProtocolError;
use crate::TerminalBackend;
use crate::protocol::kitty_delete_image;
use crate::protocol::kitty_transmit_png;
use crate::protocol::kitty_transmit_png_file;

const ESC: &str = "\x1b";

/// PNG source accepted by one terminal transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSource {
    /// PNG bytes for a direct protocol transfer.
    PngBytes(Vec<u8>),
    /// Local PNG path for a local file transfer.
    LocalPng(PathBuf),
}

/// One positioned terminal image draw request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageDraw {
    /// Stable image identifier used for replacement and deletion.
    pub image_id: NonZeroU32,
    /// Zero based terminal column.
    pub x: u16,
    /// Zero based terminal row.
    pub y: u16,
    /// Terminal cell rectangle reserved by layout.
    pub size: PlacementSize,
    /// PNG data source selected for the transport.
    pub source: ImageSource,
}

/// Failure while preparing a positioned image command.
#[derive(Debug)]
pub enum PlacementError {
    /// The source did not match the selected terminal transport.
    SourceMismatch {
        /// Backend that rejected the source.
        backend: TerminalBackend,
    },
    /// The terminal protocol encoder rejected the source.
    Protocol(ProtocolError),
}

impl fmt::Display for PlacementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceMismatch { backend } => {
                write!(formatter, "image source does not match {backend:?}")
            }
            Self::Protocol(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for PlacementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SourceMismatch { .. } => None,
            Self::Protocol(error) => Some(error),
        }
    }
}

impl From<ProtocolError> for PlacementError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveImage {
    backend: TerminalBackend,
    draw: ImageDraw,
}

/// Tracks placement state so redraw and cleanup remain deterministic.
#[derive(Debug, Default)]
pub struct ImageRenderState {
    active: Option<ActiveImage>,
}

impl ImageRenderState {
    /// Encodes the minimal transition to a new image or a cleared state.
    pub fn render(
        &mut self,
        backend: TerminalBackend,
        draw: Option<ImageDraw>,
    ) -> Result<Vec<u8>, PlacementError> {
        if backend == TerminalBackend::Text || draw.is_none() {
            return Ok(self.clear());
        }

        let draw = draw.expect("draw was checked above");
        let next = ActiveImage { backend, draw };
        if self.active.as_ref() == Some(&next) {
            return Ok(Vec::new());
        }

        let payload = encode_payload(&next)?;
        let mut command = String::new();
        if let Some(active) = &self.active {
            command.push_str(&kitty_delete_image(active.draw.image_id));
        }
        command.push_str(ESC);
        command.push('7');
        command.push_str(&move_to(next.draw.x, next.draw.y));
        command.push_str(&payload);
        command.push_str(ESC);
        command.push('8');
        self.active = Some(next);
        Ok(command.into_bytes())
    }

    /// Deletes the active image and forgets its placement.
    pub fn clear(&mut self) -> Vec<u8> {
        self.active
            .take()
            .map(|active| kitty_delete_image(active.draw.image_id).into_bytes())
            .unwrap_or_default()
    }

    /// Reports whether an image currently owns a terminal placement.
    pub fn has_active_image(&self) -> bool {
        self.active.is_some()
    }
}

fn encode_payload(active: &ActiveImage) -> Result<String, PlacementError> {
    match (active.backend, &active.draw.source) {
        (TerminalBackend::KittyDirect, ImageSource::PngBytes(png)) => Ok(kitty_transmit_png(
            png,
            active.draw.size,
            active.draw.image_id,
        )?),
        (TerminalBackend::KittyLocalFile, ImageSource::LocalPng(path)) => Ok(
            kitty_transmit_png_file(path, active.draw.size, active.draw.image_id)?,
        ),
        (TerminalBackend::Text, _) => unreachable!("text rendering is handled before encoding"),
        (backend, _) => Err(PlacementError::SourceMismatch { backend }),
    }
}

fn move_to(x: u16, y: u16) -> String {
    format!("{ESC}[{};{}H", u32::from(y) + 1, u32::from(x) + 1)
}
