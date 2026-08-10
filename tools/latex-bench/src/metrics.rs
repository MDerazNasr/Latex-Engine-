//! Percentile calculations for bounded benchmark samples.

use std::time::Duration;

pub(crate) fn percentile_95(samples: &mut [Duration]) -> Option<Duration> {
    if samples.is_empty() {
        return None;
    }
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(95).div_ceil(100);
    samples.get(rank.saturating_sub(1)).copied()
}

pub(crate) fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
