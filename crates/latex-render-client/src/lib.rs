#![doc = "Supervised asynchronous client for the local MathJax worker."]

mod client;
mod config;
mod health;
mod line_reader;
mod process;
mod protocol;
mod supervisor;

#[cfg(test)]
mod line_reader_tests;
#[cfg(test)]
mod protocol_tests;

pub use client::WorkerClient;
pub use config::{WorkerClientConfig, WorkerCommand};
pub use health::{WorkerHealth, WorkerState};

/// The supported worker protocol version.
pub const WORKER_PROTOCOL_VERSION: u32 = 1;
