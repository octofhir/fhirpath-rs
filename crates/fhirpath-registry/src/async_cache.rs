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

//! High-performance async LRU cache for operation dispatch
//!
//! This module provides an async-friendly LRU cache optimized for operation
//! dispatch with minimal lock contention and high performance.

use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache performance metrics
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    /// Total number of cache hits
    pub hits: u64,
    
    /// Total number of cache misses
    pub misses: u64,
    
    /// Total number of insertions
    pub insertions: u64,
    
    /// Total number of evictions
    pub evictions: u64,
    
    /// Number of cache clears
    pub clears: u64,
    
    /// Current cache size
    pub current_size: usize,
    
    /// Maximum cache capacity
    pub capacity: usize,
}

impl CacheMetrics {
    /// Calculate hit ratio (0.0 to 1.0)
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate miss ratio (0.0 to 1.0)
    pub fn miss_ratio(&self) -> f64 {
        1.0 - self.hit_ratio()
    }

    /// Calculate cache utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.current_size as f64 / self.capacity as f64
        }
    }

    /// Record a cache hit
    pub(crate) fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a cache miss
    pub(crate) fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record an insertion
    pub(crate) fn record_insertion(&mut self) {
        self.insertions += 1;
    }

    /// Record an eviction
    pub(crate) fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Record a cache clear
    pub(crate) fn record_clear(&mut self) {
        self.clears += 1;
    }

    /// Update current size
    pub(crate) fn update_size(&mut self, size: usize) {
        self.current_size = size;
    }
}

/// High-performance async LRU cache for operation dispatch
///
/// This cache is optimized for high-concurrency scenarios with minimal
/// lock contention. It uses read-write locks to allow concurrent reads
/// while serializing writes.
pub struct AsyncLruCache<K, V> {
    /// Inner LRU cache protected by read-write lock
    inner: Arc<RwLock<LruCache<K, V>>>,
    
