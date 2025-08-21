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

//! Metrics tracking for background schema loading

use parking_lot::RwLock;
use std::time::{Duration, Instant};

/// Comprehensive metrics for background schema loading
#[derive(Debug, Default)]
pub struct LoadingMetrics {
    /// Time taken to load essential types
    pub essential_load_time: Duration,

    /// Total number of schemas loaded successfully
    pub total_loaded: u32,

    /// Number of schema loading failures
    pub load_failures: u32,

    /// Number of cache hits (avoiding loads)
    pub cache_hits: u32,

    /// Number of background loads completed
    pub background_loads: u32,

    /// Number of predictive loads completed
    pub predictive_loads: u32,

    /// Average time per schema load
    pub average_load_time: Duration,

    /// Peak queue length observed
    pub peak_queue_length: usize,

    /// Peak concurrent workers active
    pub peak_concurrent_workers: usize,

    /// Number of retries performed
    pub retry_count: u32,

    /// Time when metrics collection started
    pub started_at: Option<Instant>,
}

impl LoadingMetrics {
    /// Create new metrics with current timestamp
    pub fn new() -> Self {
        Self {
            started_at: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let total_attempts = self.total_loaded + self.load_failures;
        if total_attempts == 0 {
            100.0
        } else {
            (self.total_loaded as f64 / total_attempts as f64) * 100.0
        }
    }

    /// Calculate cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.total_loaded + self.cache_hits + self.load_failures;
        if total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Get total uptime since metrics started
    pub fn uptime(&self) -> Duration {
        self.started_at
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }

    /// Update average load time with new measurement
    pub fn update_average_load_time(&mut self, new_load_time: Duration) {
        if self.total_loaded == 0 {
            self.average_load_time = new_load_time;
        } else {
            let total_nanos = self.average_load_time.as_nanos() * self.total_loaded as u128
                + new_load_time.as_nanos();
            let new_count = self.total_loaded + 1;
            self.average_load_time = Duration::from_nanos((total_nanos / new_count as u128) as u64);
        }
    }

    /// Record a successful load
    pub fn record_success(&mut self, load_time: Duration) {
        self.total_loaded += 1;
        self.background_loads += 1;
        self.update_average_load_time(load_time);
    }

    /// Record a predictive load
    pub fn record_predictive_load(&mut self, load_time: Duration) {
        self.total_loaded += 1;
        self.predictive_loads += 1;
        self.update_average_load_time(load_time);
    }

    /// Record a failed load
    pub fn record_failure(&mut self) {
        self.load_failures += 1;
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record a retry attempt
    pub fn record_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Update peak queue length if current is higher
    pub fn update_peak_queue_length(&mut self, current_length: usize) {
        if current_length > self.peak_queue_length {
            self.peak_queue_length = current_length;
        }
    }

    /// Update peak concurrent workers if current is higher
    pub fn update_peak_concurrent_workers(&mut self, current_workers: usize) {
        if current_workers > self.peak_concurrent_workers {
            self.peak_concurrent_workers = current_workers;
        }
    }

    /// Reset all metrics (useful for testing)
    pub fn reset(&mut self) {
        *self = LoadingMetrics::new();
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> LoadingMetricsSnapshot {
        LoadingMetricsSnapshot {
            essential_load_time: self.essential_load_time,
            total_loaded: self.total_loaded,
            load_failures: self.load_failures,
            cache_hits: self.cache_hits,
            background_loads: self.background_loads,
            predictive_loads: self.predictive_loads,
            average_load_time: self.average_load_time,
            peak_queue_length: self.peak_queue_length,
            peak_concurrent_workers: self.peak_concurrent_workers,
            retry_count: self.retry_count,
            success_rate: self.success_rate(),
            cache_hit_rate: self.cache_hit_rate(),
            uptime: self.uptime(),
        }
    }
}

/// Immutable snapshot of loading metrics
#[derive(Debug, Clone)]
pub struct LoadingMetricsSnapshot {
    pub essential_load_time: Duration,
    pub total_loaded: u32,
    pub load_failures: u32,
    pub cache_hits: u32,
    pub background_loads: u32,
    pub predictive_loads: u32,
    pub average_load_time: Duration,
    pub peak_queue_length: usize,
    pub peak_concurrent_workers: usize,
    pub retry_count: u32,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub uptime: Duration,
}

impl LoadingMetricsSnapshot {
    /// Get loads per second rate
    pub fn loads_per_second(&self) -> f64 {
        let uptime_secs = self.uptime.as_secs_f64();
        if uptime_secs == 0.0 {
            0.0
        } else {
            self.total_loaded as f64 / uptime_secs
        }
    }

    /// Get predictive load percentage
    pub fn predictive_load_percentage(&self) -> f64 {
        if self.total_loaded == 0 {
            0.0
        } else {
            (self.predictive_loads as f64 / self.total_loaded as f64) * 100.0
        }
    }
}

/// Thread-safe wrapper for LoadingMetrics
#[derive(Debug)]
pub struct LoadingMetricsCollector {
    metrics: RwLock<LoadingMetrics>,
}

impl LoadingMetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(LoadingMetrics::new()),
        }
    }

