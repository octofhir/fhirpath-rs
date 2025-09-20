//! Metadata Collection System
//!
//! This module provides comprehensive metadata collection during FHIRPath evaluation
//! for debugging, profiling, and development tool support.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

// Note: Imports are currently unused but may be needed for future enhancements

/// Comprehensive metadata collector for FHIRPath evaluation
#[derive(Debug)]
pub struct MetadataCollector {
    /// Node evaluation information
    node_evaluations: Arc<Mutex<Vec<NodeEvaluationInfo>>>,
    /// Type resolution tracking
    type_resolutions: Arc<Mutex<Vec<TypeResolutionInfo>>>,
    /// Cache performance statistics
    cache_stats: Arc<Mutex<CacheStats>>,
    /// Trace events for detailed flow analysis
    trace_events: Arc<Mutex<Vec<TraceEvent>>>,
    /// Performance metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Start time for total execution time
    start_time: Instant,
    /// Unique identifier for this collection session
    session_id: String,
}

/// Information about a single node evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvaluationInfo {
    /// Type of AST node (e.g., "PropertyAccess", "FunctionCall")
    pub node_type: String,
    /// Source location in the original expression
    pub node_location: Option<SourceLocation>,
    /// Number of input values
    pub input_count: usize,
    /// Number of output values
    pub output_count: usize,
    /// Time taken for this evaluation
    pub execution_time: Duration,
    /// Error message if evaluation failed
    pub error: Option<String>,
    /// Input type information (for debugging)
    pub input_types: Vec<String>,
    /// Output type information (for debugging)
    pub output_types: Vec<String>,
    /// Node depth in the AST
    pub depth: usize,
    /// Unique evaluation ID
    pub evaluation_id: usize,
}

/// Information about type resolution operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeResolutionInfo {
    /// Name of the type being resolved
    pub type_name: String,
    /// Property name if resolving property type
    pub property_name: Option<String>,
    /// Resolved type information
    pub resolved_type: Option<String>, // Using String for serialization
    /// Source of the resolution
    pub source: TypeResolutionSource,
    /// Time taken for resolution
    pub execution_time: Duration,
    /// Whether resolution was successful
    pub success: bool,
    /// Cache hit/miss information
    pub cache_hit: bool,
}

/// Source of type resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeResolutionSource {
    /// Resolved via ModelProvider
    ModelProvider,
    /// Resolved via type inference
    Inference,
    /// Retrieved from cache
    Cache,
    /// Resolved from literal value
    Literal,
}

/// Cache performance statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: usize,
    /// Total cache misses
    pub misses: usize,
    /// Cache size (number of entries)
    pub size: usize,
    /// Cache evictions
    pub evictions: usize,
    /// Cache operation times
    pub operation_times: HashMap<String, Duration>,
    /// Memory usage (approximate)
    pub memory_usage_bytes: usize,
}

/// Trace events for detailed evaluation flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    /// Evaluation started
    EvaluationStart {
        node_type: String,
        input_count: usize,
        depth: usize,
        timestamp: Duration,
    },
    /// Evaluation completed
    EvaluationEnd {
        node_type: String,
        execution_time: Duration,
        success: bool,
        output_count: usize,
        timestamp: Duration,
    },
    /// Function call event
    FunctionCall {
        function_name: String,
        input_count: usize,
        parameter_count: usize,
        timestamp: Duration,
    },
    /// Operator evaluation event
    OperatorEvaluation {
        operator: String,
        left_count: usize,
        right_count: usize,
        timestamp: Duration,
    },
    /// Property access event
    PropertyAccess {
        property_name: String,
        input_count: usize,
        timestamp: Duration,
    },
    /// Type resolution event
    TypeResolution {
        type_name: String,
        property: Option<String>,
        source: TypeResolutionSource,
        timestamp: Duration,
    },
}