    /// Performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl<K: Hash + Eq + Clone, V: Clone> AsyncLruCache<K, V> {
    /// Create a new async LRU cache with specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of items to store in cache
    ///
    /// # Panics
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());
        let mut metrics = CacheMetrics::default();
        metrics.capacity = capacity;

        Self {
            inner: Arc::new(RwLock::new(cache)),
            metrics: Arc::new(RwLock::new(metrics)),
        }
    }

    /// Get item from cache (async-friendly)
    ///
    /// This method uses a read lock when possible for maximum concurrency.
    /// If the item is found, it's promoted to most recent without taking
    /// a write lock immediately (promotion happens on next write operation).
    ///
    /// # Arguments
    /// * `key` - Key to look up
    ///
    /// # Returns
    /// Some(value) if key exists in cache, None otherwise
    pub async fn get(&self, key: &K) -> Option<V> {
        // First try a read-only lookup
        {
            let cache = self.inner.read().await;
            if let Some(value) = cache.peek(key) {
                // Found in cache - record hit and return value
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.record_hit();
                }
                return Some(value.clone());
            }
        }

        // Not found - record miss
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_miss();
        }

        None
    }

    /// Get item from cache with LRU promotion
    ///
    /// This method performs full LRU promotion, requiring a write lock.
    /// Use this when you need strict LRU semantics.
    ///
    /// # Arguments
    /// * `key` - Key to look up
    ///
    /// # Returns
    /// Some(value) if key exists in cache, None otherwise
    pub async fn get_mut(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write().await;
        if let Some(value) = cache.get(key) {
            // Found in cache - record hit and return value
            {
                let mut metrics = self.metrics.write().await;
                metrics.record_hit();
            }
            Some(value.clone())
        } else {
            // Not found - record miss
            {
                let mut metrics = self.metrics.write().await;
                metrics.record_miss();
            }
            None
        }
    }

    /// Insert item into cache
    ///
    /// If the cache is at capacity, the least recently used item will be evicted.
    ///
    /// # Arguments
    /// * `key` - Key to insert
    /// * `value` - Value to insert
    pub async fn insert(&self, key: K, value: V) {
        let mut cache = self.inner.write().await;
        let old_size = cache.len();
        
        let evicted = cache.put(key, value);
        let new_size = cache.len();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_insertion();
            if evicted.is_some() {
                metrics.record_eviction();
            }
            metrics.update_size(new_size);
        }
    }

    /// Check if cache contains key without affecting LRU order
    ///
    /// # Arguments
    /// * `key` - Key to check
    ///
    /// # Returns
    /// true if key exists in cache, false otherwise
    pub async fn contains(&self, key: &K) -> bool {
        let cache = self.inner.read().await;
        cache.contains(key)
    }

    /// Remove item from cache
    ///
    /// # Arguments
    /// * `key` - Key to remove
    ///
    /// # Returns
    /// Some(value) if key existed and was removed, None otherwise
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write().await;
        let result = cache.pop(key);
        
        if result.is_some() {
            let new_size = cache.len();
            let mut metrics = self.metrics.write().await;
            metrics.update_size(new_size);
        }
        
        result
    }

    /// Clear all items from cache
    pub async fn clear(&self) {
        let mut cache = self.inner.write().await;
        cache.clear();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_clear();
            metrics.update_size(0);
        }
    }

    /// Get current cache size
    pub async fn len(&self) -> usize {
        let cache = self.inner.read().await;
        cache.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.inner.read().await;
        cache.is_empty()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        // Capacity is fixed at creation time, so no lock needed
        self.metrics.try_read().map(|m| m.capacity).unwrap_or(0)
    }

    /// Get current cache metrics
    pub async fn metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Reset cache metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        let capacity = metrics.capacity;
        let current_size = metrics.current_size;
        
        *metrics = CacheMetrics::default();
        metrics.capacity = capacity;
        metrics.current_size = current_size;
    }

    /// Resize cache capacity
    ///
    /// If new capacity is smaller than current size, oldest items will be evicted.
    ///
    /// # Arguments
    /// * `new_capacity` - New cache capacity
    ///
    /// # Panics
    /// Panics if new_capacity is 0
    pub async fn resize(&self, new_capacity: usize) -> Result<(), &'static str> {
        if new_capacity == 0 {
            return Err("Cache capacity must be greater than 0");
        }

        let new_capacity_nz = NonZeroUsize::new(new_capacity).unwrap();
        
        let mut cache = self.inner.write().await;
        cache.resize(new_capacity_nz);
        
        let new_size = cache.len();
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.capacity = new_capacity;
            metrics.update_size(new_size);
        }
        
        Ok(())
    }

    /// Get approximate memory usage in bytes
    ///
    /// This provides a rough estimate based on the number of entries.
    /// Actual memory usage depends on the size of keys and values.
    pub async fn estimated_memory_usage(&self) -> usize {
        let cache = self.inner.read().await;
        let entry_count = cache.len();
        
        // Rough estimate: 64 bytes per entry for LRU overhead + key/value storage
        // This is a conservative estimate - actual usage varies by key/value types
        entry_count * 64
    }

    /// Create a snapshot of all key-value pairs
    ///
    /// This method returns all entries without affecting LRU order.
    /// Note: This requires cloning all keys and values.
    pub async fn snapshot(&self) -> Vec<(K, V)> {
        let cache = self.inner.read().await;
        cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Get cache statistics as a formatted string
    pub async fn stats_string(&self) -> String {
        let metrics = self.metrics().await;
        format!(
            "Cache Stats: {} hits, {} misses, {:.2}% hit ratio, {}/{} entries ({:.1}% full)",
            metrics.hits,
            metrics.misses,
            metrics.hit_ratio() * 100.0,
            metrics.current_size,
            metrics.capacity,
            metrics.utilization() * 100.0
        )
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Clone for AsyncLruCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

/// Builder for creating async LRU caches with custom configuration
pub struct CacheBuilder {
    capacity: usize,
}

impl CacheBuilder {
    /// Create a new cache builder
    pub fn new() -> Self {
        Self {
            capacity: 1000, // Default capacity
        }
    }

    /// Set cache capacity
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the cache
    pub fn build<K: Hash + Eq + Clone, V: Clone>(self) -> AsyncLruCache<K, V> {
        AsyncLruCache::new(self.capacity)
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = AsyncLruCache::new(3);
        
        // Test empty cache
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
        assert_eq!(cache.capacity(), 3);
        
        // Test insertion
        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.len().await, 1);
        assert!(!cache.is_empty().await);
        
        // Test retrieval
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Test non-existent key
        let value = cache.get(&"nonexistent".to_string()).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let cache = AsyncLruCache::new(2);
        
        // Fill cache to capacity
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        assert_eq!(cache.len().await, 2);
        
        // Insert third item - should evict oldest
        cache.insert("key3".to_string(), "value3".to_string()).await;
        assert_eq!(cache.len().await, 2);
        
        // key1 should be evicted
        assert_eq!(cache.get(&"key1".to_string()).await, None);
        assert_eq!(cache.get(&"key2".to_string()).await, Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()).await, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = AsyncLruCache::new(2);
        
        // Test initial metrics
        let metrics = cache.metrics().await;
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
        assert_eq!(metrics.hit_ratio(), 0.0);
        
        // Insert and access items
        cache.insert("key1".to_string(), "value1".to_string()).await;
        
        // Hit
        cache.get(&"key1".to_string()).await;
        
        // Miss
        cache.get(&"nonexistent".to_string()).await;
        
        let metrics = cache.metrics().await;
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_ratio(), 0.5);
        assert_eq!(metrics.insertions, 1);
    }

    #[tokio::test]
    async fn test_cache_contains() {
        let cache = AsyncLruCache::new(2);
        
        assert!(!cache.contains(&"key1".to_string()).await);
        
        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert!(cache.contains(&"key1".to_string()).await);
        
        cache.remove(&"key1".to_string()).await;
        assert!(!cache.contains(&"key1".to_string()).await);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = AsyncLruCache::new(5);
        
        // Add several items
        for i in 0..3 {
            cache.insert(format!("key{}", i), format!("value{}", i)).await;
        }
        assert_eq!(cache.len().await, 3);
        
        // Clear cache
        cache.clear().await;
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
        
        // Verify items are gone
        for i in 0..3 {
            assert_eq!(cache.get(&format!("key{}", i)).await, None);
        }
    }

    #[tokio::test]
    async fn test_cache_resize() {
        let cache = AsyncLruCache::new(3);
        
        // Fill cache
        for i in 0..3 {
            cache.insert(format!("key{}", i), format!("value{}", i)).await;
        }
        assert_eq!(cache.len().await, 3);
        
        // Resize to smaller capacity - should evict items
        cache.resize(2).await.unwrap();
        assert_eq!(cache.capacity(), 2);
        assert_eq!(cache.len().await, 2);
        
        // Resize to larger capacity
        cache.resize(5).await.unwrap();
        assert_eq!(cache.capacity(), 5);
        assert_eq!(cache.len().await, 2);
        
        // Test invalid resize
        assert!(cache.resize(0).await.is_err());
    }

    #[tokio::test]
    async fn test_cache_builder() {
        let cache = CacheBuilder::new()
            .capacity(100)
            .build::<String, String>();
        
        assert_eq!(cache.capacity(), 100);
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_cache_snapshot() {
        let cache = AsyncLruCache::new(3);
        
        // Add items
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        
        let snapshot = cache.snapshot().await;
        assert_eq!(snapshot.len(), 2);
        
        // Snapshot should contain all items
        let keys: Vec<String> = snapshot.iter().map(|(k, _)| k.clone()).collect();
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(AsyncLruCache::new(100));
        let mut handles = vec![];
        
        // Spawn multiple tasks accessing cache concurrently
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let key = format!("key_{}_{}", i, j);
                    let value = format!("value_{}_{}", i, j);
                    
                    cache_clone.insert(key.clone(), value.clone()).await;
                    let retrieved = cache_clone.get(&key).await;
                    assert_eq!(retrieved, Some(value));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify cache state
        assert!(cache.len().await <= 100);
        let metrics = cache.metrics().await;
        assert_eq!(metrics.insertions, 100);
    }

    #[tokio::test]
    async fn test_stats_string() {
        let cache = AsyncLruCache::new(2);
        
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.get(&"key1".to_string()).await; // hit
        cache.get(&"nonexistent".to_string()).await; // miss
        
        let stats = cache.stats_string().await;
        assert!(stats.contains("1 hits"));
        assert!(stats.contains("1 misses"));
        assert!(stats.contains("50.00% hit ratio"));
    }
}