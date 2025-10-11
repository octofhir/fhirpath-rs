//! Performance metrics tracking for the LSP server
//!
//! This module provides metrics collection for various LSP operations
//! to help identify performance bottlenecks and track optimization efforts.

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics for LSP operations
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// Time spent generating diagnostics
    pub diagnostic_time: Duration,
    /// Time spent generating completions
    pub completion_time: Duration,
    /// Time spent on hover requests
    pub hover_time: Duration,
    /// Time spent on semantic tokens
    pub semantic_tokens_time: Duration,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Total number of documents processed
    pub documents_processed: usize,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record diagnostic generation time
    pub fn record_diagnostic(&mut self, duration: Duration) {
        self.diagnostic_time = duration;
    }

    /// Record completion generation time
    pub fn record_completion(&mut self, duration: Duration) {
        self.completion_time = duration;
    }

    /// Record hover generation time
    pub fn record_hover(&mut self, duration: Duration) {
        self.hover_time = duration;
    }

    /// Record semantic tokens generation time
    pub fn record_semantic_tokens(&mut self, duration: Duration) {
        self.semantic_tokens_time = duration;
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Increment document counter
    pub fn record_document_processed(&mut self) {
        self.documents_processed += 1;
    }

    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Get average diagnostic time (if any diagnostics were run)
    pub fn avg_diagnostic_time(&self) -> Option<Duration> {
        if self.documents_processed > 0 {
            Some(self.diagnostic_time / self.documents_processed as u32)
        } else {
            None
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get formatted summary string
    pub fn summary(&self) -> String {
        format!(
            "Metrics: docs={}, cache_hit_rate={:.1}%, diag={:?}, compl={:?}, hover={:?}",
            self.documents_processed,
            self.cache_hit_rate() * 100.0,
            self.diagnostic_time,
            self.completion_time,
            self.hover_time
        )
    }
}

/// Thread-safe metrics tracker
#[derive(Debug, Clone)]
pub struct MetricsTracker {
    metrics: Arc<RwLock<Metrics>>,
}

impl MetricsTracker {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::new())),
        }
    }

    /// Execute an operation and record its duration
    pub fn track<F, R>(&self, operation: MetricOperation, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        let mut metrics = self.metrics.write();
        match operation {
            MetricOperation::Diagnostic => metrics.record_diagnostic(duration),
            MetricOperation::Completion => metrics.record_completion(duration),
            MetricOperation::Hover => metrics.record_hover(duration),
            MetricOperation::SemanticTokens => metrics.record_semantic_tokens(duration),
        }

        result
    }

    /// Record cache hit
    pub fn cache_hit(&self) {
        self.metrics.write().record_cache_hit();
    }

    /// Record cache miss
    pub fn cache_miss(&self) {
        self.metrics.write().record_cache_miss();
    }

    /// Record document processed
    pub fn document_processed(&self) {
        self.metrics.write().record_document_processed();
    }

    /// Get snapshot of current metrics
    pub fn snapshot(&self) -> Metrics {
        self.metrics.read().clone()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.metrics.write().reset();
    }

    /// Get formatted summary
    pub fn summary(&self) -> String {
        self.metrics.read().summary()
    }
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of metric operation being tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricOperation {
    /// Diagnostic generation operation
    Diagnostic,
    /// Completion generation operation
    Completion,
    /// Hover information operation
    Hover,
    /// Semantic tokens generation operation
    SemanticTokens,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.documents_processed, 0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut metrics = Metrics::new();
        assert_eq!(metrics.cache_hit_rate(), 0.0);

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        // 2 hits out of 3 total = 66.67%
        assert!((metrics.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_record_operations() {
        let mut metrics = Metrics::new();

        metrics.record_diagnostic(Duration::from_millis(100));
        metrics.record_completion(Duration::from_millis(50));
        metrics.record_hover(Duration::from_millis(25));

        assert_eq!(metrics.diagnostic_time, Duration::from_millis(100));
        assert_eq!(metrics.completion_time, Duration::from_millis(50));
        assert_eq!(metrics.hover_time, Duration::from_millis(25));
    }

    #[test]
    fn test_metrics_reset() {
        let mut metrics = Metrics::new();
        metrics.record_cache_hit();
        metrics.record_document_processed();

        metrics.reset();

        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.documents_processed, 0);
    }

    #[test]
    fn test_metrics_tracker() {
        let tracker = MetricsTracker::new();

        // Track an operation
        let result = tracker.track(MetricOperation::Diagnostic, || {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);

        let snapshot = tracker.snapshot();
        assert!(snapshot.diagnostic_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_metrics_tracker_cache() {
        let tracker = MetricsTracker::new();

        tracker.cache_hit();
        tracker.cache_hit();
        tracker.cache_miss();

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert!((snapshot.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_summary_string() {
        let mut metrics = Metrics::new();
        metrics.record_diagnostic(Duration::from_millis(100));
        metrics.record_document_processed();
        metrics.record_cache_hit();

        let summary = metrics.summary();
        assert!(summary.contains("docs=1"));
        assert!(summary.contains("cache_hit_rate=100.0%"));
    }
}
