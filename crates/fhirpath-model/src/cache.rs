use crate::{
    legacy_cache::{CacheConfig as LegacyCacheConfig, TypeCache},
    provider::TypeReflectionInfo,
};
use crossbeam::epoch::{self, Atomic, Owned, Shared};
use dashmap::DashMap;
use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::time;

/// High-performance multi-tier caching system
#[derive(Debug)]
pub struct CacheManager {
    /// L1: Lock-free hot cache for most frequently accessed types
    hot_cache: Arc<LockFreeCache<String, Arc<TypeReflectionInfo>>>,

    /// L2: Warm cache with medium access frequency
    warm_cache: Arc<DashMap<String, WarmCacheEntry>>,

    /// L3: Cold storage for less frequent access
    cold_storage: Arc<TypeCache<TypeReflectionInfo>>,

    /// Access pattern tracker for cache promotion/demotion
    pub access_tracker: Arc<AccessPatternTracker>,

    /// Cache configuration and thresholds
    config: CacheConfig,

    /// Performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Hot cache size (highest frequency items)
    pub hot_cache_size: usize,

    /// Warm cache size (medium frequency items)
    pub warm_cache_size: usize,

    /// Cold cache size (low frequency items)
    pub cold_cache_size: usize,

    /// Promotion threshold (accesses needed to move up tier)
    pub promotion_threshold: u32,

    /// Demotion threshold (age without access for demotion)
    pub demotion_threshold: Duration,

    /// Background cleanup interval
    pub cleanup_interval: Duration,

    /// Enable predictive caching
    pub enable_predictive: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            hot_cache_size: 100,
            warm_cache_size: 500,
            cold_cache_size: 2000,
            promotion_threshold: 10,
            demotion_threshold: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60),    // 1 minute
            enable_predictive: true,
        }
    }
}

#[derive(Debug)]
struct WarmCacheEntry {
    value: Arc<TypeReflectionInfo>,
    access_count: AtomicU32,
    last_accessed: AtomicU64, // Timestamp
}

#[derive(Debug, Clone)]
pub struct CacheMetrics {
    /// Hot cache statistics
    pub hot_stats: TierStats,
    /// Warm cache statistics  
    pub warm_stats: TierStats,
    /// Cold cache statistics
    pub cold_stats: TierStats,
    /// Cache promotions (cold->warm->hot)
    pub promotions: u64,
    /// Cache demotions (hot->warm->cold)
    pub demotions: u64,
    /// Predictive cache hits
    pub predictive_hits: u64,
    /// Overall hit ratio
    pub overall_hit_ratio: f64,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            hot_stats: TierStats::default(),
            warm_stats: TierStats::default(),
            cold_stats: TierStats::default(),
            promotions: 0,
            demotions: 0,
            predictive_hits: 0,
            overall_hit_ratio: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TierStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub evictions: u64,
}

/// Lock-free cache for hot path performance using crossbeam
#[derive(Debug)]
pub struct LockFreeCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// Atomic hash table using epoch-based memory management
    table: Atomic<HashMap<K, V>>,
    /// Maximum capacity
    capacity: usize,
    /// Current size (atomic)
    size: AtomicUsize,
}

