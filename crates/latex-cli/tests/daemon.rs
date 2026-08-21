#![doc = "Process integration tests for the Codex renderer daemon."]

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use base64::Engine as _;
use base64::engine::general_purpose;
use serde_json::Value;

static NEXT_MARKER: AtomicU64 = AtomicU64::new(1);

#[test]
fn daemon_recovers_renders_png_and_reaps_worker_on_eof() {
    let marker = temporary_marker();
    let mut daemon = DaemonProcess::start(&daemon_worker(), Some(&marker));

    daemon.send("not-json");
    let malformed = daemon.response();
    assert_eq!(malformed["protocol"], 1);
    assert_eq!(malformed["id"], Value::Null);
    assert_eq!(malformed["ok"], false);
    assert_eq!(malformed["error"]["code"], "invalid_request");

    daemon.send(&request("message-42", "Use \\(x^2\\)."));
    let rendered = daemon.response();
    assert_eq!(rendered["id"], "message-42");
    assert_eq!(rendered["ok"], true);
    let equation = &rendered["result"]["equations"][0];
    assert_eq!(equation["startByte"], 4);
    assert_eq!(equation["endByte"], 11);
    assert_eq!(equation["displayMode"], false);
    assert_eq!(equation["status"], "rendered");
    assert_eq!(equation["widthPx"], 64);
    assert_eq!(equation["heightPx"], 32);
    assert!(equation.get("source").is_none());
    let png = general_purpose::STANDARD
        .decode(equation["pngBase64"].as_str().expect("PNG should be text"))
        .expect("PNG should be valid base64");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));

    let (status, stderr) = daemon.finish();
    assert!(status.success(), "{}", String::from_utf8_lossy(&stderr));
    assert!(stderr.is_empty());
    assert_eq!(
        fs::read_to_string(&marker).expect("worker exit marker should exist"),
        "stopped\n"
    );
    fs::remove_file(marker).expect("worker exit marker should be removed");
}

#[test]
#[ignore = "requires pnpm install and build in renderer/mathjax-worker"]
fn built_mathjax_worker_renders_through_daemon_process() {
    let worker = repository_root().join("renderer/mathjax-worker/dist/src/server.js");
    assert!(
        worker.is_file(),
        "build the MathJax worker before this test"
    );
    let mut daemon = DaemonProcess::start(&worker, None);

    daemon.send(&request("real-mathjax", "Display \\[\\frac{1}{2}\\]."));
    let response = daemon.response();
    assert_eq!(response["id"], "real-mathjax");
    assert_eq!(response["ok"], true);
    let equation = &response["result"]["equations"][0];
    assert_eq!(equation["displayMode"], true);
    assert_eq!(equation["status"], "rendered");
    let png = general_purpose::STANDARD
        .decode(equation["pngBase64"].as_str().expect("PNG should be text"))
        .expect("PNG should be valid base64");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));

    let (status, stderr) = daemon.finish();
    assert!(status.success(), "{}", String::from_utf8_lossy(&stderr));
    assert!(stderr.is_empty());
}

fn request(id: &str, source: &str) -> String {
    serde_json::json!({
        "protocol": 1,
        "id": id,
        "method": "render_message",
        "params": {
            "source": source,
            "inlineDollars": "smart",
            "foreground": "#e6edf3",
            "background": "transparent",
            "scale": 2,
            "maxWidthPx": 1200
        }
    })
    .to_string()
}

struct DaemonProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: mpsc::Receiver<std::io::Result<String>>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<Vec<u8>>>,
    finished: bool,
}

impl DaemonProcess {
    fn start(worker: &Path, exit_marker: Option<&Path>) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_latex-render"));
        command
            .args(["daemon", "--worker"])
            .arg(worker)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(marker) = exit_marker {
            command.env("LATEX_DAEMON_EXIT_MARKER", marker);
        }
        let mut child = command.spawn().expect("daemon should start");
        let stdin = child.stdin.take().expect("daemon stdin should be piped");
        let stdout = child.stdout.take().expect("daemon stdout should be piped");
        let stderr = child.stderr.take().expect("daemon stderr should be piped");
        let (response_sender, responses) = mpsc::channel();
        let stdout_thread = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if response_sender.send(line).is_err() {
                    break;
                }
            }
        });
        let stderr_thread = thread::spawn(move || {
            let mut bytes = Vec::new();
            BufReader::new(stderr)
                .read_to_end(&mut bytes)
                .expect("daemon stderr should be readable");
            bytes
        });
        Self {
            child,
            stdin: Some(stdin),
            responses,
            stdout_thread: Some(stdout_thread),
            stderr_thread: Some(stderr_thread),
            finished: false,
        }
    }

    fn send(&mut self, line: &str) {
        let stdin = self.stdin.as_mut().expect("daemon stdin should be open");
        stdin
            .write_all(line.as_bytes())
            .and_then(|()| stdin.write_all(b"\n"))
            .and_then(|()| stdin.flush())
            .expect("daemon request should be written");
    }

    fn response(&self) -> Value {
        let line = self
            .responses
            .recv_timeout(Duration::from_secs(15))
            .expect("daemon should respond before the deadline")
            .expect("daemon response should be readable");
        serde_json::from_str(&line).expect("daemon response should be valid JSON")
    }

    fn finish(&mut self) -> (ExitStatus, Vec<u8>) {
        self.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = self.child.try_wait().expect("daemon wait should succeed") {
                break status;
            }
            if Instant::now() >= deadline {
                self.child
                    .kill()
                    .expect("timed out daemon should be killed");
                self.child.wait().expect("killed daemon should be reaped");
                panic!("daemon should exit after input closes");
            }
            thread::sleep(Duration::from_millis(10));
        };
        self.stdout_thread
            .take()
            .expect("stdout reader should exist")
            .join()
            .expect("stdout reader should join");
        let stderr = self
            .stderr_thread
            .take()
            .expect("stderr reader should exist")
            .join()
            .expect("stderr reader should join");
        self.finished = true;
        (status, stderr)
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        if !self.finished {
            self.stdin.take();
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn daemon_worker() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/daemon-worker-v1.mjs")
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("CLI crate should be inside the repository")
        .to_owned()
}

fn temporary_marker() -> PathBuf {
    let id = NEXT_MARKER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "latex-daemon-worker-{}-{id}.marker",
        std::process::id()
    ))
}
