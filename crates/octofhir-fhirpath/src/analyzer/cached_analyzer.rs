use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::analyzer::{
    AnalysisContext, CachedModelProvider, CachedModelProviderBuilder, StaticAnalysisResult,
    StaticAnalyzer,
};
use crate::core::ModelProvider;

/// Cached semantic analyzer with performance monitoring
pub struct CachedSemanticAnalyzer {
    inner: StaticAnalyzer,
    cached_provider: Arc<CachedModelProvider>,
    analysis_cache: AnalysisCache,
    performance_metrics: PerformanceMetrics,
}

/// Analysis cache for frequently used expressions
pub struct AnalysisCache {
    expression_cache: HashMap<String, CachedAnalysisResult>,
    max_cache_size: usize,
    cache_hits: usize,
    cache_misses: usize,
}

/// Cached analysis result with metadata
#[derive(Clone)]
struct CachedAnalysisResult {
    result: StaticAnalysisResult,
    timestamp: Instant,
    hit_count: usize,
}

/// Comprehensive performance metrics for the optimized analyzer
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_analyses: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_hit_ratio: f64,
    pub average_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
    pub total_duration: Duration,
    pub memory_usage_estimate: usize,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            total_analyses: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_ratio: 0.0,
            average_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            total_duration: Duration::ZERO,
            memory_usage_estimate: 0,
        }
    }

    fn record_analysis(&mut self, duration: Duration, was_cache_hit: bool) {
        self.total_analyses += 1;

        if was_cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        self.total_duration += duration;

        if duration > self.max_duration {
            self.max_duration = duration;
        }

        if duration < self.min_duration {
            self.min_duration = duration;
        }

        // Update derived metrics
        self.average_duration = self.total_duration / self.total_analyses as u32;
        self.cache_hit_ratio = if self.total_analyses > 0 {
            self.cache_hits as f64 / self.total_analyses as f64
        } else {
            0.0
        };
    }

    fn update_memory_estimate(&mut self, cache_size: usize) {
        // Rough estimate: assume each cache entry is ~1KB
        self.memory_usage_estimate = cache_size * 1024;
    }

    /// Check if performance is meeting targets
    pub fn meets_performance_targets(&self) -> bool {
        if self.total_analyses < 10 {
            return true; // Not enough data
        }

        // Performance targets from task specification
        let simple_expression_target = Duration::from_millis(50);
        let complex_expression_target = Duration::from_millis(200);
        let cache_hit_ratio_target = 0.8;

        // Check average duration (assuming most expressions are simple)
        let duration_ok = self.average_duration <= simple_expression_target
            || (self.max_duration <= complex_expression_target
                && self.average_duration <= Duration::from_millis(100));

        // Check cache hit ratio
        let cache_ok = self.cache_hit_ratio >= cache_hit_ratio_target;

        duration_ok && cache_ok
    }

    /// Generate performance report
    pub fn report(&self) -> String {
        format!(
            "Optimized Analyzer Performance Report:\n\
             Total Analyses: {}\n\
             Cache Hit Ratio: {:.2}% (target: ≥80%)\n\
             Average Duration: {:?}\n\
             Min Duration: {:?}\n\
             Max Duration: {:?}\n\
             Memory Usage Estimate: {} KB\n\
             Performance Targets Met: {}",
            self.total_analyses,
            self.cache_hit_ratio * 100.0,
            self.average_duration,
            if self.min_duration == Duration::MAX {
                Duration::ZERO
            } else {
                self.min_duration
            },
            self.max_duration,
            self.memory_usage_estimate / 1024,
            if self.meets_performance_targets() {
                "✓"
            } else {
                "✗"
            }
        )
    }
}