impl<K, V> LockFreeCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// Create new lock-free cache
    pub fn new(capacity: usize) -> Self {
        let table = HashMap::with_capacity(capacity);
        Self {
            table: Atomic::from(Owned::new(table)),
            capacity,
            size: AtomicUsize::new(0),
        }
    }

    /// Get value without any locks
    #[inline]
    pub fn get(&self, key: &K) -> Option<V> {
        let guard = &epoch::pin();
        let table = self.table.load(Ordering::Acquire, guard);

        if table.is_null() {
            return None;
        }

        unsafe { table.deref().get(key).cloned() }
    }

    /// Insert value with lock-free operation
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let guard = &epoch::pin();

        // For simplicity, we'll use a basic approach that occasionally locks
        // A full lock-free implementation would be much more complex
        loop {
            let current_table = self.table.load(Ordering::Acquire, guard);
            let mut new_table = if current_table.is_null() {
                HashMap::with_capacity(self.capacity)
            } else {
                unsafe { current_table.deref().clone() }
            };

            let old_value = new_table.insert(key.clone(), value.clone());

            // Try to swap the new table
            let new_owned = Owned::new(new_table);
            match self.table.compare_exchange(
                current_table,
                new_owned,
                Ordering::AcqRel,
                Ordering::Acquire,
                guard,
            ) {
                Ok(_) => {
                    if old_value.is_none() {
                        self.size.fetch_add(1, Ordering::Relaxed);
                    }

                    // Defer destruction of old table
                    if !current_table.is_null() {
                        unsafe {
                            guard.defer_destroy(current_table);
                        }
                    }

                    return old_value;
                }
                Err(new_owned_error) => {
                    // Retry with updated table
                    drop(new_owned_error.new.into_box());
                    continue;
                }
            }
        }
    }

    /// Clear cache atomically
    pub fn clear(&self) {
        let guard = &epoch::pin();
        let old_table = self.table.swap(Shared::null(), Ordering::AcqRel, guard);

        if !old_table.is_null() {
            unsafe {
                guard.defer_destroy(old_table);
            }
        }

        self.size.store(0, Ordering::Relaxed);
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Tracks access patterns for intelligent caching decisions
#[derive(Debug)]
pub struct AccessPatternTracker {
    /// Access frequency tracking
    access_counts: DashMap<String, AtomicU32>,

    /// Recent access history (ring buffer)
    recent_accesses: Arc<RwLock<Vec<AccessRecord>>>,

    /// Type relationship predictions
    relationship_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Configuration
    config: PatternConfig,

    /// Ring buffer position
    ring_position: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct PatternConfig {
    /// Maximum size of recent access ring buffer
    pub ring_buffer_size: usize,
    /// Time window for relationship detection
    pub relationship_window: Duration,
    /// Minimum frequency for hot tier
    pub hot_frequency_threshold: u32,
    /// Minimum frequency for warm tier
    pub warm_frequency_threshold: u32,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            ring_buffer_size: 1000,
            relationship_window: Duration::from_secs(10),
            hot_frequency_threshold: 100,
            warm_frequency_threshold: 20,
        }
    }
}

