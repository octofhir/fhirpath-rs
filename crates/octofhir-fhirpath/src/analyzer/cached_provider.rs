use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::core::ModelProvider;
use crate::core::model_provider::{ChoiceTypeInfo, ModelError};
use async_trait::async_trait;
use octofhir_fhir_model::{ElementInfo, FhirVersion, TypeInfo};

type Result<T> = std::result::Result<T, ModelError>;
use super::cache::{CacheInfo, ModelProviderCache};

/// Cached wrapper around a ModelProvider for performance optimization
/// This wrapper transparently caches all ModelProvider operations
pub struct CachedModelProvider {
    inner: Arc<dyn ModelProvider>,
    cache: Arc<Mutex<ModelProviderCache>>,
}

impl CachedModelProvider {
    /// Create a new cached model provider with default TTL (5 minutes)
    pub fn new(inner: Arc<dyn ModelProvider>) -> Self {
        Self::with_ttl(inner, Duration::from_secs(300))
    }

    /// Create a new cached model provider with custom TTL
    pub fn with_ttl(inner: Arc<dyn ModelProvider>, ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(Mutex::new(ModelProviderCache::new(ttl))),
        }
    }

    /// Get cache information and statistics
    pub async fn get_cache_info(&self) -> CacheInfo {
        let cache = self.cache.lock().await;
        cache.get_cache_info()
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// Manually trigger cleanup of expired cache entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().await;
        cache.cleanup_expired();
    }

    /// Generate cache performance report
    pub async fn cache_report(&self) -> String {
        let info = self.get_cache_info().await;
        info.report()
    }

    /// Check if cache hit ratio is below threshold (for monitoring)
    pub async fn is_cache_efficient(&self, threshold: f64) -> bool {
        let info = self.get_cache_info().await;
        info.statistics.hit_ratio >= threshold
    }

    /// Get the underlying provider (useful for bypass operations)
    pub fn inner(&self) -> &Arc<dyn ModelProvider> {
        &self.inner
    }
}

#[async_trait]
impl ModelProvider for CachedModelProvider {
    async fn get_type(&self, type_name: &str) -> Result<Option<TypeInfo>> {
        let mut cache = self.cache.lock().await;
        cache.get_type_cached(self.inner.as_ref(), type_name).await
    }

    async fn get_element_type(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
    ) -> Result<Option<TypeInfo>> {
        let mut cache = self.cache.lock().await;
        cache
            .get_element_type_cached(self.inner.as_ref(), parent_type, property_name)
            .await
    }

    fn of_type(&self, type_info: &TypeInfo, target_type: &str) -> Option<TypeInfo> {
        // This method is typically cheap computation, so we delegate directly
        // without caching to avoid cache overhead
        self.inner.of_type(type_info, target_type)
    }

    fn get_element_names(&self, parent_type: &TypeInfo) -> Vec<String> {
        // This method is typically cheap computation, so we delegate directly
        self.inner.get_element_names(parent_type)
    }

    async fn get_children_type(&self, parent_type: &TypeInfo) -> Result<Option<TypeInfo>> {
        // For now, delegate to inner provider - could be cached if expensive
        self.inner.get_children_type(parent_type).await
    }

    async fn get_elements(&self, type_name: &str) -> Result<Vec<ElementInfo>> {
        let mut cache = self.cache.lock().await;
        cache
            .get_elements_cached(self.inner.as_ref(), type_name)
            .await
    }

    async fn get_resource_types(&self) -> Result<Vec<String>> {
        let mut cache = self.cache.lock().await;
        cache.get_resource_types_cached(self.inner.as_ref()).await
    }

    async fn get_complex_types(&self) -> Result<Vec<String>> {
        let mut cache = self.cache.lock().await;
        cache.get_complex_types_cached(self.inner.as_ref()).await
    }

    async fn get_primitive_types(&self) -> Result<Vec<String>> {
        let mut cache = self.cache.lock().await;
        cache.get_primitive_types_cached(self.inner.as_ref()).await
    }

    async fn resource_type_exists(&self, resource_type: &str) -> Result<bool> {
        // Use cached resource types for efficiency
        let resource_types = self.get_resource_types().await?;
        Ok(resource_types.contains(&resource_type.to_string()))
    }

    async fn get_fhir_version(&self) -> Result<FhirVersion> {
        // FHIR version is stable, delegate directly
        self.inner.get_fhir_version().await
    }