impl AnalysisCache {
    fn new(max_size: usize) -> Self {
        Self {
            expression_cache: HashMap::new(),
            max_cache_size: max_size,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<&CachedAnalysisResult> {
        if let Some(entry) = self.expression_cache.get_mut(key) {
            entry.hit_count += 1;
            self.cache_hits += 1;
            Some(entry)
        } else {
            self.cache_misses += 1;
            None
        }
    }

    fn insert(&mut self, key: String, result: StaticAnalysisResult) {
        // Implement LRU eviction if cache is full
        if self.expression_cache.len() >= self.max_cache_size {
            self.evict_lru();
        }

        let cached_result = CachedAnalysisResult {
            result,
            timestamp: Instant::now(),
            hit_count: 0,
        };

        self.expression_cache.insert(key, cached_result);
    }

    fn evict_lru(&mut self) {
        // Find the least recently used entry (oldest timestamp, lowest hit count)
        let mut lru_key = None;
        let mut lru_score = (Instant::now(), usize::MAX);

        for (key, entry) in &self.expression_cache {
            let score = (entry.timestamp, usize::MAX - entry.hit_count);
            if score < lru_score {
                lru_score = score;
                lru_key = Some(key.clone());
            }
        }

        if let Some(key) = lru_key {
            self.expression_cache.remove(&key);
        }
    }

    fn clear(&mut self) {
        self.expression_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    fn len(&self) -> usize {
        self.expression_cache.len()
    }
}

impl CachedSemanticAnalyzer {
    /// Create a new optimized analyzer with the given ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        let cached_provider = Arc::new(
            CachedModelProviderBuilder::new()
                .ttl(Duration::from_secs(300)) // 5 minutes
                .build(model_provider),
        );

        let inner = StaticAnalyzer::new(cached_provider.clone());

        Self {
            inner,
            cached_provider,
            analysis_cache: AnalysisCache::new(1000), // Cache up to 1000 expressions
            performance_metrics: PerformanceMetrics::new(),
        }
    }

    /// Create a new optimized analyzer with custom cache settings
    pub fn with_cache_settings(
        model_provider: Arc<dyn ModelProvider>,
        cache_ttl: Duration,
        expression_cache_size: usize,
    ) -> Self {
        let cached_provider = Arc::new(
            CachedModelProviderBuilder::new()
                .ttl(cache_ttl)
                .build(model_provider),
        );

        let inner = StaticAnalyzer::new(cached_provider.clone());

        Self {
            inner,
            cached_provider,
            analysis_cache: AnalysisCache::new(expression_cache_size),
            performance_metrics: PerformanceMetrics::new(),
        }
    }

    /// Cached expression analysis with performance monitoring
    pub async fn analyze_expression_cached(
        &mut self,
        expression: &str,
        context: AnalysisContext,
    ) -> StaticAnalysisResult {
        let start = Instant::now();
        let cache_key = self.create_cache_key(expression, &context);

        // Check cache first
        if let Some(cached) = self.analysis_cache.get(&cache_key) {
            let duration = start.elapsed();
            self.performance_metrics.record_analysis(duration, true);
            return cached.result.clone();
        }

        // Cache miss - perform analysis
        let result = self.inner.analyze_expression(expression, context).await;
        let duration = start.elapsed();

        // Cache the result
        self.analysis_cache.insert(cache_key, result.clone());

        // Update metrics
        self.performance_metrics.record_analysis(duration, false);
        self.performance_metrics
            .update_memory_estimate(self.analysis_cache.len());

        result
    }

    /// Analyze multiple expressions efficiently
    pub async fn analyze_expressions_batch(
        &mut self,
        expressions: &[(&str, AnalysisContext)],
    ) -> Vec<StaticAnalysisResult> {
        let mut results = Vec::with_capacity(expressions.len());

        for (expr, context) in expressions {
            let result = self.analyze_expression_cached(expr, context.clone()).await;
            results.push(result);
        }

        results
    }

    /// Create a stable cache key from expression and context
    fn create_cache_key(&self, expression: &str, context: &AnalysisContext) -> String {
        // Create a stable string representation of the expression and context
        let context_str = format!(
            "{}:{}",
            context.root_type.type_name,
            context.root_type.singleton.unwrap_or(false)
        );

        format!("{expression}|{context_str}")
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> CacheStatistics {
        let provider_info = self.cached_provider.get_cache_info().await;

        CacheStatistics {
            expression_cache_size: self.analysis_cache.len(),
            expression_cache_hits: self.analysis_cache.cache_hits,
            expression_cache_misses: self.analysis_cache.cache_misses,
            expression_cache_hit_ratio: if self.analysis_cache.cache_hits
                + self.analysis_cache.cache_misses
                > 0
            {
                self.analysis_cache.cache_hits as f64
                    / (self.analysis_cache.cache_hits + self.analysis_cache.cache_misses) as f64
            } else {
                0.0
            },
            model_provider_cache_info: provider_info,
        }
    }

    /// Clear all caches
    pub async fn clear_caches(&mut self) {
        self.analysis_cache.clear();
        self.cached_provider.clear_cache().await;
        self.performance_metrics = PerformanceMetrics::new();
    }

    /// Optimize cache settings based on usage patterns
    pub async fn optimize_cache_settings(&mut self) {
        let stats = self.get_cache_statistics().await;
        let metrics = &self.performance_metrics;

        // If cache hit ratio is low, increase cache size
        if stats.expression_cache_hit_ratio < 0.5 && self.analysis_cache.max_cache_size < 5000 {
            self.analysis_cache.max_cache_size *= 2;
        }

        // If memory usage is high, reduce cache size
        if metrics.memory_usage_estimate > 50_000_000 {
            // 50MB
            self.analysis_cache.max_cache_size = (self.analysis_cache.max_cache_size * 3) / 4;
        }

        // Clean up expired entries
        self.cached_provider.cleanup_expired().await;
    }

    /// Check if analyzer is performing within targets
    pub fn is_performing_well(&self) -> bool {
        self.performance_metrics.meets_performance_targets()
    }

    /// Generate comprehensive performance report
    pub async fn generate_performance_report(&self) -> String {
        let cache_stats = self.get_cache_statistics().await;
        let metrics_report = self.performance_metrics.report();

        format!(
            "{}\n\n\
             Expression Cache Statistics:\n\
             - Size: {} / {} entries\n\
             - Hit Ratio: {:.2}%\n\
             - Total Hits: {}\n\
             - Total Misses: {}\n\n\
             {}",
            metrics_report,
            cache_stats.expression_cache_size,
            self.analysis_cache.max_cache_size,
            cache_stats.expression_cache_hit_ratio * 100.0,
            cache_stats.expression_cache_hits,
            cache_stats.expression_cache_misses,
            cache_stats.model_provider_cache_info.report()
        )
    }
}

/// Combined cache statistics for both expression and model provider caches
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub expression_cache_size: usize,
    pub expression_cache_hits: usize,
    pub expression_cache_misses: usize,
    pub expression_cache_hit_ratio: f64,
    pub model_provider_cache_info: crate::analyzer::CacheInfo,
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ExpressionNode, LiteralValue};
    use octofhir_fhir_model::EmptyModelProvider;
    use std::time::Duration;

