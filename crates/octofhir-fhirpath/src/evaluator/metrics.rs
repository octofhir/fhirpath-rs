//! Performance Metrics Collection for FHIRPath Evaluation
//!
//! This module provides comprehensive performance metrics collection and reporting
//! for FHIRPath expression evaluation, including timing, operation counts, and memory usage.

use std::fmt;
use std::time::{Duration, Instant};

/// Performance metrics for FHIRPath expression evaluation
///
/// Tracks comprehensive performance data during expression evaluation including
/// timing information, operation counts, and resource usage statistics.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath::evaluator::EvaluationMetrics;
/// use std::time::Duration;
///
/// let metrics = EvaluationMetrics {
///     total_time_us: 1500,
///     parse_time_us: 200,
///     eval_time_us: 1300,
///     function_calls: 5,
///     model_provider_calls: 12,
///     service_calls: 2,
///     memory_allocations: 45,
/// };
///
/// println!("Total time: {}μs", metrics.total_time_us);
/// println!("Operations per second: {:.0}", metrics.operations_per_second());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationMetrics {
    /// Total evaluation time in microseconds
    ///
    /// Includes all time spent from the start of evaluation request
    /// to completion, including parsing, evaluation, and result formatting.
    pub total_time_us: u64,

    /// Parse time in microseconds
    ///
    /// Time spent parsing the FHIRPath expression string into an AST.
    /// This can be zero if a cached AST was used.
    pub parse_time_us: u64,

    /// Evaluation time in microseconds
    ///
    /// Time spent evaluating the parsed AST against the input collection.
    /// This is typically the bulk of the execution time.
    pub eval_time_us: u64,

    /// Number of function calls made during evaluation
    ///
    /// Tracks calls to FHIRPath functions like first(), where(), select(), etc.
    /// Useful for understanding expression complexity.
    pub function_calls: usize,

    /// Number of ModelProvider operations
    ///
    /// Tracks calls to the ModelProvider for property validation, type checking,
    /// and FHIR schema operations. High counts may indicate inefficient expressions.
    pub model_provider_calls: usize,

    /// Number of service calls (terminology, server)
    ///
    /// Tracks external service calls including terminology server operations
    /// and FHIR server API calls. These are typically the slowest operations.
    pub service_calls: usize,

    /// Memory allocations during evaluation
    ///
    /// Approximate count of significant memory allocations during evaluation.
    /// Useful for identifying memory-intensive expressions.
    pub memory_allocations: usize,
}

impl EvaluationMetrics {
    /// Create new metrics with all values set to zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metrics from timing measurements
    ///
    /// # Arguments
    /// * `total_duration` - Total evaluation duration
    /// * `parse_duration` - Parsing duration
    /// * `eval_duration` - Evaluation duration
    pub fn from_durations(
        total_duration: Duration,
        parse_duration: Duration,
        eval_duration: Duration,
    ) -> Self {
        Self {
            total_time_us: total_duration.as_micros() as u64,
            parse_time_us: parse_duration.as_micros() as u64,
            eval_time_us: eval_duration.as_micros() as u64,
            function_calls: 0,
            model_provider_calls: 0,
            service_calls: 0,
            memory_allocations: 0,
        }
    }

