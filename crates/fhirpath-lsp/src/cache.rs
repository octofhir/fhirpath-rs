//! Caching layer for analysis results

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

/// Cache key for analysis results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// Document URI
    pub uri: Url,
    /// Expression text (for per-expression caching)
    pub expression: String,
    /// Document version
    pub version: i32,
}

/// Cached analysis result
#[derive(Debug, Clone)]
pub struct CachedAnalysis {
    /// Analysis result (generic for now)
    pub result: Arc<Vec<u8>>, // Placeholder for actual analysis result
    /// When this was cached
    pub cached_at: Instant,
    /// Document version when cached
    pub version: i32,
}

/// Analysis cache with TTL and size limits
pub struct AnalysisCache {
    /// Cache storage
    cache: DashMap<CacheKey, CachedAnalysis>,
    /// Maximum cache entries
    max_entries: usize,
    /// Cache entry TTL
    ttl: Duration,
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            max_entries,
            ttl,
        }
    }

    /// Get cached analysis result
    pub fn get(&self, key: &CacheKey) -> Option<CachedAnalysis> {
        self.cache.get(key).and_then(|entry| {
            // Check if expired
            if entry.cached_at.elapsed() > self.ttl {
                drop(entry); // Drop read lock
                self.cache.remove(key);
                None
            } else {
                Some(entry.clone())
            }
        })
    }

    /// Insert analysis result into cache
    pub fn insert(&self, key: CacheKey, result: Arc<Vec<u8>>) {
        // Evict old entries if cache is full
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.cache.insert(
            key.clone(),
            CachedAnalysis {
                result,
                cached_at: Instant::now(),
                version: key.version,
            },
        );
    }

    /// Invalidate all cache entries for a document
    pub fn invalidate_document(&self, uri: &Url) {
        self.cache.retain(|key, _| &key.uri != uri);
    }

    /// Invalidate cache entries for a specific document version
    pub fn invalidate_version(&self, uri: &Url, version: i32) {
        self.cache
            .retain(|key, _| &key.uri != uri || key.version != version);
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            max_entries: self.max_entries,
        }
    }

    /// Evict oldest entries (simple LRU)
    fn evict_oldest(&self) {
        // Find oldest entry
        let oldest = self
            .cache
            .iter()
            .min_by_key(|entry| entry.cached_at)
            .map(|entry| entry.key().clone());

        if let Some(key) = oldest {
            self.cache.remove(&key);
            tracing::debug!("Evicted cache entry: {:?}", key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of entries
    pub entries: usize,
    /// Maximum allowed entries
    pub max_entries: usize,
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(300)) // 1000 entries, 5 min TTL
    }
}

/// Completion item cache
pub struct CompletionCache {
    /// Cache storage (resource type -> completions)
    cache: DashMap<String, Arc<Vec<String>>>,
}

impl CompletionCache {
    /// Create a new completion cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get cached completions for a resource type
    pub fn get(&self, resource_type: &str) -> Option<Arc<Vec<String>>> {
        self.cache.get(resource_type).map(|entry| entry.clone())
    }

    /// Cache completions for a resource type
    pub fn insert(&self, resource_type: String, completions: Arc<Vec<String>>) {
        self.cache.insert(resource_type, completions);
    }

    /// Clear cache (e.g., on config change)
    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for CompletionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = AnalysisCache::default();
        let key = CacheKey {
            uri: Url::parse("file:///test.fhirpath").unwrap(),
            expression: "Patient.name".to_string(),
            version: 1,
        };

        let result = Arc::new(vec![1, 2, 3]);
        cache.insert(key.clone(), result.clone());

        let cached = cache.get(&key).unwrap();
        assert_eq!(cached.result, result);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = AnalysisCache::default();
        let uri = Url::parse("file:///test.fhirpath").unwrap();

        let key = CacheKey {
            uri: uri.clone(),
            expression: "Patient.name".to_string(),
            version: 1,
        };

        cache.insert(key.clone(), Arc::new(vec![1, 2, 3]));
        assert!(cache.get(&key).is_some());

        cache.invalidate_document(&uri);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_ttl() {
        let cache = AnalysisCache::new(100, Duration::from_millis(10));
        let key = CacheKey {
            uri: Url::parse("file:///test.fhirpath").unwrap(),
            expression: "Patient.name".to_string(),
            version: 1,
        };

        cache.insert(key.clone(), Arc::new(vec![1, 2, 3]));
        assert!(cache.get(&key).is_some());

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(20));
        assert!(cache.get(&key).is_none());
    }
}
