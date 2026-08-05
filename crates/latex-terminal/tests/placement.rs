//! Stateful terminal placement tests.

use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::PathBuf;

use latex_terminal::ImageDraw;
use latex_terminal::ImageRenderState;
use latex_terminal::ImageSource;
use latex_terminal::PlacementError;
use latex_terminal::PlacementSize;
use latex_terminal::TerminalBackend;

#[test]
fn initial_draw_preserves_cursor_and_tracks_placement() {
    let mut state = ImageRenderState::default();

    let command = state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
        .unwrap();
    let command = String::from_utf8(command).unwrap();

    assert!(command.starts_with("\x1b7\x1b[3;4H\x1b_Ga=T"));
    assert!(command.ends_with("\x1b8"));
    assert!(state.has_active_image());
}

#[test]
fn identical_redraw_emits_nothing() {
    let mut state = ImageRenderState::default();
    state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
        .unwrap();

    assert_eq!(
        state
            .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
            .unwrap(),
        Vec::<u8>::new(),
    );
}

#[test]
fn resized_draw_deletes_before_replacing() {
    let mut state = ImageRenderState::default();
    state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
        .unwrap();

    let command = state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(30, 6)))
        .unwrap();
    let command = String::from_utf8(command).unwrap();

    assert!(command.starts_with("\x1b_Ga=d,d=I,i=42,q=2;\x1b\\"));
    assert!(command.contains("c=30,r=6"));
}

#[test]
fn clearing_or_selecting_text_deletes_the_active_image() {
    let mut state = ImageRenderState::default();
    state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
        .unwrap();

    assert_eq!(
        state.render(TerminalBackend::Text, None).unwrap(),
        b"\x1b_Ga=d,d=I,i=42,q=2;\x1b\\".to_vec(),
    );
    assert!(!state.has_active_image());
    assert_eq!(state.clear(), Vec::<u8>::new());
}

#[test]
fn a_transport_source_mismatch_keeps_existing_state() {
    let mut state = ImageRenderState::default();
    state
        .render(TerminalBackend::KittyDirect, Some(direct_draw(20, 4)))
        .unwrap();
    let invalid = ImageDraw {
        source: ImageSource::LocalPng(PathBuf::from("formula.png")),
        ..direct_draw(20, 4)
    };

    assert!(matches!(
        state.render(TerminalBackend::KittyDirect, Some(invalid)),
        Err(PlacementError::SourceMismatch {
            backend: TerminalBackend::KittyDirect,
        }),
    ));
    assert!(state.has_active_image());
}

fn direct_draw(columns: u16, rows: u16) -> ImageDraw {
    ImageDraw {
        image_id: NonZeroU32::new(42).unwrap(),
        x: 3,
        y: 2,
        size: PlacementSize::new(
            NonZeroU16::new(columns).unwrap(),
            NonZeroU16::new(rows).unwrap(),
        ),
        source: ImageSource::PngBytes(vec![137, 80, 78, 71]),
    }
}