/// Performance metrics collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Function call timings
    pub function_timings: HashMap<String, FunctionTiming>,
    /// Operator evaluation timings
    pub operator_timings: HashMap<String, OperatorTiming>,
    /// Property access timings
    pub property_timings: HashMap<String, PropertyTiming>,
    /// Memory allocation tracking
    pub memory_metrics: MemoryMetrics,
    /// Evaluation depth statistics
    pub depth_stats: DepthStats,
}

/// Function performance timing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionTiming {
    /// Total execution time
    pub total_time: Duration,
    /// Number of calls
    pub call_count: usize,
    /// Average execution time
    pub average_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Times for individual calls
    pub individual_times: Vec<Duration>,
}

/// Operator performance timing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperatorTiming {
    /// Total execution time
    pub total_time: Duration,
    /// Number of evaluations
    pub evaluation_count: usize,
    /// Average execution time
    pub average_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
}

/// Property access timing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropertyTiming {
    /// Total execution time
    pub total_time: Duration,
    /// Number of accesses
    pub access_count: usize,
    /// Average execution time
    pub average_time: Duration,
    /// Type resolution time
    pub type_resolution_time: Duration,
}

/// Memory usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Peak memory usage
    pub peak_usage_bytes: usize,
    /// Current memory usage
    pub current_usage_bytes: usize,
    /// Memory allocations count
    pub allocation_count: usize,
    /// Memory deallocations count
    pub deallocation_count: usize,
}

/// Evaluation depth statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepthStats {
    /// Maximum depth reached
    pub max_depth: usize,
    /// Average depth
    pub average_depth: f64,
    /// Depth distribution
    pub depth_distribution: HashMap<usize, usize>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
    /// Source expression
    pub source: String,
}

impl MetadataCollector {
    /// Create a new metadata collector
    pub fn new() -> Self {
        Self {
            node_evaluations: Arc::new(Mutex::new(Vec::new())),
            type_resolutions: Arc::new(Mutex::new(Vec::new())),
            cache_stats: Arc::new(Mutex::new(CacheStats::default())),
            trace_events: Arc::new(Mutex::new(Vec::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            start_time: Instant::now(),
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create a new collector with custom session ID
    pub fn with_session_id(session_id: String) -> Self {
        Self {
            node_evaluations: Arc::new(Mutex::new(Vec::new())),
            type_resolutions: Arc::new(Mutex::new(Vec::new())),
            cache_stats: Arc::new(Mutex::new(CacheStats::default())),
            trace_events: Arc::new(Mutex::new(Vec::new())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            start_time: Instant::now(),
            session_id,
        }
    }

    /// Record node evaluation information
    pub fn record_node_evaluation(&self, info: NodeEvaluationInfo) {
        if let Ok(mut evaluations) = self.node_evaluations.lock() {
            evaluations.push(info);
        }
    }

    /// Record type resolution information
    pub fn record_type_resolution(&self, info: TypeResolutionInfo) {
        if let Ok(mut resolutions) = self.type_resolutions.lock() {
            resolutions.push(info);
        }
    }

    /// Record trace event
    pub fn record_trace_event(&self, event: TraceEvent) {
        if let Ok(mut events) = self.trace_events.lock() {
            events.push(event);
        }
    }

    /// Update cache statistics
    pub fn update_cache_stats<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut CacheStats),
    {
        if let Ok(mut stats) = self.cache_stats.lock() {
            update_fn(&mut stats);
        }
    }

    /// Record function timing
    pub fn record_function_timing(&self, function_name: &str, execution_time: Duration) {
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let timing = metrics
                .function_timings
                .entry(function_name.to_string())
                .or_default();
            timing.total_time += execution_time;
            timing.call_count += 1;
            timing.individual_times.push(execution_time);

            // Update min/max
            if timing.min_time.is_zero() || execution_time < timing.min_time {
                timing.min_time = execution_time;
            }
            if execution_time > timing.max_time {
                timing.max_time = execution_time;
            }

            // Update average
            timing.average_time = timing.total_time / timing.call_count as u32;
        }
    }

    /// Record operator timing
    pub fn record_operator_timing(&self, operator: &str, execution_time: Duration) {
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let timing = metrics
                .operator_timings
                .entry(operator.to_string())
                .or_default();
            timing.total_time += execution_time;
            timing.evaluation_count += 1;

            // Update min/max
            if timing.min_time.is_zero() || execution_time < timing.min_time {
                timing.min_time = execution_time;
            }
            if execution_time > timing.max_time {
                timing.max_time = execution_time;
            }

            // Update average
            timing.average_time = timing.total_time / timing.evaluation_count as u32;
        }
    }

    /// Record property access timing
    pub fn record_property_timing(
        &self,
        property: &str,
        execution_time: Duration,
        type_resolution_time: Duration,
    ) {
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let timing = metrics
                .property_timings
                .entry(property.to_string())
                .or_default();
            timing.total_time += execution_time;
            timing.access_count += 1;
            timing.type_resolution_time += type_resolution_time;
            timing.average_time = timing.total_time / timing.access_count as u32;
        }
    }

    /// Update depth statistics
    pub fn update_depth_stats(&self, depth: usize) {
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            let stats = &mut metrics.depth_stats;
            stats.max_depth = stats.max_depth.max(depth);
            *stats.depth_distribution.entry(depth).or_insert(0) += 1;

            // Recalculate average depth
            let total_evaluations: usize = stats.depth_distribution.values().sum();
            let weighted_sum: usize = stats
                .depth_distribution
                .iter()
                .map(|(depth, count)| depth * count)
                .sum();
            stats.average_depth = weighted_sum as f64 / total_evaluations as f64;
        }
    }

