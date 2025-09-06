//! AST Caching System for FHIRPath Engine
//!
//! This module provides AST caching capabilities for the FhirPathEngine to improve
//! performance by avoiding repeated parsing of frequently used expressions.

/// Cache statistics for AST caching system
///
/// Provides information about the current state of the AST cache,
/// useful for monitoring performance and tuning cache parameters.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath::evaluator::CacheStats;
///
/// let stats = CacheStats {
///     size: 150,
///     max_size: 1000,
///     enabled: true,
/// };
///
/// println!("Cache utilization: {:.1}%", stats.utilization_percentage());
/// println!("Cache efficiency: {}", stats.efficiency_description());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    /// Current number of cached AST entries
    pub size: usize,
    /// Maximum allowed cache size
    pub max_size: usize,
    /// Whether caching is enabled
    pub enabled: bool,
}

impl CacheStats {
    /// Create new cache statistics
    ///
    /// # Arguments
    /// * `size` - Current cache size
    /// * `max_size` - Maximum cache size
    /// * `enabled` - Whether caching is enabled
    pub fn new(size: usize, max_size: usize, enabled: bool) -> Self {
        Self {
            size,
            max_size,
            enabled,
        }
    }

    /// Calculate cache utilization as percentage
    ///
    /// Returns what percentage of the maximum cache size is currently being used.
    /// Higher utilization may indicate need for larger cache size.
    ///
    /// # Returns
    /// * `f64` - Utilization percentage (0.0-100.0)
    pub fn utilization_percentage(&self) -> f64 {
        if !self.enabled || self.max_size == 0 {
            0.0
        } else {
            (self.size as f64 / self.max_size as f64) * 100.0
        }
    }

    /// Check if cache is full
    ///
    /// Returns true if the cache has reached its maximum size and
    /// new entries will trigger eviction of old entries.
    ///
    /// # Returns
    /// * `bool` - True if cache is at maximum capacity
    pub fn is_full(&self) -> bool {
        self.enabled && self.size >= self.max_size
    }

    /// Check if cache is empty
    ///
    /// # Returns
    /// * `bool` - True if cache contains no entries
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Get available cache slots
    ///
    /// Returns the number of additional entries that can be cached
    /// before the cache reaches maximum capacity.
    ///
    /// # Returns
    /// * `usize` - Number of available cache slots
    pub fn available_slots(&self) -> usize {
        if !self.enabled {
            0
        } else {
            self.max_size.saturating_sub(self.size)
        }
    }

    /// Get efficiency description
    ///
    /// Returns a human-readable description of cache efficiency
    /// based on current utilization levels.
    ///
    /// # Returns
    /// * `&str` - Efficiency description
    pub fn efficiency_description(&self) -> &str {
        if !self.enabled {
            "Disabled"
        } else if self.max_size == 0 {
            "Disabled (size=0)"
        } else {
            let utilization = self.utilization_percentage();
            match utilization {
                0.0..=10.0 => "Very low utilization",
                10.0..=30.0 => "Low utilization",
                30.0..=60.0 => "Moderate utilization",
                60.0..=80.0 => "Good utilization",
                80.0..=95.0 => "High utilization",
                _ => "Near capacity",
            }
        }
    }