    fn create_test_analyzer() -> CachedSemanticAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        CachedSemanticAnalyzer::new(provider)
    }

    fn create_simple_expression() -> ExpressionNode {
        ExpressionNode::Literal(LiteralValue::String("test".to_string()))
    }

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        let metrics = analyzer.get_performance_metrics();

        assert_eq!(metrics.total_analyses, 0);
        assert_eq!(metrics.cache_hit_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_expression_caching() {
        let mut analyzer = create_test_analyzer();
        let expr = create_simple_expression();

        // First analysis - cache miss
        let result1 = analyzer.analyze_expression_cached(&expr, None).await;
        assert!(result1.is_ok());

        let metrics = analyzer.get_performance_metrics();
        assert_eq!(metrics.total_analyses, 1);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hits, 0);

        // Second analysis - cache hit
        let result2 = analyzer.analyze_expression_cached(&expr, None).await;
        assert!(result2.is_ok());

        let metrics = analyzer.get_performance_metrics();
        assert_eq!(metrics.total_analyses, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_batch_analysis() {
        let mut analyzer = create_test_analyzer();
        let expr1 = create_simple_expression();
        let expr2 = ExpressionNode::Literal(LiteralValue::String("test2".to_string()));

        let expressions = vec![
            (&expr1, None),
            (&expr2, None),
            (&expr1, None), // This should be a cache hit
        ];

        let results = analyzer.analyze_expressions_batch(&expressions).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 3);

        let metrics = analyzer.get_performance_metrics();
        assert_eq!(metrics.total_analyses, 3);
        assert_eq!(metrics.cache_hits, 1); // Third expression is a cache hit
        assert_eq!(metrics.cache_misses, 2);
    }

    #[tokio::test]
    async fn test_cache_clearing() {
        let mut analyzer = create_test_analyzer();
        let expr = create_simple_expression();

        // Perform some analyses
        let _ = analyzer.analyze_expression_cached(&expr, None).await;
        let _ = analyzer.analyze_expression_cached(&expr, None).await;

        assert!(analyzer.get_performance_metrics().total_analyses > 0);

        // Clear caches
        analyzer.clear_caches().await;

        let metrics = analyzer.get_performance_metrics();
        assert_eq!(metrics.total_analyses, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let mut analyzer = create_test_analyzer();
        let expr = create_simple_expression();

        // Perform multiple analyses
        for _ in 0..10 {
            let _ = analyzer.analyze_expression_cached(&expr, None).await;
        }

        let metrics = analyzer.get_performance_metrics();
        assert_eq!(metrics.total_analyses, 10);
        assert!(metrics.cache_hit_ratio > 0.5); // Most should be cache hits
        assert!(metrics.average_duration > Duration::ZERO);
        assert!(metrics.max_duration >= metrics.average_duration);
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let mut analyzer = create_test_analyzer();
        let expr = create_simple_expression();

        // Perform some analyses
        let _ = analyzer.analyze_expression_cached(&expr, None).await;
        let _ = analyzer.analyze_expression_cached(&expr, None).await;

        let stats = analyzer.get_cache_statistics().await;
        assert!(stats.expression_cache_size > 0);
        assert!(stats.expression_cache_hits > 0);
        assert!(stats.expression_cache_misses > 0);
        assert!(stats.expression_cache_hit_ratio > 0.0);
    }

    #[test]
    fn test_performance_targets() {
        let mut metrics = PerformanceMetrics::new();

        // Test with good performance
        for _ in 0..20 {
            metrics.record_analysis(Duration::from_millis(30), true);
        }

        assert!(metrics.meets_performance_targets());

        // Test with poor cache performance
        let mut bad_metrics = PerformanceMetrics::new();
        for _ in 0..20 {
            bad_metrics.record_analysis(Duration::from_millis(30), false);
        }

        assert!(!bad_metrics.meets_performance_targets());
    }

    #[tokio::test]
    async fn test_cache_optimization() {
        let mut analyzer = OptimizedSemanticAnalyzer::with_cache_settings(
            Arc::new(EmptyModelProvider),
            Duration::from_secs(60),
            10, // Small cache size
        );

        let expr = create_simple_expression();

        // Fill cache beyond capacity
        for i in 0..15 {
            let unique_expr = ExpressionNode::Literal(LiteralValue::String(format!("test{}", i)));
            let _ = analyzer.analyze_expression_cached(&unique_expr, None).await;
        }

        // Check that cache size is limited
        let stats = analyzer.get_cache_statistics().await;
        assert!(stats.expression_cache_size <= 10);

        // Test optimization
        analyzer.optimize_cache_settings().await;

        // Cache size should be increased due to low hit ratio
        assert!(analyzer.analysis_cache.max_cache_size > 10);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let mut analyzer = create_test_analyzer();
        let expr = create_simple_expression();

        // Perform some analyses
        let _ = analyzer.analyze_expression_cached(&expr, None).await;

        let report = analyzer.generate_performance_report().await;
        assert!(report.contains("Optimized Analyzer Performance Report"));
        assert!(report.contains("Total Analyses"));
        assert!(report.contains("Cache Hit Ratio"));
        assert!(report.contains("Expression Cache Statistics"));
    }
}
*/