    /// Get total execution time
    pub fn execution_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get collected node evaluations
    pub fn node_evaluations(&self) -> Vec<NodeEvaluationInfo> {
        self.node_evaluations
            .lock()
            .map(|evaluations| evaluations.clone())
            .unwrap_or_default()
    }

    /// Get collected type resolutions
    pub fn type_resolutions(&self) -> Vec<TypeResolutionInfo> {
        self.type_resolutions
            .lock()
            .map(|resolutions| resolutions.clone())
            .unwrap_or_default()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache_stats
            .lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// Get trace events
    pub fn trace_events(&self) -> Vec<TraceEvent> {
        self.trace_events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics
            .lock()
            .map(|metrics| metrics.clone())
            .unwrap_or_default()
    }

    /// Generate summary report
    pub fn generate_summary(&self) -> EvaluationSummary {
        let node_evaluations = self.node_evaluations();
        let type_resolutions = self.type_resolutions();
        let cache_stats = self.cache_stats();
        let performance_metrics = self.performance_metrics();

        EvaluationSummary {
            session_id: self.session_id.clone(),
            total_execution_time: self.execution_time(),
            total_node_evaluations: node_evaluations.len(),
            total_type_resolutions: type_resolutions.len(),
            successful_evaluations: node_evaluations
                .iter()
                .filter(|n| n.error.is_none())
                .count(),
            failed_evaluations: node_evaluations
                .iter()
                .filter(|n| n.error.is_some())
                .count(),
            cache_hit_rate: cache_stats.hit_rate(),
            average_evaluation_time: self.calculate_average_evaluation_time(&node_evaluations),
            max_depth: performance_metrics.depth_stats.max_depth,
            function_calls: performance_metrics.function_timings.len(),
            operator_evaluations: performance_metrics
                .operator_timings
                .values()
                .map(|t| t.evaluation_count)
                .sum(),
        }
    }

    /// Calculate average evaluation time
    fn calculate_average_evaluation_time(&self, evaluations: &[NodeEvaluationInfo]) -> Duration {
        if evaluations.is_empty() {
            return Duration::ZERO;
        }

        let total_time: Duration = evaluations.iter().map(|e| e.execution_time).sum();

        total_time / evaluations.len() as u32
    }

    /// Clear all collected data
    pub fn clear(&self) {
        if let Ok(mut evaluations) = self.node_evaluations.lock() {
            evaluations.clear();
        }
        if let Ok(mut resolutions) = self.type_resolutions.lock() {
            resolutions.clear();
        }
        if let Ok(mut events) = self.trace_events.lock() {
            events.clear();
        }
        if let Ok(mut stats) = self.cache_stats.lock() {
            *stats = CacheStats::default();
        }
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            *metrics = PerformanceMetrics::default();
        }
    }
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate cache miss rate
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
}