    /// Get recommendation for cache tuning
    ///
    /// Provides recommendations for optimizing cache performance
    /// based on current statistics.
    ///
    /// # Returns
    /// * `Option<String>` - Optimization recommendation if applicable
    pub fn optimization_recommendation(&self) -> Option<String> {
        if !self.enabled {
            return Some("Consider enabling AST cache for better performance".to_string());
        }

        if self.max_size == 0 {
            return Some(
                "Cache size is 0 - increase max_cache_size for performance benefits".to_string(),
            );
        }

        let utilization = self.utilization_percentage();
        match utilization {
            0.0..=5.0 => Some(
                "Cache utilization is very low - consider reducing cache size to save memory"
                    .to_string(),
            ),
            95.0..=100.0 => Some(
                "Cache is near capacity - consider increasing max_cache_size for better hit rates"
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Format statistics for display
    ///
    /// Creates a formatted string with cache statistics suitable
    /// for logging or debugging output.
    ///
    /// # Returns
    /// * `String` - Formatted statistics summary
    pub fn format_summary(&self) -> String {
        if !self.enabled {
            "AST Cache: Disabled".to_string()
        } else {
            format!(
                "AST Cache: {}/{} entries ({:.1}% full) - {}",
                self.size,
                self.max_size,
                self.utilization_percentage(),
                self.efficiency_description()
            )
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            size: 0,
            max_size: 1000,
            enabled: true,
        }
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_summary())
    }
}

/// Cache performance metrics for evaluation
///
/// Tracks hit/miss ratios and other performance indicators
/// for the AST cache system over time.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath::evaluator::CacheMetrics;
///
/// let mut metrics = CacheMetrics::new();
/// metrics.record_hit();
/// metrics.record_hit();
/// metrics.record_miss();
///
/// println!("Hit rate: {:.1}%", metrics.hit_rate_percentage());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CacheMetrics {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Total number of evictions due to capacity
    pub evictions: u64,
    /// Total time saved by cache hits (microseconds)
    pub time_saved_us: u64,
}

impl CacheMetrics {
    /// Create new cache metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit
    ///
    /// # Arguments
    /// * `time_saved_us` - Microseconds saved by avoiding parsing
    pub fn record_hit(&mut self, time_saved_us: Option<u64>) {
        self.hits += 1;
        if let Some(time) = time_saved_us {
            self.time_saved_us += time;
        }
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record a cache eviction
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Get total number of cache accesses
    pub fn total_accesses(&self) -> u64 {
        self.hits + self.misses
    }

    /// Calculate hit rate as percentage
    ///
    /// # Returns
    /// * `f64` - Hit rate percentage (0.0-100.0)
    pub fn hit_rate_percentage(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Calculate miss rate as percentage
    ///
    /// # Returns
    /// * `f64` - Miss rate percentage (0.0-100.0)
    pub fn miss_rate_percentage(&self) -> f64 {
        100.0 - self.hit_rate_percentage()
    }

    /// Get average time saved per hit
    ///
    /// # Returns
    /// * `f64` - Average microseconds saved per cache hit
    pub fn avg_time_saved_per_hit_us(&self) -> f64 {
        if self.hits == 0 {
            0.0
        } else {
            self.time_saved_us as f64 / self.hits as f64
        }
    }

    /// Get cache efficiency rating
    ///
    /// Returns a qualitative assessment of cache efficiency
    /// based on hit rates and eviction frequency.
    ///
    /// # Returns
    /// * `CacheEfficiency` - Efficiency rating
    pub fn efficiency_rating(&self) -> CacheEfficiency {
        let hit_rate = self.hit_rate_percentage();
        let total_accesses = self.total_accesses();

        if total_accesses == 0 {
            return CacheEfficiency::Unknown;
        }

        let eviction_rate = if total_accesses > 0 {
            (self.evictions as f64 / total_accesses as f64) * 100.0
        } else {
            0.0
        };

        match (hit_rate, eviction_rate) {
            (hit, _) if hit >= 90.0 => CacheEfficiency::Excellent,
            (hit, evict) if hit >= 70.0 && evict < 20.0 => CacheEfficiency::Good,
            (hit, evict) if hit >= 50.0 && evict < 40.0 => CacheEfficiency::Fair,
            _ => CacheEfficiency::Poor,
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.time_saved_us = 0;
    }

    /// Format metrics for display
    pub fn format_summary(&self) -> String {
        let total = self.total_accesses();
        if total == 0 {
            "Cache Metrics: No accesses recorded".to_string()
        } else {
            format!(
                "Cache Metrics: {}/{} hits ({:.1}%), {} evictions, {:.0}μs avg saved",
                self.hits,
                total,
                self.hit_rate_percentage(),
                self.evictions,
                self.avg_time_saved_per_hit_us()
            )
        }
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            time_saved_us: 0,
        }
    }
}

impl std::fmt::Display for CacheMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_summary())
    }
}

/// Cache efficiency rating
///
/// Qualitative assessment of cache performance based on
/// hit rates and eviction patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheEfficiency {
    /// Unknown efficiency (no data)
    Unknown,
    /// Excellent efficiency (>90% hit rate, low evictions)
    Excellent,
    /// Good efficiency (>70% hit rate, moderate evictions)
    Good,
    /// Fair efficiency (>50% hit rate, acceptable evictions)
    Fair,
    /// Poor efficiency (<50% hit rate or high evictions)
    Poor,
}

