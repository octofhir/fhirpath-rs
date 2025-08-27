// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simplified cache implementation using LRU eviction strategy
//!
//! This module provides a clean, simple cache system that replaces the complex
//! multi-tier cache with a single LRU-based implementation. It maintains thread
//! safety while dramatically reducing complexity.

use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

/// Simplified cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache capacity (number of items to store)
    pub capacity: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { capacity: 1000 }
    }
}

/// Thread-safe LRU cache with basic statistics
///
/// This cache provides a simple alternative to the complex multi-tier system,
/// focusing on clarity and maintainability while preserving essential caching
/// functionality.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath_model::Cache;
///
/// // Create a cache with capacity for 100 items
/// let cache = Cache::new(100);
///
/// // Insert and retrieve values
/// cache.insert("key1".to_string(), "value1".to_string());
/// let value = cache.get(&"key1".to_string());
/// assert_eq!(value, Some("value1".to_string()));
///
/// // Check statistics
/// let stats = cache.stats();
/// println!("Hits: {}, Misses: {}", stats.hits, stats.misses);
/// ```
pub struct Cache<K, V> {
    /// LRU cache for storing key-value pairs
    cache: Arc<RwLock<LruCache<K, V>>>,
    /// Basic statistics tracking
    stats: Arc<RwLock<CacheStats>>,
}

/// Basic cache statistics
///
/// Provides essential metrics for monitoring cache performance without
/// the complexity of the multi-tier statistics system.
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
}

impl CacheStats {
    /// Calculate hit ratio as percentage
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache with specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of items to store in cache
    ///
    /// # Panics
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");

        let cache_capacity = NonZeroUsize::new(capacity).expect("Capacity must be non-zero");

        Self {
            cache: Arc::new(RwLock::new(LruCache::new(cache_capacity))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a value from the cache
    ///
    /// Returns `Some(value)` if the key exists, `None` otherwise.
    /// Updates the LRU order and increments hit/miss statistics.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        match cache.get(key) {
            Some(value) => {
                stats.hits += 1;
                Some(value.clone())
            }
            None => {
                stats.misses += 1;
                None
            }
        }
    }

    /// Insert a key-value pair into the cache
    ///
    /// If the cache is at capacity, the least recently used item will be evicted.
    /// Returns the evicted value if any.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut cache = self.cache.write().unwrap();
        cache.put(key, value)
    }

    /// Check if a key exists in the cache without updating LRU order
    ///
    /// This is useful for checking existence without affecting eviction order.
    pub fn contains(&self, key: &K) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains(key)
    }

    /// Remove a key from the cache
    ///
    /// Returns the removed value if it existed.
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().unwrap();
        cache.pop(key)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        cache.clear();
        stats.reset();
    }

    /// Get current number of items in cache
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.cap().get()
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Reset cache statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.reset();
    }
}

impl<K, V> Clone for Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl<K, V> std::fmt::Debug for Cache<K, V>
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats();
        f.debug_struct("Cache")
            .field("capacity", &self.capacity())
            .field("len", &self.len())
            .field("hit_ratio", &format!("{:.1}%", stats.hit_ratio()))
            .field("hits", &stats.hits)
            .field("misses", &stats.misses)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::new(3);

        // Test insertion and retrieval
        assert_eq!(cache.insert("key1".to_string(), "value1".to_string()), None);
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Test cache miss
        assert_eq!(cache.get(&"nonexistent".to_string()), None);

        // Test contains
        assert!(cache.contains(&"key1".to_string()));
        assert!(!cache.contains(&"nonexistent".to_string()));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = Cache::new(2);

        // Fill cache to capacity
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        // Access key1 to make it recently used
        cache.get(&"key1".to_string());

        // Insert key3, should evict key2 (least recently used)
        cache.insert("key3".to_string(), "value3".to_string());

        assert!(cache.contains(&"key1".to_string()));
        assert!(!cache.contains(&"key2".to_string()));
        assert!(cache.contains(&"key3".to_string()));
    }

    #[test]
    fn test_cache_statistics() {
        let cache = Cache::new(10);

        // Generate some hits and misses
        cache.insert("key1".to_string(), "value1".to_string());
        cache.get(&"key1".to_string()); // hit
        cache.get(&"key1".to_string()); // hit
        cache.get(&"nonexistent".to_string()); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_ratio() - 66.66666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(10);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.get(&"key1".to_string()); // Generate some stats

        assert_eq!(cache.len(), 1);
        assert!(cache.stats().hits > 0);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_remove() {
        let cache = Cache::new(10);

        cache.insert("key1".to_string(), "value1".to_string());
        assert!(cache.contains(&"key1".to_string()));

        let removed = cache.remove(&"key1".to_string());
        assert_eq!(removed, Some("value1".to_string()));
        assert!(!cache.contains(&"key1".to_string()));

        // Test removing non-existent key
        let removed = cache.remove(&"nonexistent".to_string());
        assert_eq!(removed, None);
    }

    #[test]
    #[should_panic(expected = "Cache capacity must be greater than 0")]
    fn test_cache_zero_capacity_panics() {
        Cache::<String, String>::new(0);
    }

    #[test]
    fn test_cache_clone() {
        let cache1 = Cache::new(10);
        cache1.insert("key1".to_string(), "value1".to_string());

        let cache2 = cache1.clone();

        // Both caches should see the same data
        assert_eq!(cache2.get(&"key1".to_string()), Some("value1".to_string()));

        // Inserting in one affects the other (shared state)
        cache2.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache1.get(&"key2".to_string()), Some("value2".to_string()));
    }

    #[test]
    fn test_cache_debug_formatting() {
        let cache = Cache::new(100);
        cache.insert("test".to_string(), "value".to_string());
        cache.get(&"test".to_string()); // Generate hit
        cache.get(&"missing".to_string()); // Generate miss

        let debug_str = format!("{:?}", cache);
        assert!(debug_str.contains("Cache"));
        assert!(debug_str.contains("capacity"));
        assert!(debug_str.contains("hit_ratio"));
    }
}
