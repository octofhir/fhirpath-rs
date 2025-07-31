//! Cache statistics collection

use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics for cache performance monitoring
#[derive(Debug, Default)]
pub struct CacheStatistics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl CacheStatistics {
    /// Create new cache statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get number of cache hits
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get number of cache misses
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get number of cache evictions
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get total number of cache accesses
    pub fn total_accesses(&self) -> u64 {
        self.hits() + self.misses()
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 {
            0.0
        } else {
            self.hits() as f64 / total as f64
        }
    }

    /// Reset all statistics to zero
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Get a summary string of cache statistics
    pub fn summary(&self) -> String {
        format!(
            "Cache Stats: hits={}, misses={}, evictions={}, hit_rate={:.2}%",
            self.hits(),
            self.misses(),
            self.evictions(),
            self.hit_rate() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_statistics_basic() {
        let stats = CacheStatistics::new();
        
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.evictions(), 0);
        assert_eq!(stats.total_accesses(), 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_statistics_recording() {
        let stats = CacheStatistics::new();
        
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        
        assert_eq!(stats.hits(), 2);
        assert_eq!(stats.misses(), 1);
        assert_eq!(stats.evictions(), 1);
        assert_eq!(stats.total_accesses(), 3);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_cache_statistics_reset() {
        let stats = CacheStatistics::new();
        
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        
        stats.reset();
        
        assert_eq!(stats.hits(), 0);
        assert_eq!(stats.misses(), 0);
        assert_eq!(stats.evictions(), 0);
    }

    #[test]
    fn test_cache_statistics_summary() {
        let stats = CacheStatistics::new();
        
        stats.record_hit();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        
        let summary = stats.summary();
        assert!(summary.contains("hits=3"));
        assert!(summary.contains("misses=1"));
        assert!(summary.contains("hit_rate=75.00%"));
    }

    #[test]
    fn test_cache_statistics_thread_safety() {
        use std::sync::Arc;
        use std::thread;
        
        let stats = Arc::new(CacheStatistics::new());
        let mut handles = vec![];
        
        // Spawn multiple threads that record hits
        for _ in 0..10 {
            let stats_clone = Arc::clone(&stats);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    stats_clone.record_hit();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(stats.hits(), 10_000);
    }
}