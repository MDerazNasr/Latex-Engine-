#![doc = "End to end test for the built MathJax worker."]

use std::fs;
use std::path::PathBuf;

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand, WorkerState};
use latex_render_core::{RenderRequest, Rgba};

#[tokio::test]
#[ignore = "requires pnpm build in renderer/mathjax-worker"]
async fn built_mathjax_worker_renders_through_supervised_client() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be inside the repository")
        .to_owned();
    let worker = repository
        .join("renderer")
        .join("mathjax-worker")
        .join("dist")
        .join("src")
        .join("server.js");
    assert!(
        worker.is_file(),
        "build the MathJax worker before this test"
    );

    let config = WorkerClientConfig::new(WorkerCommand::new("node").arg(worker));
    let mut client = WorkerClient::start(config).expect("client should start");
    let corpus_path = repository
        .join("fixtures")
        .join("rendering")
        .join("math-corpus.json");
    let corpus: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus_path).expect("corpus should be readable"))
            .expect("corpus should be valid JSON");
    let entries = corpus.as_array().expect("corpus should be an array");
    assert_eq!(entries.len(), 25);

    for entry in entries {
        let name = entry["name"].as_str().expect("name should be text");
        let source = entry["source"].as_str().expect("source should be text");
        let display_mode = entry["displayMode"]
            .as_bool()
            .expect("display mode should be a boolean");
        let rendered = client
            .render_request(RenderRequest {
                source: source.to_owned(),
                display_mode,
                foreground: Rgba::opaque(230, 237, 243),
                background: None,
                scale: 2.0,
                max_width_px: 1200,
            })
            .await
            .unwrap_or_else(|error| panic!("MathJax corpus entry {name} failed: {error:?}"));

        assert!(rendered.svg.starts_with(b"<svg "));
        assert!(rendered.svg.ends_with(b"</svg>"));
        assert!(
            !rendered
                .svg
                .windows(b"data-latex".len())
                .any(|bytes| bytes == b"data-latex")
        );
        assert!(rendered.width_px > 0);
        assert!(rendered.height_px > 0);
    }
    assert_eq!(client.health().await.state, WorkerState::Ready);
    client.shutdown().await.expect("shutdown should succeed");
}
