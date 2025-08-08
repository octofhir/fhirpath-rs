//! Async-first memory pool for FHIRPath pipeline optimization
//!
//! This module provides memory pools optimized for async contexts, avoiding
//! thread-local storage patterns that break async execution. Uses tokio::sync::Mutex
//! for proper async compatibility and per-task pools to reduce contention.

use crate::ast::ExpressionNode;
use crate::model::value::FhirPathValue;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

/// Configuration for memory pool behavior
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial capacity for each pool
    pub initial_capacity: usize,
    /// Maximum number of objects to keep in pool
    pub max_capacity: usize,
    /// Enable automatic pool size adjustment
    pub auto_adjust: bool,
    /// Pool warming threshold (warm when usage exceeds this ratio)
    pub warm_threshold: f64,
    /// Cleanup interval for weak references (in seconds)
    pub cleanup_interval_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 32,
            max_capacity: 256,
            auto_adjust: true,
            warm_threshold: 0.8,
            cleanup_interval_secs: 60,
        }
    }
}

/// Statistics for a memory pool
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of objects currently in pool
    pub pool_size: usize,
    /// Number of objects currently borrowed
    pub borrowed_count: usize,
    /// Total allocations since creation
    pub total_allocations: u64,
    /// Total pool hits since creation
    pub pool_hits: u64,
    /// Total pool misses since creation
    pub pool_misses: u64,
    /// Number of cleanup cycles performed
    pub cleanup_cycles: u64,
}

impl PoolStats {
    /// Calculate hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_allocations == 0 {
            return 0.0;
        }
        (self.pool_hits as f64 / self.total_allocations as f64) * 100.0
    }
}

/// Generic async-safe object pool
pub struct AsyncPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    config: PoolConfig,
    stats: Arc<Mutex<PoolStats>>,
    weak_refs: Arc<Mutex<Vec<Weak<T>>>>,
}

impl<T> Default for AsyncPool<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsyncPool<T>
where
    T: Default + Clone + Send + Sync + 'static,
{
    /// Create a new async pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new async pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        let initial_objects = (0..config.initial_capacity).map(|_| T::default()).collect();

        Self {
            objects: Arc::new(Mutex::new(initial_objects)),
            config,
            stats: Arc::new(Mutex::new(PoolStats::default())),
            weak_refs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Borrow an object from the pool
    pub async fn borrow(&self) -> PooledObject<T> {
        let mut stats = self.stats.lock().await;
        stats.total_allocations += 1;

        let mut objects = self.objects.lock().await;

        if let Some(obj) = objects.pop() {
            stats.pool_hits += 1;
            stats.borrowed_count += 1;
            drop(stats);
            drop(objects);

            PooledObject::new(Arc::new(obj), self.clone_pool_ref())
        } else {
            stats.pool_misses += 1;
            stats.borrowed_count += 1;
            drop(stats);
            drop(objects);

            // Create new object
            PooledObject::new(Arc::new(T::default()), self.clone_pool_ref())
        }
    }

    /// Return an object to the pool
    pub async fn return_object(&self, obj: T) {
        let mut objects = self.objects.lock().await;
        let mut stats = self.stats.lock().await;

        if objects.len() < self.config.max_capacity {
            objects.push(obj);
        }

        stats.borrowed_count = stats.borrowed_count.saturating_sub(1);
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let stats = self.stats.lock().await;
        let objects = self.objects.lock().await;

        PoolStats {
            pool_size: objects.len(),
            borrowed_count: stats.borrowed_count,
            total_allocations: stats.total_allocations,
            pool_hits: stats.pool_hits,
            pool_misses: stats.pool_misses,
            cleanup_cycles: stats.cleanup_cycles,
        }
    }

    /// Warm up the pool with additional objects
    pub async fn warm(&self, additional_count: usize) {
        let mut objects = self.objects.lock().await;

        let target_size = (objects.len() + additional_count).min(self.config.max_capacity);
        while objects.len() < target_size {
            objects.push(T::default());
        }
    }

    /// Clean up weak references
    pub async fn cleanup_weak_refs(&self) {
        let mut weak_refs = self.weak_refs.lock().await;
        let mut stats = self.stats.lock().await;

        weak_refs.retain(|weak_ref| weak_ref.strong_count() > 0);
        stats.cleanup_cycles += 1;
    }

    /// Adjust pool size based on usage patterns
    pub async fn auto_adjust(&self) {
        if !self.config.auto_adjust {
            return;
        }

        let stats = self.stats.lock().await;
        let hit_rate = stats.hit_rate();

        // If hit rate is low, consider reducing pool size
        // If hit rate is high, consider increasing pool size
        let mut objects = self.objects.lock().await;

        if hit_rate < 50.0 && objects.len() > self.config.initial_capacity {
            // Reduce pool size
            let target_size = (objects.len() * 3 / 4).max(self.config.initial_capacity);
            objects.truncate(target_size);
        } else if hit_rate > self.config.warm_threshold * 100.0
            && objects.len() < self.config.max_capacity
        {
            // Increase pool size
            let additional =
                (self.config.initial_capacity / 2).min(self.config.max_capacity - objects.len());
            for _ in 0..additional {
                objects.push(T::default());
            }
        }
    }

    fn clone_pool_ref(&self) -> AsyncPoolRef<T> {
        AsyncPoolRef {
            objects: Arc::clone(&self.objects),
            stats: Arc::clone(&self.stats),
            config: self.config.clone(),
        }
    }
}

impl<T> Clone for AsyncPool<T> {
    fn clone(&self) -> Self {
        Self {
            objects: Arc::clone(&self.objects),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            weak_refs: Arc::clone(&self.weak_refs),
        }
    }
}

impl<T> fmt::Debug for AsyncPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncPool")
            .field("config", &self.config)
            .finish()
    }
}

