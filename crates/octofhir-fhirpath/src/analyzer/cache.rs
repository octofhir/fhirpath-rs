use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::ModelProvider;
use crate::core::model_provider::{ChoiceTypeInfo, ModelError};
use octofhir_fhir_model::{ElementInfo, TypeInfo};

/// Model provider cache with TTL and performance statistics
pub struct ModelProviderCache {
    type_cache: HashMap<String, CacheEntry<Option<TypeInfo>>>,
    element_cache: HashMap<(String, String), CacheEntry<Option<TypeInfo>>>,
    choice_cache: HashMap<(String, String), CacheEntry<Option<Vec<ChoiceTypeInfo>>>>,
    union_cache: HashMap<String, CacheEntry<Option<Vec<TypeInfo>>>>,
    elements_cache: HashMap<String, CacheEntry<Vec<ElementInfo>>>,
    resource_types_cache: Option<CacheEntry<Vec<String>>>,
    complex_types_cache: Option<CacheEntry<Vec<String>>>,
    primitive_types_cache: Option<CacheEntry<Vec<String>>>,
    cache_stats: CacheStatistics,
    ttl: Duration,
}

/// Cache entry with timestamp for TTL support
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

/// Comprehensive cache statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub total_requests: usize,
    pub hit_ratio: f64,
    pub cache_size: usize,
    pub memory_usage_estimate: usize,
}

impl CacheStatistics {
    fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            total_requests: 0,
            hit_ratio: 0.0,
            cache_size: 0,
            memory_usage_estimate: 0,
        }
    }

    fn record_hit(&mut self) {
        self.hits += 1;
        self.total_requests += 1;
        self.update_hit_ratio();
    }

    fn record_miss(&mut self) {
        self.misses += 1;
        self.total_requests += 1;
        self.update_hit_ratio();
    }

    fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    fn update_hit_ratio(&mut self) {
        if self.total_requests > 0 {
            self.hit_ratio = self.hits as f64 / self.total_requests as f64;
        }
    }

    fn update_cache_size(&mut self, size: usize) {
        self.cache_size = size;
        // Rough memory estimate (this is a simplified calculation)
        self.memory_usage_estimate = size * 128; // Assume ~128 bytes per cache entry on average
    }
}

