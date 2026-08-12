#![doc = "Terminal cell layout integration tests."]

use latex_terminal::{
    LayoutError, LayoutMode, LayoutPolicy, MathGeometry, TerminalGeometry, layout_math,
};

fn terminal() -> TerminalGeometry {
    TerminalGeometry::new(100, 40, 1000, 800).expect("terminal should be valid")
}

#[test]
fn terminal_geometry_requires_nonzero_measured_cell_pixels() {
    let cases = [
        TerminalGeometry::new(0, 40, 1000, 800),
        TerminalGeometry::new(100, 0, 1000, 800),
        TerminalGeometry::new(100, 40, 0, 800),
        TerminalGeometry::new(100, 40, 99, 800),
        TerminalGeometry::new(100, 40, 1000, 39),
    ];
    for result in cases {
        assert_eq!(result, Err(LayoutError::InvalidTerminalGeometry));
    }
}

#[test]
fn math_geometry_rejects_null_and_invalid_baselines() {
    for result in [
        MathGeometry::new(0, 1, None, false),
        MathGeometry::new(1, 0, None, false),
        MathGeometry::new(10, 10, Some(f32::NAN), false),
        MathGeometry::new(10, 10, Some(-1.0), false),
        MathGeometry::new(10, 10, Some(11.0), false),
    ] {
        assert_eq!(result, Err(LayoutError::InvalidMathGeometry));
    }
}

#[test]
fn policy_rejects_zero_and_impossible_percentages() {
    for result in [
        LayoutPolicy::new(0, 18, 80),
        LayoutPolicy::new(101, 18, 80),
        LayoutPolicy::new(90, 0, 80),
        LayoutPolicy::new(90, 18, 0),
        LayoutPolicy::new(90, 18, 100),
    ] {
        assert_eq!(result, Err(LayoutError::InvalidPolicy));
    }
}

#[test]
fn short_inline_math_uses_one_row_and_the_current_column() {
    let math = MathGeometry::new(37, 31, Some(30.0), false).expect("math should be valid");
    let layout = layout_math(terminal(), math, 12, LayoutPolicy::default());

    assert_eq!(layout.mode, LayoutMode::Inline);
    assert_eq!(layout.column, 12);
    assert_eq!(layout.placement.rows.get(), 1);
    assert_eq!(layout.canvas_height_px, 20);
    assert!(layout.content.width <= layout.canvas_width_px);
    assert!(layout.content.height <= layout.canvas_height_px);
    assert!(layout.baseline_px.expect("baseline should exist") <= 20.0);
}

#[test]
fn inline_without_a_baseline_is_centered_in_one_row() {
    let math = MathGeometry::new(20, 10, None, false).expect("math should be valid");
    let layout = layout_math(terminal(), math, 0, LayoutPolicy::default());

    assert_eq!(layout.mode, LayoutMode::Inline);
    assert_eq!(layout.content.y, 5);
    assert_eq!(layout.baseline_px, None);
}

#[test]
fn wide_inline_math_is_promoted_to_a_centered_block() {
    let math = MathGeometry::new(1000, 20, Some(16.0), false).expect("math should be valid");
    let layout = layout_math(terminal(), math, 90, LayoutPolicy::default());

    assert_eq!(layout.mode, LayoutMode::Block);
    assert_eq!(layout.placement.columns.get(), 90);
    assert_eq!(layout.column, 5);
    assert!(layout.placement.rows.get() <= 18);
}

#[test]
fn display_math_is_capped_centered_and_never_upscaled() {
    let math = MathGeometry::new(2000, 1000, Some(600.0), true).expect("math should be valid");
    let policy = LayoutPolicy::new(80, 10, 80).expect("policy should be valid");
    let layout = layout_math(terminal(), math, 0, policy);

    assert_eq!(layout.mode, LayoutMode::Block);
    assert!(layout.placement.columns.get() <= 80);
    assert!(layout.placement.rows.get() <= 10);
    assert_eq!(layout.column, (100 - layout.placement.columns.get()) / 2);
    assert!(layout.scale <= 1.0);
    assert_eq!(
        layout.canvas_width_px,
        u32::from(layout.placement.columns.get()) * terminal().cell_width_px()
    );
}

#[test]
fn terminal_resize_recomputes_cell_and_pixel_layout() {
    let math = MathGeometry::new(800, 200, Some(150.0), true).expect("math should be valid");
    let wide = layout_math(terminal(), math, 0, LayoutPolicy::default());
    let narrow_terminal =
        TerminalGeometry::new(40, 40, 400, 800).expect("terminal should be valid");
    let narrow = layout_math(narrow_terminal, math, 0, LayoutPolicy::default());

    assert!(narrow.canvas_width_px < wide.canvas_width_px);
    assert!(narrow.scale < wide.scale);
    assert!(narrow.placement.columns.get() <= 36);
}
