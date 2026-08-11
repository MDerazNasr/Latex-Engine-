#![doc = "Canonical SVG and perceptual PNG rendering snapshot suite."]

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand, WorkerState};
use latex_render_core::{RenderErrorCode, RenderRequest, Rgba};
use latex_render_svg::{
    RasterLimits, RasterRequest, SvgSanitizerLimits, rasterize_svg, sanitize_svg,
};
use serde::{Deserialize, Serialize};
use tiny_skia::Pixmap;

const MAX_CHANNEL_DELTA: u8 = 2;
const CHANGED_PIXEL_DENOMINATOR: usize = 1_000;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusEntry {
    name: String,
    source: String,
    display_mode: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotMetadata {
    name: String,
    theme: String,
    display_mode: bool,
    width_px: u32,
    height_px: u32,
    baseline_px: Option<f32>,
}

struct Theme {
    name: &'static str,
    foreground: Rgba,
}

struct Snapshot {
    stem: String,
    svg: Vec<u8>,
    png: Vec<u8>,
    metadata: SnapshotMetadata,
}

const THEMES: &[Theme] = &[
    Theme {
        name: "dark",
        foreground: Rgba::opaque(230, 237, 243),
    },
    Theme {
        name: "light",
        foreground: Rgba::opaque(17, 24, 39),
    },
];

#[tokio::test]
#[ignore = "requires the built MathJax worker and explicit snapshot review"]
async fn rendering_corpus_matches_reviewed_snapshots() {
    let repository = repository();
    let worker = repository.join("renderer/mathjax-worker/dist/src/server.js");
    assert!(
        worker.is_file(),
        "build the MathJax worker before this test"
    );
    let corpus = load_corpus(&repository);
    assert_eq!(corpus.len(), 25);

    let config = WorkerClientConfig::new(WorkerCommand::new("node").arg(worker));
    let mut client = WorkerClient::start(config).expect("client should start");
    let mut snapshots = Vec::with_capacity(corpus.len() * THEMES.len());
    for entry in &corpus {
        assert!(safe_name(&entry.name), "corpus name must be path safe");
        for theme in THEMES {
            snapshots.push(render_snapshot(&client, entry, theme).await);
        }
    }
    let invalid = client
        .render_request(RenderRequest {
            source: r"\frac{1}{".to_owned(),
            display_mode: true,
            foreground: THEMES[0].foreground,
            background: None,
            scale: 2.0,
            max_width_px: 1200,
        })
        .await
        .expect_err("invalid TeX should remain an error");
    assert_eq!(invalid.code, RenderErrorCode::InvalidTex);
    assert_eq!(client.health().await.state, WorkerState::Ready);
    client.shutdown().await.expect("worker should stop cleanly");

    let directory = repository.join("fixtures/rendering/snapshots");
    if std::env::var_os("UPDATE_LATEX_SNAPSHOTS").as_deref() == Some(OsStr::new("1")) {
        update_snapshots(&directory, &snapshots);
    } else {
        verify_snapshots(&directory, &snapshots);
    }
}

async fn render_snapshot(client: &WorkerClient, entry: &CorpusEntry, theme: &Theme) -> Snapshot {
    let rendered = client
        .render_request(RenderRequest {
            source: entry.source.clone(),
            display_mode: entry.display_mode,
            foreground: theme.foreground,
            background: None,
            scale: 2.0,
            max_width_px: 1200,
        })
        .await
        .unwrap_or_else(|error| panic!("snapshot {} failed: {error:?}", entry.name));
    assert!(!contains(&rendered.svg, b"data-latex"));
    let sanitized = sanitize_svg(&rendered.svg, SvgSanitizerLimits::default())
        .expect("client SVG should remain sanitized");
    let request = RasterRequest {
        width_px: rendered.width_px,
        height_px: rendered.height_px,
    };
    let image = tokio::task::spawn_blocking(move || {
        rasterize_svg(&sanitized, request, RasterLimits::default())
    })
    .await
    .expect("raster task should join")
    .expect("snapshot should rasterize");
    Snapshot {
        stem: format!("{}-{}", entry.name, theme.name),
        svg: rendered.svg,
        png: image.bytes,
        metadata: SnapshotMetadata {
            name: entry.name.clone(),
            theme: theme.name.to_owned(),
            display_mode: entry.display_mode,
            width_px: rendered.width_px,
            height_px: rendered.height_px,
            baseline_px: rendered.baseline_px,
        },
    }
}

fn update_snapshots(directory: &Path, snapshots: &[Snapshot]) {
    fs::create_dir_all(directory).expect("snapshot directory should be created");
    for snapshot in snapshots {
        fs::write(
            directory.join(format!("{}.svg", snapshot.stem)),
            &snapshot.svg,
        )
        .expect("SVG snapshot should be written");
        fs::write(
            directory.join(format!("{}.png", snapshot.stem)),
            &snapshot.png,
        )
        .expect("PNG snapshot should be written");
    }
    fs::write(directory.join("manifest.json"), manifest(snapshots))
        .expect("snapshot manifest should be written");
    assert_expected_files(directory, snapshots);
}

fn verify_snapshots(directory: &Path, snapshots: &[Snapshot]) {
    assert_expected_files(directory, snapshots);
    for snapshot in snapshots {
        let expected_svg = fs::read(directory.join(format!("{}.svg", snapshot.stem)))
            .expect("SVG snapshot is missing; review an explicit snapshot update");
        assert_eq!(
            snapshot.svg, expected_svg,
            "SVG changed for {}",
            snapshot.stem
        );
        let expected_png = fs::read(directory.join(format!("{}.png", snapshot.stem)))
            .expect("PNG snapshot is missing; review an explicit snapshot update");
        assert_perceptual_match(&snapshot.stem, &snapshot.png, &expected_png);
    }
    let expected_manifest =
        fs::read(directory.join("manifest.json")).expect("snapshot manifest should exist");
    assert_eq!(manifest(snapshots), expected_manifest, "metadata changed");
}

fn assert_perceptual_match(name: &str, actual: &[u8], expected: &[u8]) {
    let actual = Pixmap::decode_png(actual).expect("actual PNG should decode");
    let expected = Pixmap::decode_png(expected).expect("snapshot PNG should decode");
    assert_eq!(actual.width(), expected.width(), "width changed for {name}");
    assert_eq!(
        actual.height(),
        expected.height(),
        "height changed for {name}"
    );
    let mut changed_pixels = 0usize;
    let mut maximum_delta = 0u8;
    for (actual, expected) in actual
        .data()
        .chunks_exact(4)
        .zip(expected.data().chunks_exact(4))
    {
        let mut changed = false;
        for (actual, expected) in actual.iter().zip(expected) {
            let delta = actual.abs_diff(*expected);
            maximum_delta = maximum_delta.max(delta);
            changed |= delta != 0;
        }
        changed_pixels += usize::from(changed);
    }
    let pixel_count = actual.width() as usize * actual.height() as usize;
    let allowed_pixels = pixel_count / CHANGED_PIXEL_DENOMINATOR;
    assert!(
        maximum_delta <= MAX_CHANNEL_DELTA && changed_pixels <= allowed_pixels,
        "PNG changed for {name}: {changed_pixels} pixels and channel delta {maximum_delta}"
    );
}

fn assert_expected_files(directory: &Path, snapshots: &[Snapshot]) {
    let mut expected = BTreeSet::from(["manifest.json".to_owned()]);
    for snapshot in snapshots {
        expected.insert(format!("{}.svg", snapshot.stem));
        expected.insert(format!("{}.png", snapshot.stem));
    }
    let actual = fs::read_dir(directory)
        .expect("snapshot directory should exist")
        .map(|entry| {
            entry
                .expect("snapshot entry should be readable")
                .file_name()
                .into_string()
                .expect("snapshot name should contain valid UTF 8")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "snapshot file set changed");
}

fn manifest(snapshots: &[Snapshot]) -> Vec<u8> {
    let metadata = snapshots
        .iter()
        .map(|snapshot| &snapshot.metadata)
        .collect::<Vec<_>>();
    let mut json = serde_json::to_string_pretty(&metadata).expect("metadata should serialize");
    json.push('\n');
    json.into_bytes()
}

fn load_corpus(repository: &Path) -> Vec<CorpusEntry> {
    let path = repository.join("fixtures/rendering/math-corpus.json");
    serde_json::from_str(&fs::read_to_string(path).expect("corpus should be readable"))
        .expect("corpus should contain valid JSON")
}

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("client crate should be inside the repository")
        .to_owned()
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|candidate| candidate == needle)
}