impl ModelProviderCache {
    /// Create a new cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            type_cache: HashMap::new(),
            element_cache: HashMap::new(),
            choice_cache: HashMap::new(),
            union_cache: HashMap::new(),
            elements_cache: HashMap::new(),
            resource_types_cache: None,
            complex_types_cache: None,
            primitive_types_cache: None,
            cache_stats: CacheStatistics::new(),
            ttl,
        }
    }

    /// Cache type lookups with TTL support
    pub async fn get_type_cached(
        &mut self,
        provider: &dyn ModelProvider,
        type_name: &str,
    ) -> Result<Option<TypeInfo>, ModelError> {
        // Check cache first
        if let Some(entry) = self.type_cache.get(type_name) {
            if !entry.is_expired(self.ttl) {
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired, remove it
                self.type_cache.remove(type_name);
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_type(type_name).await?;

        // Cache the result
        self.type_cache
            .insert(type_name.to_string(), CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache element type lookups with TTL support
    pub async fn get_element_type_cached(
        &mut self,
        provider: &dyn ModelProvider,
        parent_type: &TypeInfo,
        property_name: &str,
    ) -> Result<Option<TypeInfo>, ModelError> {
        let key = (parent_type.type_name.clone(), property_name.to_string());

        // Check cache first
        if let Some(entry) = self.element_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired, remove it
                self.element_cache.remove(&key);
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider
            .get_element_type(parent_type, property_name)
            .await?;

        // Cache the result
        self.element_cache
            .insert(key, CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache choice type lookups
    pub async fn get_choice_types_cached(
        &mut self,
        provider: &dyn ModelProvider,
        parent_type: &str,
        property_name: &str,
    ) -> Result<Option<Vec<ChoiceTypeInfo>>, ModelError> {
        let key = (parent_type.to_string(), property_name.to_string());

        // Check cache first
        if let Some(entry) = self.choice_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired, remove it
                self.choice_cache.remove(&key);
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider
            .get_choice_types(parent_type, property_name)
            .await?;

        // Cache the result
        self.choice_cache
            .insert(key, CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache union type lookups
    pub async fn get_union_types_cached(
        &mut self,
        provider: &dyn ModelProvider,
        type_info: &TypeInfo,
    ) -> Result<Option<Vec<TypeInfo>>, ModelError> {
        let key = type_info.type_name.clone();

        // Check cache first
        if let Some(entry) = self.union_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired, remove it
                self.union_cache.remove(&key);
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_union_types(type_info).await?;

        // Cache the result
        self.union_cache
            .insert(key, CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache elements lookup
    pub async fn get_elements_cached(
        &mut self,
        provider: &dyn ModelProvider,
        type_name: &str,
    ) -> Result<Vec<ElementInfo>, ModelError> {
        // Check cache first
        if let Some(entry) = self.elements_cache.get(type_name) {
            if !entry.is_expired(self.ttl) {
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired, remove it
                self.elements_cache.remove(type_name);
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_elements(type_name).await?;

        // Cache the result
        self.elements_cache
            .insert(type_name.to_string(), CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache resource types lookup (usually stable, longer TTL)
    pub async fn get_resource_types_cached(
        &mut self,
        provider: &dyn ModelProvider,
    ) -> Result<Vec<String>, ModelError> {
        // Check cache first
        if let Some(entry) = &self.resource_types_cache {
            if !entry.is_expired(self.ttl * 10) {
                // 10x longer TTL for stable data
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired
                self.resource_types_cache = None;
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_resource_types().await?;

        // Cache the result
        self.resource_types_cache = Some(CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache complex types lookup
    pub async fn get_complex_types_cached(
        &mut self,
        provider: &dyn ModelProvider,
    ) -> Result<Vec<String>, ModelError> {
        // Check cache first
        if let Some(entry) = &self.complex_types_cache {
            if !entry.is_expired(self.ttl * 10) {
                // 10x longer TTL for stable data
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired
                self.complex_types_cache = None;
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_complex_types().await?;

        // Cache the result
        self.complex_types_cache = Some(CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Cache primitive types lookup
    pub async fn get_primitive_types_cached(
        &mut self,
        provider: &dyn ModelProvider,
    ) -> Result<Vec<String>, ModelError> {
        // Check cache first
        if let Some(entry) = &self.primitive_types_cache {
            if !entry.is_expired(self.ttl * 10) {
                // 10x longer TTL for stable data
                self.cache_stats.record_hit();
                return Ok(entry.value.clone());
            } else {
                // Entry expired
                self.primitive_types_cache = None;
                self.cache_stats.record_eviction();
            }
        }

        // Cache miss - fetch from provider
        self.cache_stats.record_miss();
        let result = provider.get_primitive_types().await?;

        // Cache the result
        self.primitive_types_cache = Some(CacheEntry::new(result.clone()));
        self.update_cache_stats();

        Ok(result)
    }

    /// Get current cache statistics
    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.cache_stats
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.type_cache.clear();
        self.element_cache.clear();
        self.choice_cache.clear();
        self.union_cache.clear();
        self.elements_cache.clear();
        self.resource_types_cache = None;
        self.complex_types_cache = None;
        self.primitive_types_cache = None;
        self.cache_stats = CacheStatistics::new();
    }

    /// Remove expired entries (manual cleanup)
    pub fn cleanup_expired(&mut self) {
        let _now = Instant::now();
        let ttl = self.ttl;
        let mut evictions = 0;

        // Clean type cache
        self.type_cache.retain(|_, entry| {
            if entry.timestamp.elapsed() > ttl {
                evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean element cache
        self.element_cache.retain(|_, entry| {
            if entry.timestamp.elapsed() > ttl {
                evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean choice cache
        self.choice_cache.retain(|_, entry| {
            if entry.timestamp.elapsed() > ttl {
                evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean union cache
        self.union_cache.retain(|_, entry| {
            if entry.timestamp.elapsed() > ttl {
                evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean elements cache
        self.elements_cache.retain(|_, entry| {
            if entry.timestamp.elapsed() > ttl {
                evictions += 1;
                false
            } else {
                true
            }
        });

        // Clean stable caches (longer TTL)
        let long_ttl = ttl * 10;
        if let Some(entry) = &self.resource_types_cache {
            if entry.timestamp.elapsed() > long_ttl {
                self.resource_types_cache = None;
                evictions += 1;
            }
        }

        if let Some(entry) = &self.complex_types_cache {
            if entry.timestamp.elapsed() > long_ttl {
                self.complex_types_cache = None;
                evictions += 1;
            }
        }

        if let Some(entry) = &self.primitive_types_cache {
            if entry.timestamp.elapsed() > long_ttl {
                self.primitive_types_cache = None;
                evictions += 1;
            }
        }

        self.cache_stats.evictions += evictions;
        self.update_cache_stats();
    }

    /// Update cache statistics
    fn update_cache_stats(&mut self) {
        let total_size = self.type_cache.len()
            + self.element_cache.len()
            + self.choice_cache.len()
            + self.union_cache.len()
            + self.elements_cache.len()
            + if self.resource_types_cache.is_some() {
                1
            } else {
                0
            }
            + if self.complex_types_cache.is_some() {
                1
            } else {
                0
            }
            + if self.primitive_types_cache.is_some() {
                1
            } else {
                0
            };

        self.cache_stats.update_cache_size(total_size);
    }

    /// Get cache configuration info
    pub fn get_cache_info(&self) -> CacheInfo {
        CacheInfo {
            ttl: self.ttl,
            statistics: self.cache_stats.clone(),
        }
    }
}

/// Cache configuration and status information
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub ttl: Duration,
    pub statistics: CacheStatistics,
}

impl CacheInfo {
    /// Generate a human-readable cache report
    pub fn report(&self) -> String {
        format!(
            "ModelProvider Cache Report:\n\
             TTL: {:?}\n\
             Total Requests: {}\n\
             Cache Hits: {} ({:.2}%)\n\
             Cache Misses: {}\n\
             Cache Evictions: {}\n\
             Current Cache Size: {} entries\n\
             Estimated Memory Usage: {} bytes",
            self.ttl,
            self.statistics.total_requests,
            self.statistics.hits,
            self.statistics.hit_ratio * 100.0,
            self.statistics.misses,
            self.statistics.evictions,
            self.statistics.cache_size,
            self.statistics.memory_usage_estimate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = ModelProviderCache::new(Duration::from_secs(60));
        let stats = cache.get_statistics();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.hit_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_type_caching() {
        let mut cache = ModelProviderCache::new(Duration::from_secs(60));
        let provider = EmptyModelProvider;

        // First request should be a miss
        let result1 = cache.get_type_cached(&provider, "Patient").await;
        assert!(result1.is_ok());
        assert_eq!(cache.get_statistics().misses, 1);
        assert_eq!(cache.get_statistics().hits, 0);

        // Second request should be a hit
        let result2 = cache.get_type_cached(&provider, "Patient").await;
        assert!(result2.is_ok());
        assert_eq!(cache.get_statistics().misses, 1);
        assert_eq!(cache.get_statistics().hits, 1);
        assert_eq!(cache.get_statistics().hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut cache = ModelProviderCache::new(Duration::from_millis(50));
        let provider = EmptyModelProvider;

        // First request
        let _result1 = cache.get_type_cached(&provider, "Patient").await;
        assert_eq!(cache.get_statistics().misses, 1);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Second request should be a miss due to expiration
        let _result2 = cache.get_type_cached(&provider, "Patient").await;
        assert_eq!(cache.get_statistics().misses, 2);
        assert_eq!(cache.get_statistics().evictions, 1);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let mut cache = ModelProviderCache::new(Duration::from_secs(60));
        let provider = EmptyModelProvider;

        // Add some entries
        let _result1 = cache.get_type_cached(&provider, "Patient").await;
        let _result2 = cache.get_resource_types_cached(&provider).await;

        assert!(cache.get_statistics().total_requests > 0);
        assert!(cache.get_statistics().cache_size > 0);

        // Clear cache
        cache.clear();

        assert_eq!(cache.get_statistics().total_requests, 0);
        assert_eq!(cache.get_statistics().cache_size, 0);
    }

    #[test]
    fn test_cache_info_report() {
        let cache = ModelProviderCache::new(Duration::from_secs(60));
        let info = cache.get_cache_info();
        let report = info.report();

        assert!(report.contains("ModelProvider Cache Report"));
        assert!(report.contains("TTL: 60s"));
        assert!(report.contains("Total Requests: 0"));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let mut cache = ModelProviderCache::new(Duration::from_millis(50));
        let provider = EmptyModelProvider;

        // Add some entries
        let _result1 = cache.get_type_cached(&provider, "Patient").await;
        let _result2 = cache.get_type_cached(&provider, "Observation").await;

        assert_eq!(cache.get_statistics().cache_size, 2);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Manual cleanup
        cache.cleanup_expired();

        assert_eq!(cache.get_statistics().cache_size, 0);
        assert_eq!(cache.get_statistics().evictions, 2);
    }
}