    fn is_type_derived_from(&self, derived_type: &str, base_type: &str) -> bool {
        // Type hierarchy is typically stable computation, delegate directly
        self.inner.is_type_derived_from(derived_type, base_type)
    }

    async fn get_choice_types(
        &self,
        parent_type: &str,
        property_name: &str,
    ) -> Result<Option<Vec<ChoiceTypeInfo>>> {
        let mut cache = self.cache.lock().await;
        cache
            .get_choice_types_cached(self.inner.as_ref(), parent_type, property_name)
            .await
    }

    async fn get_union_types(&self, type_info: &TypeInfo) -> Result<Option<Vec<TypeInfo>>> {
        let mut cache = self.cache.lock().await;
        cache
            .get_union_types_cached(self.inner.as_ref(), type_info)
            .await
    }

    fn is_union_type(&self, type_info: &TypeInfo) -> bool {
        // Union type check is typically cheap computation, delegate directly
        self.inner.is_union_type(type_info)
    }
}

impl std::fmt::Debug for CachedModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedModelProvider")
            .field("inner", &format_args!("ModelProvider"))
            .field("cache", &format_args!("ModelProviderCache"))
            .finish()
    }
}

/// Builder for creating CachedModelProvider with custom configuration
pub struct CachedModelProviderBuilder {
    ttl: Duration,
    auto_cleanup: bool,
    cleanup_interval: Duration,
}

impl CachedModelProviderBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            ttl: Duration::from_secs(300), // 5 minutes default
            auto_cleanup: false,
            cleanup_interval: Duration::from_secs(60), // 1 minute default
        }
    }

    /// Set the cache TTL (time-to-live)
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Enable automatic cleanup of expired entries
    pub fn auto_cleanup(mut self, enabled: bool) -> Self {
        self.auto_cleanup = enabled;
        self
    }

    /// Set the interval for automatic cleanup
    pub fn cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }

    /// Build the CachedModelProvider
    pub fn build(self, inner: Arc<dyn ModelProvider>) -> CachedModelProvider {
        // TODO: Implement auto-cleanup in a background task if enabled
        // This would require a more sophisticated architecture with background threads
        // For now, users can manually call cleanup_expired()

        CachedModelProvider::with_ttl(inner, self.ttl)
    }
}

impl Default for CachedModelProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for cache management
pub mod cache_utils {
    use super::*;

    /// Create a high-performance cached provider for real-time usage
    pub fn create_realtime_cached_provider(inner: Arc<dyn ModelProvider>) -> CachedModelProvider {
        CachedModelProviderBuilder::new()
            .ttl(Duration::from_secs(300)) // 5 minutes
            .auto_cleanup(true)
            .cleanup_interval(Duration::from_secs(30)) // 30 seconds
            .build(inner)
    }

    /// Create a long-term cached provider for batch processing
    pub fn create_batch_cached_provider(inner: Arc<dyn ModelProvider>) -> CachedModelProvider {
        CachedModelProviderBuilder::new()
            .ttl(Duration::from_secs(3600)) // 1 hour
            .auto_cleanup(false)
            .build(inner)
    }

    /// Create a minimal cache for testing
    pub fn create_test_cached_provider(inner: Arc<dyn ModelProvider>) -> CachedModelProvider {
        CachedModelProviderBuilder::new()
            .ttl(Duration::from_secs(10)) // 10 seconds
            .auto_cleanup(false)
            .build(inner)
    }

    /// Monitor cache performance and log warnings if hit ratio is low
    pub async fn monitor_cache_performance(
        provider: &CachedModelProvider,
        min_hit_ratio: f64,
    ) -> bool {
        let info = provider.get_cache_info().await;
        let hit_ratio = info.statistics.hit_ratio;

        if info.statistics.total_requests > 100 && hit_ratio < min_hit_ratio {
            eprintln!(
                "Warning: Cache hit ratio ({:.2}%) is below threshold ({:.2}%)",
                hit_ratio * 100.0,
                min_hit_ratio * 100.0
            );
            false
        } else {
            true
        }
    }

