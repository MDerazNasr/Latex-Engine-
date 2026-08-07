//! Bounded newline framed input for the worker protocol.

use std::io::{Error, ErrorKind, Result};

use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub(crate) async fn read_bounded_line<R>(
    reader: &mut R,
    output: &mut Vec<u8>,
    max_bytes: usize,
) -> Result<()>
where
    R: AsyncBufRead + Unpin,
{
    output.clear();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "worker closed stdout before a complete line",
            ));
        }

        if let Some(newline) = available.iter().position(|byte| *byte == b'\n') {
            if output.len().saturating_add(newline) > max_bytes {
                return Err(too_long(max_bytes));
            }
            output.extend_from_slice(&available[..newline]);
            reader.consume(newline + 1);
            if output.last() == Some(&b'\r') {
                output.pop();
            }
            return Ok(());
        }

        if output.len().saturating_add(available.len()) > max_bytes {
            return Err(too_long(max_bytes));
        }
        let consumed = available.len();
        output.extend_from_slice(available);
        reader.consume(consumed);
    }
}

fn too_long(max_bytes: usize) -> Error {
    Error::new(
        ErrorKind::InvalidData,
        format!("worker protocol line exceeds {max_bytes} bytes"),
    )
}
