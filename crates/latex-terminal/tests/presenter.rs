#![doc = "Generation checked terminal presentation integration tests."]

use std::num::NonZeroU32;

use latex_render_core::RenderErrorCode;
use latex_render_svg::{PngImage, RasterLimits, SvgSanitizerLimits, sanitize_svg};
use latex_terminal::{
    ImageSource, LayoutPolicy, MathGeometry, PresentationError, PublishOutcome,
    RasterizedPresentation, TerminalBackend, TerminalGeometry, TerminalPresenter, layout_math,
    rasterize_presentation,
};

const RECTANGLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50" role="img" focusable="false" style="color:#000000"><rect x="0" y="0" width="100" height="50" fill="currentColor"/></svg>"##;
const IMAGE_ID: NonZeroU32 = NonZeroU32::new(71).expect("image identifier is nonzero");

#[test]
fn job_rasterizes_and_publishes_the_exact_reserved_canvas() {
    let layout = display_layout();
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let job = presenter
        .begin(IMAGE_ID, 5, layout)
        .expect("generation should advance")
        .expect("image backend should issue a job");

    assert_eq!(job.raster_request().canvas_width_px, layout.canvas_width_px);
    assert_eq!(
        job.raster_request().canvas_height_px,
        layout.canvas_height_px
    );
    assert_eq!(job.raster_request().content.x_px, layout.content.x);
    assert_eq!(job.raster_request().content.y_px, layout.content.y);

    let raster = rasterize_presentation(&svg, job, RasterLimits::default())
        .expect("presentation should rasterize");
    let source = raster.direct_source();
    let outcome = presenter
        .publish(raster, source)
        .expect("presentation should publish");
    let PublishOutcome::Published(command) = outcome else {
        panic!("current generation should publish");
    };
    let command = String::from_utf8(command).expect("protocol bytes should be UTF 8");

    assert!(command.contains("c=10,r=3"));
    assert!(command.contains("\x1b[6;36H"));
    assert!(presenter.has_active_image());
}

#[test]
fn completion_from_an_older_generation_is_silently_discarded() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let older = presenter
        .begin(IMAGE_ID, 1, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    let current = presenter
        .begin(IMAGE_ID, 2, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    let older = rasterize_presentation(&svg, older, RasterLimits::default())
        .expect("older job should rasterize");
    let older_source = older.direct_source();

    assert_eq!(
        presenter
            .publish(older, older_source)
            .expect("stale publication should not fail"),
        PublishOutcome::Stale
    );
    assert!(!presenter.has_active_image());

    let current = rasterize_presentation(&svg, current, RasterLimits::default())
        .expect("current job should rasterize");
    let current_source = current.direct_source();
    assert!(matches!(
        presenter
            .publish(current, current_source)
            .expect("current publication should succeed"),
        PublishOutcome::Published(_)
    ));
}

#[test]
fn fallback_clears_the_image_and_invalidates_pending_work() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let job = presenter
        .begin(IMAGE_ID, 1, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    let raster = rasterize_presentation(&svg, job.clone(), RasterLimits::default())
        .expect("job should rasterize");
    let source = raster.direct_source();
    presenter
        .publish(raster, source)
        .expect("image should publish");

    let cleanup = String::from_utf8(presenter.fallback().expect("fallback should succeed"))
        .expect("protocol bytes should be UTF 8");
    assert!(cleanup.contains("a=d,d=I,i=71"));
    assert!(!presenter.has_active_image());

    let stale = rasterize_presentation(&svg, job, RasterLimits::default())
        .expect("cancelled job may finish in the blocking pool");
    let stale_source = stale.direct_source();
    assert_eq!(
        presenter
            .publish(stale, stale_source)
            .expect("stale publication should not fail"),
        PublishOutcome::Stale
    );
}

#[test]
fn text_or_changed_backends_never_publish_an_old_job() {
    let mut text = TerminalPresenter::new(TerminalBackend::Text);
    assert!(
        text.begin(IMAGE_ID, 0, display_layout())
            .expect("generation should advance")
            .is_none()
    );

    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let job = presenter
        .begin(IMAGE_ID, 0, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    assert!(
        presenter
            .set_backend(TerminalBackend::KittyLocalFile)
            .expect("backend should change")
            .is_empty()
    );
    let raster = rasterize_presentation(&svg, job, RasterLimits::default())
        .expect("old job should still finish safely");
    let source = raster.direct_source();
    assert_eq!(
        presenter
            .publish(raster, source)
            .expect("old backend should be stale"),
        PublishOutcome::Stale
    );
}

#[test]
fn mismatched_dimensions_and_source_bytes_fail_before_placement() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let job = presenter
        .begin(IMAGE_ID, 0, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    let bad_dimensions = PngImage {
        bytes: vec![1],
        width_px: job.layout().canvas_width_px + 1,
        height_px: job.layout().canvas_height_px,
    };
    assert!(matches!(
        RasterizedPresentation::new(job.clone(), bad_dimensions),
        Err(PresentationError::RasterDimensionsMismatch)
    ));
    let invalid_bytes = PngImage {
        bytes: vec![1, 2, 3],
        width_px: job.layout().canvas_width_px,
        height_px: job.layout().canvas_height_px,
    };
    assert!(matches!(
        RasterizedPresentation::new(job.clone(), invalid_bytes),
        Err(PresentationError::InvalidRasterBytes)
    ));

    let raster =
        rasterize_presentation(&svg, job, RasterLimits::default()).expect("job should rasterize");
    assert!(matches!(
        presenter.publish(raster, ImageSource::PngBytes(vec![1, 2, 3])),
        Err(PresentationError::RasterSourceMismatch)
    ));
    assert!(!presenter.has_active_image());
}

#[test]
fn raster_limits_propagate_without_changing_placement_state() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let mut presenter = TerminalPresenter::new(TerminalBackend::KittyDirect);
    let job = presenter
        .begin(IMAGE_ID, 0, display_layout())
        .expect("generation should advance")
        .expect("job should exist");
    let limits = RasterLimits {
        max_rgba_bytes: 1,
        ..RasterLimits::default()
    };

    let error = rasterize_presentation(&svg, job, limits)
        .expect_err("allocation limit should reject the job");
    assert!(matches!(
        error,
        PresentationError::Raster(error)
            if error.code == RenderErrorCode::OutputLimitExceeded
    ));
    assert!(!presenter.has_active_image());
}

fn display_layout() -> latex_terminal::ImageLayout {
    let terminal = TerminalGeometry::new(80, 24, 800, 480).expect("terminal should be valid");
    let math = MathGeometry::new(100, 50, Some(40.0), true).expect("math should be valid");
    layout_math(terminal, math, 0, LayoutPolicy::default())
}