/// Reference to a pool for returning objects
#[derive(Debug)]
struct AsyncPoolRef<T> {
    objects: Arc<Mutex<Vec<T>>>,
    stats: Arc<Mutex<PoolStats>>,
    config: PoolConfig,
}

/// A pooled object that automatically returns to pool when dropped
pub struct PooledObject<T>
where
    T: Clone + Send + Sync + 'static,
{
    object: Option<Arc<T>>,
    pool_ref: AsyncPoolRef<T>,
}

impl<T> PooledObject<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn new(object: Arc<T>, pool_ref: AsyncPoolRef<T>) -> Self {
        Self {
            object: Some(object),
            pool_ref,
        }
    }

    /// Get a reference to the pooled object
    pub fn as_ref(&self) -> Option<&T> {
        self.object.as_ref().map(|arc| arc.as_ref())
    }

    /// Clone the object (this creates a new Arc reference)
    pub fn clone_object(&self) -> Option<Arc<T>> {
        self.object.clone()
    }
}

impl<T> Drop for PooledObject<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if let Some(obj_arc) = self.object.take() {
            // Try to extract the object if we have the only reference
            if let Ok(obj) = Arc::try_unwrap(obj_arc) {
                let objects = Arc::clone(&self.pool_ref.objects);
                let stats = Arc::clone(&self.pool_ref.stats);
                let max_capacity = self.pool_ref.config.max_capacity;

                // Spawn a task to return the object to the pool
                tokio::spawn(async move {
                    let mut objects_guard = objects.lock().await;
                    let mut stats_guard = stats.lock().await;

                    if objects_guard.len() < max_capacity {
                        objects_guard.push(obj);
                    }

                    stats_guard.borrowed_count = stats_guard.borrowed_count.saturating_sub(1);
                });
            }
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
            .expect("Pooled object should always have a value")
    }
}

impl<T> fmt::Debug for PooledObject<T>
where
    T: fmt::Debug + Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.object {
            Some(obj) => write!(f, "PooledObject({obj:?})"),
            None => write!(f, "PooledObject(None)"),
        }
    }
}

/// Specialized pools for common FHIRPath objects
pub struct FhirPathPools {
    pub values: AsyncPool<Vec<FhirPathValue>>,
    pub expressions: AsyncPool<Vec<ExpressionNode>>,
    pub strings: AsyncPool<String>,
}

impl FhirPathPools {
    /// Create new pools with optimized configurations for FHIRPath usage
    pub fn new() -> Self {
        Self {
            values: AsyncPool::with_config(PoolConfig {
                initial_capacity: 64,
                max_capacity: 512,
                auto_adjust: true,
                warm_threshold: 0.75,
                cleanup_interval_secs: 30,
            }),
            expressions: AsyncPool::with_config(PoolConfig {
                initial_capacity: 32,
                max_capacity: 256,
                auto_adjust: true,
                warm_threshold: 0.8,
                cleanup_interval_secs: 60,
            }),
            strings: AsyncPool::with_config(PoolConfig {
                initial_capacity: 128,
                max_capacity: 1024,
                auto_adjust: true,
                warm_threshold: 0.7,
                cleanup_interval_secs: 45,
            }),
        }
    }

