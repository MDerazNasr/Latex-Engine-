use std::time::Duration;

use crate::metrics::{milliseconds, percentile_95};

#[test]
fn percentile_rejects_empty_samples() {
    assert_eq!(percentile_95(&mut []), None);
}

#[test]
fn percentile_uses_the_nearest_rank() {
    let mut samples = (1..=100)
        .rev()
        .map(Duration::from_millis)
        .collect::<Vec<_>>();

    assert_eq!(percentile_95(&mut samples), Some(Duration::from_millis(95)));
}

#[test]
fn duration_formatting_preserves_submillisecond_precision() {
    assert_eq!(milliseconds(Duration::from_micros(125)), 0.125);
}
