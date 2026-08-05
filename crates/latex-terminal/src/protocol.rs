use std::fmt;
use std::fs;
use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose;

const ESC: &str = "\x1b";
const ST: &str = "\x1b\\";
const KITTY_CHUNK_SIZE: usize = 4096;

/// Terminal cell rectangle reserved for one image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementSize {
    /// Number of terminal columns.
    pub columns: NonZeroU16,
    /// Number of terminal rows.
    pub rows: NonZeroU16,
}

impl PlacementSize {
    /// Creates a nonempty terminal cell rectangle.
    pub const fn new(columns: NonZeroU16, rows: NonZeroU16) -> Self {
        Self { columns, rows }
    }
}

/// Failure while constructing a terminal image command.
#[derive(Debug)]
pub enum ProtocolError {
    /// A PNG payload was empty.
    EmptyPng,
    /// A local image path could not be resolved.
    File(std::io::Error),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPng => formatter.write_str("PNG payload is empty"),
            Self::File(error) => write!(formatter, "local PNG path is unavailable: {error}"),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmptyPng => None,
            Self::File(error) => Some(error),
        }
    }
}

/// Deletes one Kitty image and its stored data without a terminal reply.
pub fn kitty_delete_image(image_id: NonZeroU32) -> String {
    format!("{ESC}_Ga=d,d=I,i={image_id},q=2;{ST}")
}

/// Transmits PNG bytes and places the image in a terminal cell rectangle.
pub fn kitty_transmit_png(
    png: &[u8],
    size: PlacementSize,
    image_id: NonZeroU32,
) -> Result<String, ProtocolError> {
    if png.is_empty() {
        return Err(ProtocolError::EmptyPng);
    }

    let payload = general_purpose::STANDARD.encode(png);
    let chunks = payload.as_bytes().chunks(KITTY_CHUNK_SIZE);
    let chunk_count = chunks.len();
    let mut command = String::new();
    for (index, chunk) in chunks.enumerate() {
        let chunk = std::str::from_utf8(chunk).expect("base64 output is valid UTF 8");
        let more = u8::from(index + 1 < chunk_count);
        if index == 0 {
            command.push_str(&format!(
                "{ESC}_Ga=T,t=d,f=100,c={},r={},q=2,i={image_id},C=1,m={more};{chunk}{ST}",
                size.columns, size.rows,
            ));
        } else {
            command.push_str(&format!("{ESC}_Gm={more};{chunk}{ST}"));
        }
    }
    Ok(command)
}

/// Places a local PNG through the Kitty file transmission medium.
pub fn kitty_transmit_png_file(
    path: &Path,
    size: PlacementSize,
    image_id: NonZeroU32,
) -> Result<String, ProtocolError> {
    let path = fs::canonicalize(path).map_err(ProtocolError::File)?;
    let payload = general_purpose::STANDARD.encode(path.to_string_lossy().as_bytes());
    Ok(format!(
        "{ESC}_Ga=T,t=f,f=100,c={},r={},q=2,i={image_id},C=1;{payload}{ST}",
        size.columns, size.rows,
    ))
}