#[derive(Debug, Clone)]
struct AccessRecord {
    type_name: String,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum AccessSource {
    TypeReflection,
    PropertyLookup,
    InheritanceCheck,
    PolymorphicResolution,
}

#[derive(Debug, Clone, Copy)]
pub enum CacheTier {
    Hot,
    Warm,
    Cold,
}

impl Default for AccessPatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessPatternTracker {
    /// Create new access pattern tracker
    pub fn new() -> Self {
        Self::with_config(PatternConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PatternConfig) -> Self {
        Self {
            access_counts: DashMap::new(),
            recent_accesses: Arc::new(RwLock::new(Vec::with_capacity(config.ring_buffer_size))),
            relationship_graph: Arc::new(RwLock::new(HashMap::new())),
            config,
            ring_position: AtomicUsize::new(0),
        }
    }

    /// Record an access and update patterns
    pub fn record_access(&self, type_name: &str, _source: AccessSource) {
        // Update frequency counter
        self.access_counts
            .entry(type_name.to_string())
            .or_insert_with(|| AtomicU32::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Add to recent history using ring buffer
        let record = AccessRecord {
            type_name: type_name.to_string(),
            timestamp: Instant::now(),
        };

        {
            let mut recent = self.recent_accesses.write().unwrap();
            let pos =
                self.ring_position.fetch_add(1, Ordering::Relaxed) % self.config.ring_buffer_size;

            if pos < recent.len() {
                recent[pos] = record;
            } else {
                recent.push(record);
            }
        }

        // Update relationship predictions
        self.update_relationship_predictions(type_name);
    }

    /// Get access frequency for a type
    pub fn get_access_frequency(&self, type_name: &str) -> u32 {
        self.access_counts
            .get(type_name)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Predict related types that might be accessed
    pub fn predict_related_types(&self, type_name: &str) -> Vec<String> {
        let graph = self.relationship_graph.read().unwrap();
        graph.get(type_name).cloned().unwrap_or_default()
    }

    /// Determine appropriate cache tier for type
    pub fn recommend_cache_tier(&self, type_name: &str) -> CacheTier {
        let frequency = self.get_access_frequency(type_name);

        if frequency >= self.config.hot_frequency_threshold {
            CacheTier::Hot
        } else if frequency >= self.config.warm_frequency_threshold {
            CacheTier::Warm
        } else {
            CacheTier::Cold
        }
    }

    /// Update relationship graph based on access patterns
    fn update_relationship_predictions(&self, type_name: &str) {
        let recent = self.recent_accesses.read().unwrap();
        let mut relationships = Vec::new();

        // Look for types accessed within time window
        let cutoff = Instant::now() - self.config.relationship_window;
        for record in recent.iter().rev() {
            if record.timestamp < cutoff {
                break;
            }
            if record.type_name != type_name {
                relationships.push(record.type_name.clone());
            }
        }

        if !relationships.is_empty() {
            relationships.sort();
            relationships.dedup();

            let mut graph = self.relationship_graph.write().unwrap();
            graph.insert(type_name.to_string(), relationships);
        }
    }
}

impl CacheManager {
    /// Create new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            hot_cache: Arc::new(LockFreeCache::new(config.hot_cache_size)),
            warm_cache: Arc::new(DashMap::new()),
            cold_storage: Arc::new(TypeCache::with_config(LegacyCacheConfig {
                max_size: config.cold_cache_size,
                ttl: Duration::from_secs(3600), // 1 hour TTL for cold storage
                enable_stats: true,
            })),
            access_tracker: Arc::new(AccessPatternTracker::new()),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get value with intelligent tier management
    pub fn get(&self, key: &str) -> Option<Arc<TypeReflectionInfo>> {
        // Track this access
        self.access_tracker
            .record_access(key, AccessSource::TypeReflection);

        // Try hot cache first (lock-free)
        if let Some(value) = self.hot_cache.get(&key.to_string()) {
            self.record_hit(CacheTier::Hot);
            return Some(value);
        }

        // Try warm cache
        if let Some(entry) = self.warm_cache.get_mut(key) {
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            entry.last_accessed.store(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                Ordering::Relaxed,
            );

            let value = entry.value.clone();
            self.record_hit(CacheTier::Warm);

            // Consider promotion to hot cache
            if entry.access_count.load(Ordering::Relaxed) > self.config.promotion_threshold {
                self.promote_to_hot(key, &value);
            }

            return Some(value);
        }

        // Try cold storage
        if let Some(value) = self.cold_storage.get(key) {
            let arc_value = Arc::new(value);
            self.record_hit(CacheTier::Cold);

            // Consider promotion to warm cache
            self.promote_to_warm(key, &arc_value);

            return Some(arc_value);
        }

        // Cache miss - record and potentially trigger predictive loading
        self.record_miss();
        if self.config.enable_predictive {
            self.trigger_predictive_loading(key);
        }

        None
    }

    /// Put value with intelligent tier placement
    pub fn put(&self, key: String, value: Arc<TypeReflectionInfo>) {
        // Determine appropriate tier based on access patterns
        let tier = self.access_tracker.recommend_cache_tier(&key);

        match tier {
            CacheTier::Hot => {
                self.hot_cache.insert(key, value);
            }
            CacheTier::Warm => {
                self.put_warm(key, value);
            }
            CacheTier::Cold => {
                // Try to unwrap Arc for cold storage
                match Arc::try_unwrap(value) {
                    Ok(unwrapped) => {
                        self.cold_storage.put(key, unwrapped);
                    }
                    Err(value_arc) => {
                        // If we can't unwrap, put in warm cache
                        self.put_warm(key, value_arc);
                    }
                }
            }
        }
    }

    /// Put value in warm cache
    fn put_warm(&self, key: String, value: Arc<TypeReflectionInfo>) {
        let entry = WarmCacheEntry {
            value,
            access_count: AtomicU32::new(1),
            last_accessed: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        };

        // Evict if necessary
        if self.warm_cache.len() >= self.config.warm_cache_size {
            self.evict_warm_lru();
        }

        self.warm_cache.insert(key, entry);
    }

    /// Promote value to hot cache
    fn promote_to_hot(&self, key: &str, value: &Arc<TypeReflectionInfo>) {
        self.hot_cache.insert(key.to_string(), value.clone());

        // Remove from warm cache
        self.warm_cache.remove(key);

        // Update metrics
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.promotions += 1;
        }
    }

    /// Promote value to warm cache
    fn promote_to_warm(&self, key: &str, value: &Arc<TypeReflectionInfo>) {
        self.put_warm(key.to_string(), value.clone());

        // Remove from cold storage
        self.cold_storage.remove(key);

        // Update metrics
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.promotions += 1;
        }
    }

    /// Trigger predictive loading of related types
    fn trigger_predictive_loading(&self, accessed_key: &str) {
        let related_types = self.access_tracker.predict_related_types(accessed_key);

        for related_type in related_types {
            // Check if already cached
            if self.get(&related_type).is_none() {
                // Trigger background loading (implementation depends on integration)
                self.request_background_load(related_type);
            }
        }
    }

    /// Request background loading of type
    fn request_background_load(&self, type_name: String) {
        // This would integrate with the ModelProvider's background loading system
        // For now, just record the request
        log::debug!("Requesting background load for type: {type_name}");
    }

    /// Evict least recently used entry from warm cache
    fn evict_warm_lru(&self) {
        let mut oldest_key = None;
        let mut oldest_time = u64::MAX;

        for entry in self.warm_cache.iter() {
            let last_accessed = entry.last_accessed.load(Ordering::Relaxed);
            if last_accessed < oldest_time {
                oldest_time = last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            if let Some((_, entry)) = self.warm_cache.remove(&key) {
                // Demote to cold storage
                if let Ok(value) = Arc::try_unwrap(entry.value) {
                    self.cold_storage.put(key, value);
                }
            }
        }
    }

    /// Record cache hit for metrics
    fn record_hit(&self, tier: CacheTier) {
        if let Ok(mut metrics) = self.metrics.write() {
            match tier {
                CacheTier::Hot => metrics.hot_stats.hits += 1,
                CacheTier::Warm => metrics.warm_stats.hits += 1,
                CacheTier::Cold => metrics.cold_stats.hits += 1,
            }
        }
    }

    /// Record cache miss for metrics
    fn record_miss(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.cold_stats.misses += 1;
        }
    }

    /// Get comprehensive cache statistics
    pub fn get_comprehensive_stats(&self) -> CacheMetrics {
        let stats_guard = self.metrics.read().unwrap();
        let mut stats = (*stats_guard).clone();

        // Update current sizes
        stats.hot_stats.size = self.hot_cache.len() as u64;
        stats.warm_stats.size = self.warm_cache.len() as u64;
        stats.cold_stats.size = self.cold_storage.len() as u64;

        // Calculate overall hit ratio
        let total_hits = stats.hot_stats.hits + stats.warm_stats.hits + stats.cold_stats.hits;
        let total_misses =
            stats.hot_stats.misses + stats.warm_stats.misses + stats.cold_stats.misses;
        let total_requests = total_hits + total_misses;

        if total_requests > 0 {
            stats.overall_hit_ratio = total_hits as f64 / total_requests as f64;
        }

        stats
    }

    /// Background maintenance task
    pub async fn run_maintenance(&self) {
        let mut interval = time::interval(self.config.cleanup_interval);

        loop {
            interval.tick().await;

            // Clean up expired entries
            self.cleanup_expired();

            // Rebalance tiers based on access patterns
            self.rebalance_tiers();

            // Update metrics
            self.update_metrics();
        }
    }

    /// Clean up expired entries across all tiers
    pub fn cleanup_expired(&self) {
        // Clean cold storage (has TTL)
        self.cold_storage.cleanup_expired();

        // Clean warm cache based on age and access patterns
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.config.demotion_threshold.as_secs();

        let expired_keys: Vec<String> = self
            .warm_cache
            .iter()
            .filter_map(|entry| {
                if entry.last_accessed.load(Ordering::Relaxed) < cutoff_time {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            if let Some((_, entry)) = self.warm_cache.remove(&key) {
                // Demote to cold storage
                if let Ok(value) = Arc::try_unwrap(entry.value) {
                    self.cold_storage.put(key, value);
                }

                // Update metrics
                if let Ok(mut metrics) = self.metrics.write() {
                    metrics.demotions += 1;
                }
            }
        }
    }

    /// Rebalance cache tiers based on access patterns
    fn rebalance_tiers(&self) {
        // This is a placeholder for more sophisticated rebalancing logic
        // In a full implementation, this would analyze access patterns and
        // move entries between tiers as needed
        log::debug!("Running cache tier rebalancing");
    }

    /// Update performance metrics
    fn update_metrics(&self) {
        // Update size metrics and other computed values
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.hot_stats.size = self.hot_cache.len() as u64;
            metrics.warm_stats.size = self.warm_cache.len() as u64;
            metrics.cold_stats.size = self.cold_storage.len() as u64;
        }
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.hot_cache.clear();
        self.warm_cache.clear();
        self.cold_storage.clear();

        // Reset metrics
        if let Ok(mut metrics) = self.metrics.write() {
            *metrics = CacheMetrics::default();
        }
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            hot_cache: self.hot_cache.clone(),
            warm_cache: self.warm_cache.clone(),
            cold_storage: self.cold_storage.clone(),
            access_tracker: self.access_tracker.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
}