    /// Calculate operations per second based on total time
    ///
    /// Returns the number of evaluations that could be performed per second
    /// at the current performance level. Useful for benchmarking.
    ///
    /// # Returns
    /// * `f64` - Operations per second (0.0 if total_time_us is 0)
    pub fn operations_per_second(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            1_000_000.0 / self.total_time_us as f64
        }
    }

    /// Calculate parse efficiency as percentage of total time
    ///
    /// Returns what percentage of total evaluation time was spent parsing.
    /// Lower percentages indicate better cache utilization.
    ///
    /// # Returns
    /// * `f64` - Parse time as percentage of total (0.0-100.0)
    pub fn parse_percentage(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            (self.parse_time_us as f64 / self.total_time_us as f64) * 100.0
        }
    }

    /// Calculate evaluation efficiency as percentage of total time
    ///
    /// Returns what percentage of total evaluation time was spent in actual evaluation.
    /// Higher percentages indicate most time is spent in core evaluation logic.
    ///
    /// # Returns
    /// * `f64` - Evaluation time as percentage of total (0.0-100.0)
    pub fn eval_percentage(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            (self.eval_time_us as f64 / self.total_time_us as f64) * 100.0
        }
    }

    /// Calculate average time per function call
    ///
    /// Returns the average microseconds spent per function call.
    /// Useful for identifying expensive function usage patterns.
    ///
    /// # Returns
    /// * `f64` - Average microseconds per function call (0.0 if no function calls)
    pub fn avg_function_time_us(&self) -> f64 {
        if self.function_calls == 0 {
            0.0
        } else {
            self.eval_time_us as f64 / self.function_calls as f64
        }
    }

    /// Calculate average time per model provider call
    ///
    /// Returns the average microseconds spent per model provider operation.
    /// Useful for identifying model provider performance issues.
    ///
    /// # Returns
    /// * `f64` - Average microseconds per model provider call (0.0 if no calls)
    pub fn avg_model_provider_time_us(&self) -> f64 {
        if self.model_provider_calls == 0 {
            0.0
        } else {
            self.eval_time_us as f64 / self.model_provider_calls as f64
        }
    }

    /// Check if performance meets target thresholds
    ///
    /// Validates performance against common targets:
    /// - Simple expressions: 10K+ ops/sec
    /// - Complex expressions: 1K+ ops/sec  
    /// - AST cached expressions: 50K+ ops/sec (when parse_time_us < 10% of total)
    ///
    /// # Returns
    /// * `PerformanceLevel` - Achieved performance level
    pub fn performance_level(&self) -> PerformanceLevel {
        let ops_per_sec = self.operations_per_second();
        let parse_pct = self.parse_percentage();

        if ops_per_sec >= 50000.0 && parse_pct < 10.0 {
            PerformanceLevel::Excellent
        } else if ops_per_sec >= 10000.0 {
            PerformanceLevel::Good
        } else if ops_per_sec >= 1000.0 {
            PerformanceLevel::Adequate
        } else {
            PerformanceLevel::Poor
        }
    }

    /// Add metrics from another evaluation
    ///
    /// Combines metrics from multiple evaluations for aggregate reporting.
    ///
    /// # Arguments
    /// * `other` - Other metrics to add to this one
    pub fn add(&mut self, other: &EvaluationMetrics) {
        self.total_time_us += other.total_time_us;
        self.parse_time_us += other.parse_time_us;
        self.eval_time_us += other.eval_time_us;
        self.function_calls += other.function_calls;
        self.model_provider_calls += other.model_provider_calls;
        self.service_calls += other.service_calls;
        self.memory_allocations += other.memory_allocations;
    }

    /// Create summary for multiple evaluations
    ///
    /// Calculates average metrics across multiple evaluation runs.
    ///
    /// # Arguments
    /// * `count` - Number of evaluations these metrics represent
    ///
    /// # Returns
    /// * `EvaluationMetrics` - Average metrics per evaluation
    pub fn average(&self, count: usize) -> EvaluationMetrics {
        if count == 0 {
            return EvaluationMetrics::default();
        }

        EvaluationMetrics {
            total_time_us: self.total_time_us / count as u64,
            parse_time_us: self.parse_time_us / count as u64,
            eval_time_us: self.eval_time_us / count as u64,
            function_calls: self.function_calls / count,
            model_provider_calls: self.model_provider_calls / count,
            service_calls: self.service_calls / count,
            memory_allocations: self.memory_allocations / count,
        }
    }

    /// Format metrics for human-readable display
    ///
    /// Creates a formatted string with key performance indicators
    /// suitable for logging or debugging output.
    ///
    /// # Returns
    /// * `String` - Formatted metrics summary
    pub fn format_summary(&self) -> String {
        format!(
            "Evaluation: {}μs total ({:.0} ops/sec), {} functions, {} model ops, {} service calls",
            self.total_time_us,
            self.operations_per_second(),
            self.function_calls,
            self.model_provider_calls,
            self.service_calls
        )
    }
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self {
            total_time_us: 0,
            parse_time_us: 0,
            eval_time_us: 0,
            function_calls: 0,
            model_provider_calls: 0,
            service_calls: 0,
            memory_allocations: 0,
        }
    }
}

impl fmt::Display for EvaluationMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_summary())
    }
}

