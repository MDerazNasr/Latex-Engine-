#![doc = "Bounded session local PNG store integration tests."]

use std::fs;

use latex_terminal::{ImageSource, LocalPngStore, LocalPngStoreLimits, LocalStoreError};

const FIRST_PNG: &[u8] = include_bytes!("../../../fixtures/terminal/quadratic-formula.png");
const SECOND_PNG: &[u8] = include_bytes!("../../../fixtures/rendering/snapshots/power-dark.png");

#[test]
fn content_is_written_once_inside_a_private_session_directory() {
    let mut store = LocalPngStore::create(LocalPngStoreLimits::default())
        .expect("temporary store should be created");
    let first = store.store_png(FIRST_PNG).expect("PNG should be stored");
    let repeated = store.store_png(FIRST_PNG).expect("PNG should be reused");
    let ImageSource::LocalPng(path) = first else {
        panic!("store should return a local path");
    };

    assert_eq!(repeated, ImageSource::LocalPng(path.clone()));
    assert_eq!(path.parent(), Some(store.directory()));
    assert_eq!(
        fs::read(path).expect("stored PNG should be readable"),
        FIRST_PNG
    );
    assert_eq!(store.file_count(), 1);
    assert_eq!(store.used_bytes(), FIRST_PNG.len());
}

#[test]
fn malformed_and_over_capacity_inputs_fail_without_partial_files() {
    let limits =
        LocalPngStoreLimits::new(1, FIRST_PNG.len() - 1).expect("limits should be nonzero");
    let mut store = LocalPngStore::create(limits).expect("temporary store should be created");

    assert!(matches!(
        store.store_png(b"not a PNG"),
        Err(LocalStoreError::InvalidPng)
    ));
    assert!(matches!(
        store.store_png(FIRST_PNG),
        Err(LocalStoreError::CapacityExceeded)
    ));
    assert_eq!(store.file_count(), 0);
    assert_eq!(store.used_bytes(), 0);
    assert!(
        fs::read_dir(store.directory())
            .expect("store directory should be readable")
            .next()
            .is_none()
    );
}

#[test]
fn distinct_file_limit_is_enforced_but_deduplication_still_succeeds() {
    let limits = LocalPngStoreLimits::new(1, FIRST_PNG.len() + SECOND_PNG.len())
        .expect("limits should be valid");
    let mut store = LocalPngStore::create(limits).expect("temporary store should be created");

    store.store_png(FIRST_PNG).expect("first PNG should fit");
    store
        .store_png(FIRST_PNG)
        .expect("repeated PNG should not consume capacity");
    assert!(matches!(
        store.store_png(SECOND_PNG),
        Err(LocalStoreError::CapacityExceeded)
    ));
    assert_eq!(store.file_count(), 1);
}

#[test]
fn stores_are_unique_and_drop_removes_only_the_owned_directory() {
    let first = LocalPngStore::create(LocalPngStoreLimits::default())
        .expect("first store should be created");
    let second = LocalPngStore::create(LocalPngStoreLimits::default())
        .expect("second store should be created");
    let first_path = first.directory().to_path_buf();
    let second_path = second.directory().to_path_buf();

    assert_ne!(first_path, second_path);
    drop(first);
    assert!(!first_path.exists());
    assert!(second_path.is_dir());
    drop(second);
    assert!(!second_path.exists());
}

#[test]
fn zero_limits_are_rejected() {
    assert!(matches!(
        LocalPngStoreLimits::new(0, 1),
        Err(LocalStoreError::InvalidLimits)
    ));
    assert!(matches!(
        LocalPngStoreLimits::new(1, 0),
        Err(LocalStoreError::InvalidLimits)
    ));
}

#[cfg(unix)]
#[test]
fn unix_directory_and_file_permissions_are_private() {
    use std::os::unix::fs::PermissionsExt as _;

    let mut store = LocalPngStore::create(LocalPngStoreLimits::default())
        .expect("temporary store should be created");
    let source = store.store_png(FIRST_PNG).expect("PNG should be stored");
    let ImageSource::LocalPng(path) = source else {
        panic!("store should return a local path");
    };
    let directory_mode = fs::metadata(store.directory())
        .expect("directory metadata should exist")
        .permissions()
        .mode()
        & 0o777;
    let file_mode = fs::metadata(path)
        .expect("file metadata should exist")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(directory_mode, 0o700);
    assert_eq!(file_mode, 0o600);
}
