//! Function caching infrastructure for improved performance

pub mod config;
pub mod key;
pub mod stats;
#[cfg(test)]
mod comprehensive_tests;

pub use config::CacheConfig;
pub use key::FunctionCacheKey;
pub use stats::CacheStatistics;

use crate::function::FunctionImpl;
use dashmap::DashMap;
use fhirpath_model::FhirPathValue;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Result cache entry with timestamp for TTL support
#[derive(Clone)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// When the entry was created
    pub created_at: Instant,
}

impl<T: std::fmt::Debug> std::fmt::Debug for CacheEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheEntry")
            .field("value", &self.value)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T) -> Self {
        Self {
            value,
            created_at: Instant::now(),
        }
    }

    /// Check if this cache entry has expired based on TTL
    pub fn is_expired(&self, ttl: Option<Duration>) -> bool {
        if let Some(ttl) = ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

/// Thread-safe cache for function resolutions
pub struct FunctionResolutionCache {
    cache: Arc<DashMap<FunctionCacheKey, CacheEntry<Arc<FunctionImpl>>>>,
    stats: Arc<CacheStatistics>,
    config: Arc<CacheConfig>,
}

impl std::fmt::Debug for FunctionResolutionCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionResolutionCache")
            .field("cache_size", &self.cache.len())
            .field("stats", &self.stats)
            .field("config", &self.config)
            .finish()
    }
}

impl Clone for FunctionResolutionCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
            config: Arc::clone(&self.config),
        }
    }
}

impl FunctionResolutionCache {
    /// Create a new function resolution cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::with_capacity(config.resolution_cache_size)),
            stats: Arc::new(CacheStatistics::default()),
            config: Arc::new(config),
        }
    }

    /// Get a cached function implementation
    pub fn get(&self, key: &FunctionCacheKey) -> Option<Arc<FunctionImpl>> {
        let entry = self.cache.get(key)?;
        
        if entry.is_expired(self.config.cache_ttl) {
            self.cache.remove(key);
            self.stats.record_miss();
            return None;
        }
        
        self.stats.record_hit();
        Some(Arc::clone(&entry.value))
    }

    /// Insert a function implementation into the cache
    pub fn insert(&self, key: FunctionCacheKey, value: Arc<FunctionImpl>) {
        // Simple size management - remove oldest entries if at capacity
        if self.cache.len() >= self.config.resolution_cache_size {
            // In a real implementation, we'd use an LRU policy
            // For now, just remove a random entry
            if let Some(entry) = self.cache.iter().next() {
                self.cache.remove(entry.key());
                self.stats.record_eviction();
            }
        }
        
        self.cache.insert(key, CacheEntry::new(value));
    }

    /// Clear all cached function resolutions
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get resolution cache statistics
    pub fn stats(&self) -> &CacheStatistics {
        &self.stats
    }
}

/// Thread-safe cache for pure function results
pub struct FunctionResultCache {
    cache: Arc<DashMap<String, CacheEntry<FhirPathValue>>>,
    stats: Arc<CacheStatistics>,
    config: Arc<CacheConfig>,
}

impl std::fmt::Debug for FunctionResultCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionResultCache")
            .field("cache_size", &self.cache.len())
            .field("stats", &self.stats)
            .field("config", &self.config)
            .finish()
    }
}

impl Clone for FunctionResultCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
            config: Arc::clone(&self.config),
        }
    }
}

impl FunctionResultCache {
    /// Create a new function result cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::with_capacity(config.result_cache_size)),
            stats: Arc::new(CacheStatistics::default()),
            config: Arc::new(config),
        }
    }

    /// Get a cached function result
    pub fn get(&self, key: &str) -> Option<FhirPathValue> {
        if !self.config.enable_result_caching {
            return None;
        }

        let entry = self.cache.get(key)?;
        
        if entry.is_expired(self.config.cache_ttl) {
            self.cache.remove(key);
            self.stats.record_miss();
            return None;
        }
        
        self.stats.record_hit();
        Some(entry.value.clone())
    }

    /// Insert a function result into the cache
    pub fn insert(&self, key: String, value: FhirPathValue) {
        if !self.config.enable_result_caching {
            return;
        }

        // Simple size management
        if self.cache.len() >= self.config.result_cache_size {
            if let Some(entry) = self.cache.iter().next() {
                self.cache.remove(entry.key());
                self.stats.record_eviction();
            }
        }
        
        self.cache.insert(key, CacheEntry::new(value));
    }

    /// Clear all cached function results
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get result cache statistics
    pub fn stats(&self) -> &CacheStatistics {
        &self.stats
    }
}

/// Generate a cache key for function results
pub fn generate_result_cache_key(
    function_name: &str,
    args: &[FhirPathValue],
    context_hash: u64,
) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    function_name.hash(&mut hasher);
    
    for arg in args {
        // Hash the debug representation for now
        // In production, we'd implement proper hashing for FhirPathValue
        format!("{:?}", arg).hash(&mut hasher);
    }
    
    context_hash.hash(&mut hasher);
    
    format!("{}:{:x}", function_name, hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fhirpath_model::TypeInfo;

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test");
        assert!(!entry.is_expired(None));
        assert!(!entry.is_expired(Some(Duration::from_secs(1))));
        
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired(Some(Duration::from_millis(5))));
    }

    #[test]
    fn test_resolution_cache_basic() {
        let config = CacheConfig::default();
        let cache = FunctionResolutionCache::new(config);
        
        let key = FunctionCacheKey::new("test", vec![TypeInfo::String]);
        assert!(cache.get(&key).is_none());
        
        // Would need a real FunctionImpl to test insertion
        // This is just a placeholder to show the structure
    }

    #[test]
    fn test_result_cache_key_generation() {
        let key1 = generate_result_cache_key(
            "test",
            &[FhirPathValue::String("hello".to_string())],
            12345,
        );
        
        let key2 = generate_result_cache_key(
            "test",
            &[FhirPathValue::String("hello".to_string())],
            12345,
        );
        
        let key3 = generate_result_cache_key(
            "test",
            &[FhirPathValue::String("world".to_string())],
            12345,
        );
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}