/// Performance level classification
///
/// Categorizes evaluation performance based on operations per second
/// and parsing efficiency to help identify optimization opportunities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceLevel {
    /// Excellent performance (50K+ ops/sec with good cache utilization)
    Excellent,
    /// Good performance (10K+ ops/sec)
    Good,
    /// Adequate performance (1K+ ops/sec)
    Adequate,
    /// Poor performance (<1K ops/sec, needs optimization)
    Poor,
}

impl fmt::Display for PerformanceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerformanceLevel::Excellent => write!(f, "Excellent"),
            PerformanceLevel::Good => write!(f, "Good"),
            PerformanceLevel::Adequate => write!(f, "Adequate"),
            PerformanceLevel::Poor => write!(f, "Poor"),
        }
    }
}

/// Metrics collector for tracking performance during evaluation
///
/// A helper utility for collecting timing and operation metrics during
/// the evaluation process. Provides methods for starting/stopping timers
/// and incrementing operation counters.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath::evaluator::MetricsCollector;
///
/// let mut collector = MetricsCollector::new();
///
/// collector.start_timing();
/// // ... perform parsing ...
/// collector.record_parse_time();
///
/// // ... perform evaluation ...
/// collector.increment_function_calls();
/// collector.increment_model_provider_calls();
///
/// collector.finish_timing();
/// let metrics = collector.into_metrics();
/// ```
#[derive(Debug)]
pub struct MetricsCollector {
    /// Start time of overall evaluation
    start_time: Option<Instant>,
    /// Start time of parsing phase
    parse_start_time: Option<Instant>,
    /// Accumulated metrics
    metrics: EvaluationMetrics,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            start_time: None,
            parse_start_time: None,
            metrics: EvaluationMetrics::default(),
        }
    }

    /// Start overall timing
    pub fn start_timing(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Start parsing timing
    pub fn start_parse_timing(&mut self) {
        self.parse_start_time = Some(Instant::now());
    }

    /// Record parsing completion time
    pub fn record_parse_time(&mut self) {
        if let Some(start) = self.parse_start_time.take() {
            self.metrics.parse_time_us = start.elapsed().as_micros() as u64;
        }
    }

    /// Finish overall timing
    pub fn finish_timing(&mut self) {
        if let Some(start) = self.start_time.take() {
            let total_elapsed = start.elapsed();
            self.metrics.total_time_us = total_elapsed.as_micros() as u64;
            self.metrics.eval_time_us = self
                .metrics
                .total_time_us
                .saturating_sub(self.metrics.parse_time_us);
        }
    }

    /// Increment function call counter
    pub fn increment_function_calls(&mut self) {
        self.metrics.function_calls += 1;
    }

    /// Add multiple function calls
    pub fn add_function_calls(&mut self, count: usize) {
        self.metrics.function_calls += count;
    }

    /// Increment model provider call counter
    pub fn increment_model_provider_calls(&mut self) {
        self.metrics.model_provider_calls += 1;
    }

    /// Add multiple model provider calls
    pub fn add_model_provider_calls(&mut self, count: usize) {
        self.metrics.model_provider_calls += count;
    }

    /// Increment service call counter
    pub fn increment_service_calls(&mut self) {
        self.metrics.service_calls += 1;
    }

    /// Add multiple service calls
    pub fn add_service_calls(&mut self, count: usize) {
        self.metrics.service_calls += count;
    }

    /// Increment memory allocation counter
    pub fn increment_memory_allocations(&mut self) {
        self.metrics.memory_allocations += 1;
    }

    /// Add multiple memory allocations
    pub fn add_memory_allocations(&mut self, count: usize) {
        self.metrics.memory_allocations += count;
    }

    /// Get current metrics (without consuming the collector)
    pub fn current_metrics(&self) -> &EvaluationMetrics {
        &self.metrics
    }

    /// Convert collector into final metrics
    pub fn into_metrics(self) -> EvaluationMetrics {
        self.metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = EvaluationMetrics::new();
        assert_eq!(metrics.total_time_us, 0);
        assert_eq!(metrics.function_calls, 0);
        assert_eq!(metrics.operations_per_second(), 0.0);
    }

    #[test]
    fn test_operations_per_second() {
        let metrics = EvaluationMetrics {
            total_time_us: 1000, // 1ms = 1000 ops/sec
            ..Default::default()
        };
        assert_eq!(metrics.operations_per_second(), 1000.0);

        let fast_metrics = EvaluationMetrics {
            total_time_us: 100, // 0.1ms = 10000 ops/sec
            ..Default::default()
        };
        assert_eq!(fast_metrics.operations_per_second(), 10000.0);
    }

    #[test]
    fn test_percentage_calculations() {
        let metrics = EvaluationMetrics {
            total_time_us: 1000,
            parse_time_us: 100,
            eval_time_us: 900,
            ..Default::default()
        };

        assert_eq!(metrics.parse_percentage(), 10.0);
        assert_eq!(metrics.eval_percentage(), 90.0);
    }

    #[test]
    fn test_average_time_calculations() {
        let metrics = EvaluationMetrics {
            eval_time_us: 1000,
            function_calls: 10,
            model_provider_calls: 5,
            ..Default::default()
        };

        assert_eq!(metrics.avg_function_time_us(), 100.0);
        assert_eq!(metrics.avg_model_provider_time_us(), 200.0);
    }

    #[test]
    fn test_performance_levels() {
        // Excellent performance
        let excellent = EvaluationMetrics {
            total_time_us: 20, // 50K ops/sec
            parse_time_us: 1,  // 5% parse time
            ..Default::default()
        };
        assert_eq!(excellent.performance_level(), PerformanceLevel::Excellent);

        // Good performance
        let good = EvaluationMetrics {
            total_time_us: 100, // 10K ops/sec
            ..Default::default()
        };
        assert_eq!(good.performance_level(), PerformanceLevel::Good);

        // Adequate performance
        let adequate = EvaluationMetrics {
            total_time_us: 1000, // 1K ops/sec
            ..Default::default()
        };
        assert_eq!(adequate.performance_level(), PerformanceLevel::Adequate);

        // Poor performance
        let poor = EvaluationMetrics {
            total_time_us: 10000, // 100 ops/sec
            ..Default::default()
        };
        assert_eq!(poor.performance_level(), PerformanceLevel::Poor);
    }

    #[test]
    fn test_metrics_addition() {
        let mut metrics1 = EvaluationMetrics {
            total_time_us: 1000,
            function_calls: 5,
            ..Default::default()
        };

        let metrics2 = EvaluationMetrics {
            total_time_us: 2000,
            function_calls: 3,
            ..Default::default()
        };

        metrics1.add(&metrics2);

        assert_eq!(metrics1.total_time_us, 3000);
        assert_eq!(metrics1.function_calls, 8);
    }

    #[test]
    fn test_metrics_averaging() {
        let total_metrics = EvaluationMetrics {
            total_time_us: 3000,
            function_calls: 15,
            model_provider_calls: 6,
            ..Default::default()
        };

        let average = total_metrics.average(3);

        assert_eq!(average.total_time_us, 1000);
        assert_eq!(average.function_calls, 5);
        assert_eq!(average.model_provider_calls, 2);
    }

    #[test]
    fn test_format_summary() {
        let metrics = EvaluationMetrics {
            total_time_us: 1500,
            function_calls: 5,
            model_provider_calls: 10,
            service_calls: 2,
            ..Default::default()
        };

        let summary = metrics.format_summary();
        assert!(summary.contains("1500μs"));
        assert!(summary.contains("5 functions"));
        assert!(summary.contains("10 model ops"));
        assert!(summary.contains("2 service calls"));
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.start_timing();
        thread::sleep(Duration::from_millis(1)); // Small delay for timing

        collector.start_parse_timing();
        thread::sleep(Duration::from_millis(1));
        collector.record_parse_time();

        collector.increment_function_calls();
        collector.increment_model_provider_calls();

        collector.finish_timing();
        let metrics = collector.into_metrics();

        assert!(metrics.total_time_us > 0);
        assert!(metrics.parse_time_us > 0);
        assert_eq!(metrics.function_calls, 1);
        assert_eq!(metrics.model_provider_calls, 1);
    }

    #[test]
    fn test_from_durations() {
        let total = Duration::from_millis(10);
        let parse = Duration::from_millis(2);
        let eval = Duration::from_millis(8);

        let metrics = EvaluationMetrics::from_durations(total, parse, eval);

        assert_eq!(metrics.total_time_us, 10000);
        assert_eq!(metrics.parse_time_us, 2000);
        assert_eq!(metrics.eval_time_us, 8000);
    }
}