impl std::fmt::Display for CacheEfficiency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheEfficiency::Unknown => write!(f, "Unknown"),
            CacheEfficiency::Excellent => write!(f, "Excellent"),
            CacheEfficiency::Good => write!(f, "Good"),
            CacheEfficiency::Fair => write!(f, "Fair"),
            CacheEfficiency::Poor => write!(f, "Poor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats::new(50, 100, true);
        assert_eq!(stats.size, 50);
        assert_eq!(stats.max_size, 100);
        assert!(stats.enabled);
    }

    #[test]
    fn test_utilization_percentage() {
        let stats = CacheStats::new(25, 100, true);
        assert_eq!(stats.utilization_percentage(), 25.0);

        let disabled_stats = CacheStats::new(25, 100, false);
        assert_eq!(disabled_stats.utilization_percentage(), 0.0);
    }

    #[test]
    fn test_cache_full_empty() {
        let full_stats = CacheStats::new(100, 100, true);
        assert!(full_stats.is_full());
        assert!(!full_stats.is_empty());

        let empty_stats = CacheStats::new(0, 100, true);
        assert!(!empty_stats.is_full());
        assert!(empty_stats.is_empty());

        let disabled_stats = CacheStats::new(100, 100, false);
        assert!(!disabled_stats.is_full()); // Disabled cache is never "full"
    }

    #[test]
    fn test_available_slots() {
        let stats = CacheStats::new(30, 100, true);
        assert_eq!(stats.available_slots(), 70);

        let full_stats = CacheStats::new(100, 100, true);
        assert_eq!(full_stats.available_slots(), 0);

        let disabled_stats = CacheStats::new(30, 100, false);
        assert_eq!(disabled_stats.available_slots(), 0);
    }

    #[test]
    fn test_efficiency_description() {
        let low_util = CacheStats::new(5, 100, true);
        assert_eq!(low_util.efficiency_description(), "Very low utilization");

        let good_util = CacheStats::new(70, 100, true);
        assert_eq!(good_util.efficiency_description(), "Good utilization");

        let near_capacity = CacheStats::new(98, 100, true);
        assert_eq!(near_capacity.efficiency_description(), "Near capacity");

        let disabled = CacheStats::new(50, 100, false);
        assert_eq!(disabled.efficiency_description(), "Disabled");
    }

    #[test]
    fn test_optimization_recommendations() {
        let disabled = CacheStats::new(50, 100, false);
        let rec = disabled.optimization_recommendation();
        assert!(rec.is_some());
        assert!(rec.unwrap().contains("enabling AST cache"));

        let zero_size = CacheStats::new(0, 0, true);
        let rec = zero_size.optimization_recommendation();
        assert!(rec.is_some());
        assert!(rec.unwrap().contains("increase max_cache_size"));

        let very_low = CacheStats::new(2, 100, true);
        let rec = very_low.optimization_recommendation();
        assert!(rec.is_some());
        assert!(rec.unwrap().contains("reducing cache size"));

        let near_full = CacheStats::new(97, 100, true);
        let rec = near_full.optimization_recommendation();
        assert!(rec.is_some());
        assert!(rec.unwrap().contains("increasing max_cache_size"));

        let optimal = CacheStats::new(50, 100, true);
        let rec = optimal.optimization_recommendation();
        assert!(rec.is_none());
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = CacheMetrics::new();

        metrics.record_hit(Some(100));
        metrics.record_hit(Some(150));
        metrics.record_miss();
        metrics.record_eviction();

        assert_eq!(metrics.hits, 2);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.evictions, 1);
        assert_eq!(metrics.total_accesses(), 3);
        assert_eq!(metrics.time_saved_us, 250);

        // Hit rate should be 2/3 = 66.67%
        assert!((metrics.hit_rate_percentage() - 66.67).abs() < 0.01);

        // Miss rate should be 1/3 = 33.33%
        assert!((metrics.miss_rate_percentage() - 33.33).abs() < 0.01);

        // Average time saved should be 250/2 = 125μs
        assert_eq!(metrics.avg_time_saved_per_hit_us(), 125.0);
    }

    #[test]
    fn test_efficiency_ratings() {
        let mut excellent = CacheMetrics::new();
        for _ in 0..95 {
            excellent.record_hit(None);
        }
        for _ in 0..5 {
            excellent.record_miss();
        }
        assert_eq!(excellent.efficiency_rating(), CacheEfficiency::Excellent);

        let mut good = CacheMetrics::new();
        for _ in 0..75 {
            good.record_hit(None);
        }
        for _ in 0..25 {
            good.record_miss();
        }
        assert_eq!(good.efficiency_rating(), CacheEfficiency::Good);

        let mut poor = CacheMetrics::new();
        for _ in 0..30 {
            poor.record_hit(None);
        }
        for _ in 0..70 {
            poor.record_miss();
        }
        assert_eq!(poor.efficiency_rating(), CacheEfficiency::Poor);

        let unknown = CacheMetrics::new();
        assert_eq!(unknown.efficiency_rating(), CacheEfficiency::Unknown);
    }

    #[test]
    fn test_metrics_reset() {
        let mut metrics = CacheMetrics::new();
        metrics.record_hit(Some(100));
        metrics.record_miss();
        metrics.record_eviction();

        assert!(metrics.total_accesses() > 0);

        metrics.reset();
        assert_eq!(metrics.total_accesses(), 0);
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
        assert_eq!(metrics.evictions, 0);
        assert_eq!(metrics.time_saved_us, 0);
    }

    #[test]
    fn test_format_summary() {
        let stats = CacheStats::new(150, 1000, true);
        let summary = stats.format_summary();
        assert!(summary.contains("150/1000"));
        assert!(summary.contains("15.0%"));

        let disabled = CacheStats::new(0, 1000, false);
        let summary = disabled.format_summary();
        assert!(summary.contains("Disabled"));

        let mut metrics = CacheMetrics::new();
        metrics.record_hit(Some(100));
        metrics.record_miss();
        let summary = metrics.format_summary();
        assert!(summary.contains("1/2"));
        assert!(summary.contains("50.0%"));
    }
}