    /// Get comprehensive statistics for all pools
    pub async fn comprehensive_stats(&self) -> HashMap<String, PoolStats> {
        let mut stats = HashMap::new();

        stats.insert("values".to_string(), self.values.stats().await);
        stats.insert("expressions".to_string(), self.expressions.stats().await);
        stats.insert("strings".to_string(), self.strings.stats().await);

        stats
    }

    /// Warm all pools
    pub async fn warm_all(&self) {
        let (_, _, _) = tokio::join!(
            self.values.warm(32),
            self.expressions.warm(16),
            self.strings.warm(64)
        );
    }

    /// Auto-adjust all pools
    pub async fn auto_adjust_all(&self) {
        let (_, _, _) = tokio::join!(
            self.values.auto_adjust(),
            self.expressions.auto_adjust(),
            self.strings.auto_adjust()
        );
    }

    /// Clean up all pools
    pub async fn cleanup_all(&self) {
        let (_, _, _) = tokio::join!(
            self.values.cleanup_weak_refs(),
            self.expressions.cleanup_weak_refs(),
            self.strings.cleanup_weak_refs()
        );
    }
}

impl Default for FhirPathPools {
    fn default() -> Self {
        Self::new()
    }
}

/// Global pool manager for the entire FHIRPath pipeline
static GLOBAL_POOLS: Lazy<FhirPathPools> = Lazy::new(FhirPathPools::new);

/// Get access to the global memory pools
pub fn global_pools() -> &'static FhirPathPools {
    &GLOBAL_POOLS
}

/// Memory pool monitoring task
pub struct PoolMonitor {
    pools: FhirPathPools,
    monitoring_interval_secs: u64,
}

impl PoolMonitor {
    /// Create a new pool monitor
    pub fn new(pools: FhirPathPools, monitoring_interval_secs: u64) -> Self {
        Self {
            pools,
            monitoring_interval_secs,
        }
    }

    /// Start the monitoring task
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                self.monitoring_interval_secs,
            ));

            loop {
                interval.tick().await;

                // Auto-adjust pools based on usage
                self.pools.auto_adjust_all().await;

                // Clean up weak references
                self.pools.cleanup_all().await;

                // Optional: Log statistics
                #[cfg(feature = "diagnostics")]
                {
                    let stats = self.pools.comprehensive_stats().await;
                    log::debug!("Memory pool stats: {:?}", stats);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_async_pool_basic_operations() {
        let pool: AsyncPool<String> = AsyncPool::new();

        // Borrow an object
        let obj1 = pool.borrow().await;
        assert!(obj1.as_ref().is_some());

        let stats = pool.stats().await;
        assert_eq!(stats.borrowed_count, 1);
        assert_eq!(stats.total_allocations, 1);
    }

    #[test]
    async fn test_async_pool_return_to_pool() {
        let pool: AsyncPool<Vec<i32>> = AsyncPool::new();

        {
            let _obj = pool.borrow().await;
        } // obj should be returned to pool here

        // Give the async drop task time to complete
        tokio::task::yield_now().await;

        let stats = pool.stats().await;
        assert!(stats.pool_size > 0); // Object should be back in pool
    }

    #[test]
    async fn test_pool_warming() {
        let pool: AsyncPool<String> = AsyncPool::new();

        let initial_stats = pool.stats().await;
        let initial_size = initial_stats.pool_size;

        pool.warm(10).await;

        let warmed_stats = pool.stats().await;
        assert!(warmed_stats.pool_size >= initial_size + 10);
    }

    #[test]
    async fn test_fhirpath_pools_creation() {
        let pools = FhirPathPools::new();

        let stats = pools.comprehensive_stats().await;
        assert!(stats.contains_key("values"));
        assert!(stats.contains_key("expressions"));
        assert!(stats.contains_key("strings"));
    }

    #[test]
    async fn test_global_pools_access() {
        let pools = global_pools();
        let value_obj = pools.values.borrow().await;
        assert!(value_obj.as_ref().is_some());
    }

    #[test]
    async fn test_pool_stats_hit_rate() {
        let pool: AsyncPool<String> = AsyncPool::new();

        // Borrow and return to create a hit
        {
            let _obj = pool.borrow().await;
        }
        tokio::task::yield_now().await;

        // Borrow again - should be a hit
        let _obj2 = pool.borrow().await;

        let stats = pool.stats().await;
        assert!(stats.hit_rate() > 0.0);
    }

    #[test]
    async fn test_pool_monitor_creation() {
        let pools = FhirPathPools::new();
        let monitor = PoolMonitor::new(pools, 1);

        // Start monitoring (but cancel immediately)
        let handle = monitor.start();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        handle.abort();
    }
}