    /// Record a successful schema load
    pub fn record_success(&self, load_time: Duration) {
        self.metrics.write().record_success(load_time);
    }

    /// Record a predictive load
    pub fn record_predictive_load(&self, load_time: Duration) {
        self.metrics.write().record_predictive_load(load_time);
    }

    /// Record a failed load
    pub fn record_failure(&self) {
        self.metrics.write().record_failure();
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.metrics.write().record_cache_hit();
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.metrics.write().record_retry();
    }

    /// Update peak queue length
    pub fn update_peak_queue_length(&self, current_length: usize) {
        self.metrics
            .write()
            .update_peak_queue_length(current_length);
    }

    /// Update peak concurrent workers
    pub fn update_peak_concurrent_workers(&self, current_workers: usize) {
        self.metrics
            .write()
            .update_peak_concurrent_workers(current_workers);
    }

    /// Set essential load time
    pub fn set_essential_load_time(&self, time: Duration) {
        self.metrics.write().essential_load_time = time;
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> LoadingMetricsSnapshot {
        self.metrics.read().snapshot()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.metrics.write().reset();
    }
}

impl Default for LoadingMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metrics_calculations() {
        let mut metrics = LoadingMetrics::new();

        // Test initial state
        assert_eq!(metrics.success_rate(), 100.0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);

        // Record some activities
        metrics.record_success(Duration::from_millis(100));
        metrics.record_success(Duration::from_millis(200));
        metrics.record_failure();
        metrics.record_cache_hit();

        // Test calculations
        assert_eq!(metrics.success_rate(), 66.66666666666666); // 2 success out of 3 attempts
        assert_eq!(metrics.cache_hit_rate(), 25.0); // 1 hit out of 4 requests
        assert_eq!(metrics.total_loaded, 2);
        assert_eq!(metrics.load_failures, 1);
        assert_eq!(metrics.cache_hits, 1);
    }

    #[test]
    fn test_average_load_time() {
        let mut metrics = LoadingMetrics::new();

        metrics.record_success(Duration::from_millis(100));
        assert_eq!(metrics.average_load_time, Duration::from_millis(100));

        metrics.record_success(Duration::from_millis(200));
        assert_eq!(metrics.average_load_time, Duration::from_millis(150));

        metrics.record_success(Duration::from_millis(300));
        assert_eq!(metrics.average_load_time, Duration::from_millis(200));
    }

    #[test]
    fn test_peak_tracking() {
        let mut metrics = LoadingMetrics::new();

        metrics.update_peak_queue_length(5);
        assert_eq!(metrics.peak_queue_length, 5);

        metrics.update_peak_queue_length(3);
        assert_eq!(metrics.peak_queue_length, 5); // Should not decrease

        metrics.update_peak_queue_length(10);
        assert_eq!(metrics.peak_queue_length, 10); // Should increase

        metrics.update_peak_concurrent_workers(4);
        assert_eq!(metrics.peak_concurrent_workers, 4);
    }

    #[test]
    fn test_snapshot() {
        let mut metrics = LoadingMetrics::new();

        metrics.record_success(Duration::from_millis(100));
        metrics.record_predictive_load(Duration::from_millis(150));
        metrics.update_peak_queue_length(7);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_loaded, 2);
        assert_eq!(snapshot.predictive_loads, 1);
        assert_eq!(snapshot.peak_queue_length, 7);
        assert_eq!(snapshot.predictive_load_percentage(), 50.0);
    }

    #[test]
    fn test_metrics_collector_thread_safety() {
        let collector = std::sync::Arc::new(LoadingMetricsCollector::new());
        let mut handles = vec![];

        // Spawn multiple threads to update metrics
        for _ in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    collector_clone.record_success(Duration::from_millis(50));
                    collector_clone.record_cache_hit();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.total_loaded, 1000);
        assert_eq!(snapshot.cache_hits, 1000);
        assert_eq!(snapshot.success_rate, 100.0);
        assert_eq!(snapshot.cache_hit_rate, 50.0);
    }
}
