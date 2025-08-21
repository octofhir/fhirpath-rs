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

//! Caching infrastructure for model provider

use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Statistics about cache usage
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Number of items evicted
    pub evictions: u64,
    /// Number of items currently in cache
    pub size: u64,
}

impl CacheStats {
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }
}

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_size: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Enable cache statistics tracking
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            ttl: Duration::from_secs(300), // 5 minutes
            enable_stats: true,
        }
    }
}

/// A cached entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// The cached value
    value: T,
    /// When this entry was created
    created_at: Instant,
    /// When this entry was last accessed
    last_accessed: Instant,
    /// Number of times this entry has been accessed
    access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Type information cache with TTL and LRU eviction
#[derive(Debug)]
pub struct TypeCache<T> {
    /// The cache storage
    cache: DashMap<String, CacheEntry<T>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl<T: Clone> TypeCache<T> {
    /// Create a new type cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new type cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &str) -> Option<T> {
        if let Some(mut entry) = self.cache.get_mut(key) {
            // Check if entry is expired
            if entry.is_expired(self.config.ttl) {
                // Remove expired entry
                drop(entry);
                self.cache.remove(key);
                self.record_miss();
                return None;
            }

            // Update access time and return value
            let value = entry.access().clone();
            self.record_hit();
            Some(value)
        } else {
            self.record_miss();
            None
        }
    }

    /// Put a value into the cache
    pub fn put(&self, key: String, value: T) {
        // Check if we need to evict entries
        if self.cache.len() >= self.config.max_size {
            self.evict_lru();
        }

        self.cache.insert(key, CacheEntry::new(value));
        self.update_size();
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &str) -> Option<T> {
        self.cache.remove(key).map(|(_, entry)| {
            self.update_size();
            entry.value
        })
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.cache.clear();
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.size = 0;
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        if self.config.enable_stats {
            self.stats.read().clone()
        } else {
            CacheStats::default()
        }
    }

    /// Get the current size of the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Get the current length of the cache (alias for size)
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut expired_keys = Vec::new();

        // Find expired entries
        for entry in self.cache.iter() {
            if (now - entry.created_at) > self.config.ttl {
                expired_keys.push(entry.key().clone());
            }
        }

        // Remove expired entries
        for key in expired_keys {
            self.cache.remove(&key);
        }

        self.update_size();
    }

    /// Evict the least recently used entry
    fn evict_lru(&self) {
        if self.cache.is_empty() {
            return;
        }

        // Find the entry with the oldest last_accessed time
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        for entry in self.cache.iter() {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }

        // Remove the oldest entry
        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            self.record_eviction();
        }
    }

    /// Record a cache hit
    fn record_hit(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.hits += 1;
        }
    }

    /// Record a cache miss
    fn record_miss(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.misses += 1;
        }
    }

    /// Record a cache eviction
    fn record_eviction(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.evictions += 1;
        }
    }

    /// Update cache size in statistics
    fn update_size(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            stats.size = self.cache.len() as u64;
        }
    }
}

impl<T: Clone> Default for TypeCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Element information cache
pub type ElementCache = TypeCache<super::provider::TypeReflectionInfo>;

/// Type reflection cache  
pub type TypeReflectionCache = TypeCache<super::provider::TypeReflectionInfo>;

/// Legacy cache manager that coordinates multiple caches
pub struct LegacyCacheManager {
    /// Type information cache
    pub type_cache: Arc<TypeReflectionCache>,
    /// Element information cache
    pub element_cache: Arc<ElementCache>,
    /// Cache configuration
    config: CacheConfig,
}

impl LegacyCacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        let config = CacheConfig::default();
        Self {
            type_cache: Arc::new(TypeReflectionCache::with_config(config.clone())),
            element_cache: Arc::new(ElementCache::with_config(config.clone())),
            config,
        }
    }

    /// Create a legacy cache manager with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            type_cache: Arc::new(TypeReflectionCache::with_config(config.clone())),
            element_cache: Arc::new(ElementCache::with_config(config.clone())),
            config,
        }
    }

    /// Get combined cache statistics
    pub fn combined_stats(&self) -> CacheStats {
        let type_stats = self.type_cache.stats();
        let element_stats = self.element_cache.stats();

        CacheStats {
            hits: type_stats.hits + element_stats.hits,
            misses: type_stats.misses + element_stats.misses,
            evictions: type_stats.evictions + element_stats.evictions,
            size: type_stats.size + element_stats.size,
        }
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.type_cache.clear();
        self.element_cache.clear();
    }

    /// Clean up expired entries in all caches
    pub fn cleanup_all_expired(&self) {
        self.type_cache.cleanup_expired();
        self.element_cache.cleanup_expired();
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl Default for LegacyCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::provider::TypeReflectionInfo;
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = TypeCache::new();
        let value = "test_value".to_string();

        // Initially empty
        assert!(cache.get("key1").is_none());

        // Put and get
        cache.put("key1".to_string(), value.clone());
        assert_eq!(cache.get("key1"), Some(value));

        // Size should be 1
        assert_eq!(cache.size(), 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_size: 2,
            ttl: Duration::from_secs(60),
            enable_stats: true,
        };
        let cache = TypeCache::with_config(config);

        // Fill cache to capacity
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        assert_eq!(cache.size(), 2);

        // Access key1 to make it more recently used
        cache.get("key1");

        // Add third item, should evict key2 (LRU)
        cache.put("key3".to_string(), "value3".to_string());
        assert_eq!(cache.size(), 2);

        // key1 and key3 should exist, key2 should be evicted
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_none());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_millis(100),
            enable_stats: true,
        };
        let cache = TypeCache::with_config(config);

        cache.put("key1".to_string(), "value1".to_string());
        assert!(cache.get("key1").is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired now
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = TypeCache::new();

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Miss
        cache.get("nonexistent");
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);

        // Put and hit
        cache.put("key1".to_string(), "value1".to_string());
        cache.get("key1");
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_manager() {
        let manager = LegacyCacheManager::new();

        let type_info = TypeReflectionInfo::SimpleType {
            namespace: "System".to_string(),
            name: "String".to_string(),
            base_type: None,
        };

        // Test type cache
        manager
            .type_cache
            .put("String".to_string(), type_info.clone());
        assert_eq!(manager.type_cache.get("String"), Some(type_info.clone()));

        // Test element cache
        manager
            .element_cache
            .put("Patient.name".to_string(), type_info.clone());
        assert_eq!(manager.element_cache.get("Patient.name"), Some(type_info));

        // Test combined stats
        let stats = manager.combined_stats();
        assert_eq!(stats.size, 2);
    }

    #[test]
    fn test_cache_cleanup() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_millis(100),
            enable_stats: true,
        };
        let cache = TypeCache::with_config(config);

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        assert_eq!(cache.size(), 2);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Cleanup expired entries
        cache.cleanup_expired();
        assert_eq!(cache.size(), 0);
    }
}
