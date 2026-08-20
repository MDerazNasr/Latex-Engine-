//! Bounded serial process loop for daemon protocol version 1.

use latex_render_client::WorkerClient;
use latex_render_client::WorkerClientConfig;
use latex_render_client::WorkerCommand;
use latex_render_core::MathRenderer;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

use crate::args::WorkerOptions;
use crate::daemon_protocol_v1::DaemonResponseV1;
use crate::daemon_protocol_v1::MAX_DAEMON_REQUEST_LINE_BYTES;
use crate::daemon_protocol_v1::MAX_DAEMON_RESPONSE_LINE_BYTES;
use crate::daemon_protocol_v1::decode_request_v1;
use crate::daemon_protocol_v1::error;
use crate::daemon_renderer_v1::render_message_v1;
use crate::error::CliError;
use crate::error::CliErrorKind;
use crate::worker_path::resolve_worker;

pub(crate) async fn run_daemon_v1(options: &WorkerOptions) -> Result<(), CliError> {
    let worker = resolve_worker(options)?;
    let command = WorkerCommand::new(&options.node).arg(worker);
    let mut client =
        WorkerClient::start(WorkerClientConfig::new(command)).map_err(CliError::from_render)?;
    let serve_result = {
        // Asynchronous standard input keeps worker supervision live while the daemon is idle.
        let mut reader = BufReader::new(tokio::io::stdin());
        let mut writer = tokio::io::stdout();
        serve_daemon_v1(&client, &mut reader, &mut writer).await
    };
    let shutdown_result = client.shutdown().await.map_err(CliError::from_render);
    match (serve_result, shutdown_result) {
        (Err(serve_error), _) => Err(serve_error),
        (Ok(()), result) => result,
    }
}

async fn serve_daemon_v1(
    renderer: &(impl MathRenderer + ?Sized),
    reader: &mut (impl AsyncBufRead + Unpin),
    writer: &mut (impl AsyncWrite + Unpin),
) -> Result<(), CliError> {
    loop {
        let line = match read_bounded_line_v1(reader).await.map_err(input_error)? {
            LineReadV1::Eof => return Ok(()),
            LineReadV1::TooLong => {
                write_response_v1(
                    writer,
                    DaemonResponseV1::error(None, error("input_limit_exceeded", false)),
                    None,
                )
                .await?;
                continue;
            }
            LineReadV1::Line(line) => line,
        };

        match decode_request_v1(&line) {
            Ok(request) => {
                let id = request.id.clone();
                let response = match render_message_v1(renderer, request).await {
                    Ok(equations) => DaemonResponseV1::success(id.clone(), equations),
                    Err(render_error) => DaemonResponseV1::error(Some(id.clone()), render_error),
                };
                write_response_v1(writer, response, Some(id)).await?;
            }
            Err(decode_error) => {
                let id = decode_error.id;
                write_response_v1(
                    writer,
                    DaemonResponseV1::error(id.clone(), decode_error.error),
                    id,
                )
                .await?;
            }
        }
    }
}

async fn write_response_v1(
    writer: &mut (impl AsyncWrite + Unpin),
    response: DaemonResponseV1,
    id: Option<String>,
) -> Result<(), CliError> {
    let mut json = serde_json::to_vec(&response).map_err(|_| {
        CliError::new(
            CliErrorKind::Internal,
            "Daemon response could not be serialized",
        )
    })?;
    if json.len() > MAX_DAEMON_RESPONSE_LINE_BYTES {
        json = serde_json::to_vec(&DaemonResponseV1::error(
            id,
            error("output_limit_exceeded", false),
        ))
        .map_err(|_| {
            CliError::new(
                CliErrorKind::Internal,
                "Daemon limit response could not be serialized",
            )
        })?;
    }
    if json.len() > MAX_DAEMON_RESPONSE_LINE_BYTES {
        return Err(CliError::new(
            CliErrorKind::Internal,
            "Daemon limit response exceeded its byte limit",
        ));
    }
    json.push(b'\n');
    writer.write_all(&json).await.map_err(output_error)?;
    writer.flush().await.map_err(output_error)
}

async fn read_bounded_line_v1(
    reader: &mut (impl AsyncBufRead + Unpin),
) -> std::io::Result<LineReadV1> {
    let mut line = Vec::new();
    let mut too_long = false;
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            return if too_long {
                Ok(LineReadV1::TooLong)
            } else if line.is_empty() {
                Ok(LineReadV1::Eof)
            } else {
                Ok(LineReadV1::Line(line))
            };
        }

        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let content_len = newline.unwrap_or(buffer.len());
        if !too_long {
            if line.len().saturating_add(content_len) > MAX_DAEMON_REQUEST_LINE_BYTES {
                too_long = true;
            } else {
                line.extend_from_slice(&buffer[..content_len]);
            }
        }
        let consumed = newline.map_or(buffer.len(), |position| position + 1);
        reader.consume(consumed);
        if newline.is_some() {
            return if too_long {
                Ok(LineReadV1::TooLong)
            } else {
                Ok(LineReadV1::Line(line))
            };
        }
    }
}

fn input_error(_: std::io::Error) -> CliError {
    CliError::new(CliErrorKind::Internal, "Daemon input could not be read")
}

fn output_error(_: std::io::Error) -> CliError {
    CliError::new(CliErrorKind::Output, "Daemon response could not be written")
}

enum LineReadV1 {
    Eof,
    Line(Vec<u8>),
    TooLong,
}

#[cfg(test)]
#[path = "daemon_v1_tests.rs"]
mod tests;
