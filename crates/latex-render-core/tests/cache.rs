#![doc = "Bounded render cache integration tests."]

mod common;

use latex_render_core::{CacheInsert, CacheLimits, RenderCache};

use common::result;

#[test]
fn cache_rejects_zero_limits() {
    assert!(
        RenderCache::new(CacheLimits {
            max_entries: 0,
            max_bytes: 100,
        })
        .is_none()
    );
    assert!(
        RenderCache::new(CacheLimits {
            max_entries: 1,
            max_bytes: 0,
        })
        .is_none()
    );
}

#[test]
fn cache_tracks_hits_misses_and_clear() {
    let mut cache = cache(2, 1024);
    assert_eq!(cache.get("missing"), None);
    assert_eq!(cache.insert(result("a")), CacheInsert::Stored);
    assert_eq!(cache.get("a").expect("entry should exist").cache_key, "a");

    let stats = cache.stats();
    assert_eq!(stats.entries, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!(stats.bytes > 0);

    cache.clear();
    assert_eq!(cache.stats().entries, 0);
    assert_eq!(cache.stats().bytes, 0);
    assert_eq!(cache.stats().hits, 1);
}

#[test]
fn cache_evicts_the_least_recently_used_entry() {
    let mut cache = cache(2, 1024);
    cache.insert(result("a"));
    cache.insert(result("b"));
    assert!(cache.get("a").is_some());

    cache.insert(result("c"));

    assert!(cache.get("a").is_some());
    assert!(cache.get("b").is_none());
    assert!(cache.get("c").is_some());
    assert_eq!(cache.stats().evictions, 1);
}

#[test]
fn replacing_an_entry_does_not_count_as_an_eviction() {
    let mut cache = cache(1, 1024);
    let mut replacement = result("a");
    replacement.accessibility_text = "replacement".to_owned();

    cache.insert(result("a"));
    cache.insert(replacement);

    assert_eq!(
        cache
            .get("a")
            .expect("replacement should exist")
            .accessibility_text,
        "replacement"
    );
    assert_eq!(cache.stats().evictions, 0);
}

#[test]
fn cache_enforces_its_byte_limit() {
    let entry_bytes = result("a").svg.len() + result("a").accessibility_text.len() + 1;
    let mut cache = cache(3, entry_bytes + 1);

    cache.insert(result("a"));
    cache.insert(result("b"));

    assert!(cache.get("a").is_none());
    assert!(cache.get("b").is_some());
    assert_eq!(cache.stats().evictions, 1);
}

#[test]
fn oversized_and_keyless_results_are_not_stored() {
    let mut cache = cache(2, 8);
    assert_eq!(cache.insert(result("a")), CacheInsert::SkippedOversized);
    assert_eq!(cache.insert(result("")), CacheInsert::SkippedInvalidKey);
    assert_eq!(cache.stats().entries, 0);
}

fn cache(max_entries: usize, max_bytes: usize) -> RenderCache {
    RenderCache::new(CacheLimits {
        max_entries,
        max_bytes,
    })
    .expect("test limits should create a cache")
}