    /// Get performance recommendations based on cache statistics
    pub async fn get_performance_recommendations(provider: &CachedModelProvider) -> Vec<String> {
        let info = provider.get_cache_info().await;
        let stats = &info.statistics;
        let mut recommendations = Vec::new();

        if stats.total_requests > 0 {
            if stats.hit_ratio < 0.5 {
                recommendations
                    .push("Consider increasing cache TTL - hit ratio is low".to_string());
            }

            if stats.hit_ratio > 0.95 && stats.cache_size > 1000 {
                recommendations
                    .push("Cache is very effective but large - consider reducing TTL".to_string());
            }

            if stats.evictions > stats.hits {
                recommendations
                    .push("High eviction rate - consider increasing cache TTL".to_string());
            }

            if stats.memory_usage_estimate > 10_000_000 {
                // 10MB
                recommendations.push("Cache memory usage is high - consider reducing TTL or implementing cache size limits".to_string());
            }
        } else {
            recommendations
                .push("No cache activity yet - monitor performance after usage".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cached_provider_creation() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        let info = cached.get_cache_info().await;
        assert_eq!(info.statistics.total_requests, 0);
        assert_eq!(info.statistics.hit_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_cached_provider_delegation() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // Test that calls are properly delegated
        let result = cached.get_type("Patient").await;
        assert!(result.is_ok());

        let info = cached.get_cache_info().await;
        assert_eq!(info.statistics.total_requests, 1);
        assert_eq!(info.statistics.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_effectiveness() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // First call - cache miss
        let _result1 = cached.get_type("Patient").await;

        // Second call - cache hit
        let _result2 = cached.get_type("Patient").await;

        let info = cached.get_cache_info().await;
        assert_eq!(info.statistics.total_requests, 2);
        assert_eq!(info.statistics.hits, 1);
        assert_eq!(info.statistics.misses, 1);
        assert_eq!(info.statistics.hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // Add some cache entries
        let _result1 = cached.get_type("Patient").await;
        let _result2 = cached.get_resource_types().await;

        assert!(cached.get_cache_info().await.statistics.total_requests > 0);

        // Clear cache
        cached.clear_cache().await;

        let info = cached.get_cache_info().await;
        assert_eq!(info.statistics.total_requests, 0);
        assert_eq!(info.statistics.cache_size, 0);
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProviderBuilder::new()
            .ttl(Duration::from_secs(60))
            .auto_cleanup(true)
            .cleanup_interval(Duration::from_secs(30))
            .build(inner);

        let info = cached.get_cache_info().await;
        assert_eq!(info.ttl, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_cache_utils_factory_methods() {
        let inner = Arc::new(EmptyModelProvider);

        let realtime = cache_utils::create_realtime_cached_provider(inner.clone());
        let batch = cache_utils::create_batch_cached_provider(inner.clone());
        let test = cache_utils::create_test_cached_provider(inner);

        // Different TTLs for different use cases
        assert_eq!(
            realtime.get_cache_info().await.ttl,
            Duration::from_secs(300)
        );
        assert_eq!(batch.get_cache_info().await.ttl, Duration::from_secs(3600));
        assert_eq!(test.get_cache_info().await.ttl, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_cache_monitoring() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // Initially no activity
        assert!(cache_utils::monitor_cache_performance(&cached, 0.5).await);

        // Add some requests
        for _ in 0..50 {
            let _ = cached.get_type("Patient").await;
        }
        for _ in 0..50 {
            let _ = cached.get_type("Observation").await;
        }

        // Should have good hit ratio now
        assert!(cache_utils::monitor_cache_performance(&cached, 0.3).await);
    }

    #[tokio::test]
    async fn test_performance_recommendations() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // Initially should suggest monitoring
        let recommendations = cache_utils::get_performance_recommendations(&cached).await;
        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("No cache activity yet"))
        );

        // Add some requests
        for i in 0..10 {
            let _ = cached.get_type(&format!("Type{}", i)).await;
        }

        // Should have different recommendations
        let recommendations = cache_utils::get_performance_recommendations(&cached).await;
        // With many different types, hit ratio will be low
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("hit ratio is low") || r.contains("monitor performance"))
        );
    }

    #[tokio::test]
    async fn test_cache_efficiency_check() {
        let inner = Arc::new(EmptyModelProvider);
        let cached = CachedModelProvider::new(inner);

        // Add repeated requests for good hit ratio
        for _ in 0..10 {
            let _ = cached.get_type("Patient").await;
        }

        // Should be efficient
        assert!(cached.is_cache_efficient(0.5).await);
        assert!(cached.is_cache_efficient(0.8).await);
        assert!(cached.is_cache_efficient(0.9).await);
    }
}
