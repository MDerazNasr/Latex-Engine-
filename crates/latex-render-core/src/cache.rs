//! Size bounded memory cache for completed renders.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::RenderedMath;

/// Resource limits for one render cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CacheLimits {
    /// Maximum stored entry count.
    pub max_entries: usize,
    /// Maximum estimated content bytes.
    pub max_bytes: usize,
}

impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            max_entries: 256,
            max_bytes: 32 * 1024 * 1024,
        }
    }
}

/// Outcome of adding one render result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheInsert {
    /// The value was stored and is available for lookup.
    Stored,
    /// The value alone exceeds the configured byte limit.
    SkippedOversized,
    /// The value has no cache key.
    SkippedInvalidKey,
}

/// Snapshot of cache usage and lifetime counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheStats {
    /// Current stored entry count.
    pub entries: usize,
    /// Current estimated content bytes.
    pub bytes: usize,
    /// Successful lookup count.
    pub hits: u64,
    /// Unsuccessful lookup count.
    pub misses: u64,
    /// Capacity eviction count.
    pub evictions: u64,
}

#[derive(Debug)]
struct CacheEntry {
    value: Arc<RenderedMath>,
    bytes: usize,
    last_used: u64,
}

/// An in memory least recently used cache bounded by entries and bytes.
#[derive(Debug)]
pub struct RenderCache {
    limits: CacheLimits,
    entries: HashMap<String, CacheEntry>,
    usage: BTreeMap<(u64, String), ()>,
    clock: u64,
    bytes: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl RenderCache {
    /// Creates an empty cache or returns `None` for zero limits.
    pub fn new(limits: CacheLimits) -> Option<Self> {
        if limits.max_entries == 0 || limits.max_bytes == 0 {
            return None;
        }
        Some(Self {
            limits,
            entries: HashMap::new(),
            usage: BTreeMap::new(),
            clock: 0,
            bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        })
    }

    /// Returns a shared result and updates its recent use position.
    pub fn get(&mut self, key: &str) -> Option<Arc<RenderedMath>> {
        if !self.entries.contains_key(key) {
            self.misses = self.misses.saturating_add(1);
            return None;
        }

        let next = self.next_clock();
        let entry = self
            .entries
            .get_mut(key)
            .expect("entry existence was checked");
        self.usage.remove(&(entry.last_used, key.to_owned()));
        entry.last_used = next;
        self.usage.insert((next, key.to_owned()), ());
        self.hits = self.hits.saturating_add(1);
        Some(Arc::clone(&entry.value))
    }

    /// Stores a result after evicting older entries when required.
    pub fn insert(&mut self, value: RenderedMath) -> CacheInsert {
        if value.cache_key.is_empty() {
            return CacheInsert::SkippedInvalidKey;
        }
        let value_bytes = value.estimated_size_bytes();
        if value_bytes > self.limits.max_bytes {
            return CacheInsert::SkippedOversized;
        }

        let key = value.cache_key.clone();
        self.remove_key(&key);
        while self.entries.len() >= self.limits.max_entries
            || self.bytes.saturating_add(value_bytes) > self.limits.max_bytes
        {
            if !self.evict_oldest() {
                break;
            }
        }

        let last_used = self.next_clock();
        self.bytes = self.bytes.saturating_add(value_bytes);
        self.usage.insert((last_used, key.clone()), ());
        self.entries.insert(
            key,
            CacheEntry {
                value: Arc::new(value),
                bytes: value_bytes,
                last_used,
            },
        );
        CacheInsert::Stored
    }

    /// Removes all entries while preserving lifetime counters.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.usage.clear();
        self.bytes = 0;
    }

    /// Returns current usage and lifetime counters.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            bytes: self.bytes,
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
        }
    }

    fn next_clock(&mut self) -> u64 {
        if self.clock == u64::MAX {
            self.rebuild_usage_clock();
        }
        self.clock += 1;
        self.clock
    }

    fn rebuild_usage_clock(&mut self) {
        let keys: Vec<String> = self.usage.keys().map(|(_, key)| key.clone()).collect();
        self.usage.clear();
        for (index, key) in keys.into_iter().enumerate() {
            let clock = index as u64 + 1;
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.last_used = clock;
                self.usage.insert((clock, key), ());
            }
        }
        self.clock = self.entries.len() as u64;
    }

    fn remove_key(&mut self, key: &str) -> bool {
        let Some(entry) = self.entries.remove(key) else {
            return false;
        };
        self.usage.remove(&(entry.last_used, key.to_owned()));
        self.bytes = self.bytes.saturating_sub(entry.bytes);
        true
    }

    fn evict_oldest(&mut self) -> bool {
        let Some((_, key)) = self.usage.first_key_value().map(|(key, _)| key.clone()) else {
            return false;
        };
        if self.remove_key(&key) {
            self.evictions = self.evictions.saturating_add(1);
            true
        } else {
            false
        }
    }
}
