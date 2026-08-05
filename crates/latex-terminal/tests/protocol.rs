//! Kitty protocol encoding tests.

use std::num::NonZeroU16;
use std::num::NonZeroU32;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose;
use latex_terminal::PlacementSize;
use latex_terminal::ProtocolError;
use latex_terminal::kitty_delete_image;
use latex_terminal::kitty_transmit_png;
use latex_terminal::kitty_transmit_png_file;

#[test]
fn direct_png_transfer_is_chunked_and_silent() {
    let command = kitty_transmit_png(&vec![42; 4096], size(20, 4), id(91)).unwrap();

    assert_eq!(command.matches("\x1b_G").count(), 2);
    assert!(command.starts_with("\x1b_Ga=T,t=d,f=100,c=20,r=4,q=2,i=91,C=1,m=1;"));
    assert!(command.contains("\x1b\\\x1b_Gm=0;"));
    assert!(command.ends_with("\x1b\\"));
}

#[test]
fn empty_png_is_rejected() {
    assert!(matches!(
        kitty_transmit_png(&[], size(1, 1), id(1)),
        Err(ProtocolError::EmptyPng),
    ));
}

#[test]
fn local_file_transfer_encodes_a_canonical_path() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let canonical = path.canonicalize().unwrap();
    let encoded = general_purpose::STANDARD.encode(canonical.to_string_lossy().as_bytes());

    assert_eq!(
        kitty_transmit_png_file(&path, size(12, 3), id(7)).unwrap(),
        format!("\x1b_Ga=T,t=f,f=100,c=12,r=3,q=2,i=7,C=1;{encoded}\x1b\\"),
    );
}

#[test]
fn deletion_targets_one_image_and_its_data() {
    assert_eq!(kitty_delete_image(id(17)), "\x1b_Ga=d,d=I,i=17,q=2;\x1b\\",);
}

fn size(columns: u16, rows: u16) -> PlacementSize {
    PlacementSize::new(
        NonZeroU16::new(columns).unwrap(),
        NonZeroU16::new(rows).unwrap(),
    )
}

fn id(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