/// Summary of evaluation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummary {
    /// Session identifier
    pub session_id: String,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Total number of node evaluations
    pub total_node_evaluations: usize,
    /// Total number of type resolutions
    pub total_type_resolutions: usize,
    /// Number of successful evaluations
    pub successful_evaluations: usize,
    /// Number of failed evaluations
    pub failed_evaluations: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Average evaluation time per node
    pub average_evaluation_time: Duration,
    /// Maximum evaluation depth
    pub max_depth: usize,
    /// Number of function calls
    pub function_calls: usize,
    /// Number of operator evaluations
    pub operator_evaluations: usize,
}

impl Default for MetadataCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceEvent {
    /// Get timestamp for this event
    pub fn timestamp(&self) -> Duration {
        match self {
            TraceEvent::EvaluationStart { timestamp, .. } => *timestamp,
            TraceEvent::EvaluationEnd { timestamp, .. } => *timestamp,
            TraceEvent::FunctionCall { timestamp, .. } => *timestamp,
            TraceEvent::OperatorEvaluation { timestamp, .. } => *timestamp,
            TraceEvent::PropertyAccess { timestamp, .. } => *timestamp,
            TraceEvent::TypeResolution { timestamp, .. } => *timestamp,
        }
    }

    /// Get event type as string
    pub fn event_type(&self) -> &'static str {
        match self {
            TraceEvent::EvaluationStart { .. } => "EvaluationStart",
            TraceEvent::EvaluationEnd { .. } => "EvaluationEnd",
            TraceEvent::FunctionCall { .. } => "FunctionCall",
            TraceEvent::OperatorEvaluation { .. } => "OperatorEvaluation",
            TraceEvent::PropertyAccess { .. } => "PropertyAccess",
            TraceEvent::TypeResolution { .. } => "TypeResolution",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_collector_creation() {
        let collector = MetadataCollector::new();
        assert!(!collector.session_id.is_empty());
        assert_eq!(collector.node_evaluations().len(), 0);
        assert_eq!(collector.type_resolutions().len(), 0);
    }

    #[test]
    fn test_cache_hit_rate_calculation() {
        let mut stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.hits = 8;
        stats.misses = 2;
        assert_eq!(stats.hit_rate(), 0.8);
    }

    #[test]
    fn test_function_timing() {
        let collector = MetadataCollector::new();

        collector.record_function_timing("count", Duration::from_millis(10));
        collector.record_function_timing("count", Duration::from_millis(20));

        let metrics = collector.performance_metrics();
        let count_timing = metrics.function_timings.get("count").unwrap();

        assert_eq!(count_timing.call_count, 2);
        assert_eq!(count_timing.total_time, Duration::from_millis(30));
        assert_eq!(count_timing.average_time, Duration::from_millis(15));
    }

    #[test]
    fn test_evaluation_summary() {
        let collector = MetadataCollector::new();

        // Record some test data
        collector.record_node_evaluation(NodeEvaluationInfo {
            node_type: "PropertyAccess".to_string(),
            node_location: None,
            input_count: 1,
            output_count: 1,
            execution_time: Duration::from_millis(5),
            error: None,
            input_types: vec!["Patient".to_string()],
            output_types: vec!["String".to_string()],
            depth: 0,
            evaluation_id: 1,
        });

        let summary = collector.generate_summary();
        assert_eq!(summary.total_node_evaluations, 1);
        assert_eq!(summary.successful_evaluations, 1);
        assert_eq!(summary.failed_evaluations, 0);
    }
}
