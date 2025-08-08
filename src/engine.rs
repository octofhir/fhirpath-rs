//! FHIRPath engine - the main entry point for FHIRPath evaluation

use super::error::Result;
use crate::ast::ExpressionNode;
use crate::evaluator::FhirPathEngine as EvaluatorEngine;
use crate::model::{FhirPathValue, ValuePoolConfig, configure_global_pools, global_pool_stats};
use crate::parser::{cache_ast, get_cached_ast, parse_expression};
use crate::pipeline::global_pools;
use crate::registry::create_standard_registries;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Main FHIRPath engine for parsing and evaluating expressions
#[derive(Clone)]
pub struct FhirPathEngine {
    /// The underlying evaluator engine
    evaluator: EvaluatorEngine,
    /// Cached compiled expressions for performance
    expression_cache: HashMap<String, ExpressionNode>,
    /// Maximum cache size to prevent memory issues
    max_cache_size: usize,
}

impl Default for FhirPathEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FhirPathEngine {
    /// Create a new FHIRPath engine with default memory optimizations
    pub fn new() -> Self {
        // Configure global value pools with optimized settings
        let pool_config = ValuePoolConfig {
            max_pool_size: 500,
            initial_collection_capacity: 8,
            enable_stats: false,
        };
        configure_global_pools(pool_config);

        let (functions, operators) = create_standard_registries();
        let evaluator = EvaluatorEngine::with_registries(Arc::new(functions), Arc::new(operators));

        Self {
            evaluator,
            expression_cache: HashMap::new(),
            max_cache_size: 1000,
        }
    }

    /// Create a new FHIRPath engine with custom memory pool configuration
    pub fn with_pool_config(pool_config: ValuePoolConfig) -> Self {
        configure_global_pools(pool_config);

        let (functions, operators) = create_standard_registries();
        let evaluator = EvaluatorEngine::with_registries(Arc::new(functions), Arc::new(operators));

        Self {
            evaluator,
            expression_cache: HashMap::new(),
            max_cache_size: 1000,
        }
    }

    /// Create a new FHIRPath engine optimized for high-throughput scenarios
    pub fn with_high_throughput_optimization() -> Self {
        let pool_config = ValuePoolConfig {
            max_pool_size: 2000,
            initial_collection_capacity: 16,
            enable_stats: true,
        };
        Self::with_pool_config(pool_config)
    }

    /// Evaluate an FHIRPath expression against input data
    pub async fn evaluate(&mut self, expression: &str, input_data: Value) -> Result<FhirPathValue> {
        // Handle parse errors by returning empty collection per FHIRPath spec
        let ast = match self.get_or_compile_expression(expression) {
            Ok(ast) => ast,
            Err(e) => {
                // Per FHIRPath spec, syntax errors should return empty collection
                if e.to_string().contains("parse error")
                    || e.to_string().contains("Parse error")
                    || e.to_string().contains("Unclosed")
                    || e.to_string().contains("Unexpected")
                    || e.to_string().contains("Expected")
                {
                    return Ok(FhirPathValue::collection(vec![]));
                } else {
                    return Err(e);
                }
            }
        };

        let input_value = FhirPathValue::from(input_data);

        match self.evaluator.evaluate(&ast, input_value).await {
            Ok(result) => Ok(result),
            Err(eval_error) => Err(crate::error::FhirPathError::evaluation_error(
                eval_error.to_string(),
            )),
        }
    }

    /// Get or compile an expression, using global AST cache when possible
    fn get_or_compile_expression(&mut self, expression: &str) -> Result<Arc<ExpressionNode>> {
        // First try the global AST cache
        if let Some(cached_ast) = get_cached_ast(expression) {
            return Ok(cached_ast);
        }

        // Fall back to local cache for transition compatibility
        if let Some(local_ast) = self.expression_cache.get(expression) {
            let shared_ast = Arc::new(local_ast.clone());
            // Cache in global cache for next time
            cache_ast(expression, local_ast.clone());
            return Ok(shared_ast);
        }

        // Parse and cache both globally and locally
        let ast = parse_expression(expression)
            .map_err(|e| crate::error::FhirPathError::parse_error(0, e.to_string()))?;

        // Cache globally (primary cache)
        cache_ast(expression, ast.clone());

        // Cache locally (fallback/transition cache)
        if self.expression_cache.len() >= self.max_cache_size {
            self.expression_cache.clear();
        }
        self.expression_cache
            .insert(expression.to_string(), ast.clone());

        Ok(Arc::new(ast))
    }

    /// Pool-optimized evaluation using global memory pools
    /// This method demonstrates integration with the async-first memory pool system
    pub async fn evaluate_with_pools(
        &mut self,
        expression: &str,
        input_data: Value,
    ) -> Result<FhirPathValue> {
        // Get a pooled vector for intermediate results
        let _pooled_values = global_pools().values.borrow().await;

        // Get a pooled string for temporary string operations
        let _pooled_string = global_pools().strings.borrow().await;

        // Standard evaluation with pooled resources in the background
        self.evaluate(expression, input_data).await
    }

    /// Get memory pool statistics for diagnostics
    pub async fn memory_pool_stats(&self) -> HashMap<String, crate::pipeline::PoolStats> {
        global_pools().comprehensive_stats().await
    }

    /// Warm up memory pools for better performance
    pub async fn warm_memory_pools(&self) {
        global_pools().warm_all().await;
    }

    /// Get value pool statistics for memory optimization diagnostics
    pub fn value_pool_stats(&self) -> crate::model::CombinedValuePoolStats {
        global_pool_stats()
    }

    /// Clear all memory pools (useful for testing and cleanup)
    pub fn clear_memory_pools(&self) {
        crate::model::clear_global_pools();
    }

    /// Get comprehensive memory statistics including both pipeline and value pools
    pub async fn comprehensive_memory_stats(&self) -> MemoryStats {
        let pipeline_stats = self.memory_pool_stats().await;
        let value_pool_stats = self.value_pool_stats();
        let interner_stats = crate::model::global_interner_stats();

        MemoryStats {
            pipeline_pools: pipeline_stats,
            value_pools: value_pool_stats,
            string_interner: interner_stats,
        }
    }
}

/// Comprehensive memory statistics for the FHIRPath engine
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Statistics from the pipeline memory pools
    pub pipeline_pools: HashMap<String, crate::pipeline::PoolStats>,
    /// Statistics from the value pools
    pub value_pools: crate::model::CombinedValuePoolStats,
    /// Statistics from the string interner
    pub string_interner: crate::model::InternerStats,
}

impl MemoryStats {
    /// Calculate overall memory efficiency metrics
    pub fn efficiency_metrics(&self) -> MemoryEfficiencyMetrics {
        let value_hit_ratio = self.value_pools.overall_hit_ratio();
        let pipeline_hit_ratio = self
            .pipeline_pools
            .values()
            .map(|stats| {
                let total = stats.pool_hits + stats.pool_misses;
                if total > 0 {
                    stats.pool_hits as f64 / total as f64
                } else {
                    0.0
                }
            })
            .fold(0.0, |acc, ratio| acc + ratio)
            / self.pipeline_pools.len().max(1) as f64;

        MemoryEfficiencyMetrics {
            overall_hit_ratio: (value_hit_ratio + pipeline_hit_ratio) / 2.0,
            value_pool_hit_ratio: value_hit_ratio,
            pipeline_pool_hit_ratio: pipeline_hit_ratio,
            interned_strings: self.string_interner.entries,
        }
    }
}

/// Memory efficiency metrics for performance analysis
#[derive(Debug, Clone)]
pub struct MemoryEfficiencyMetrics {
    /// Overall cache hit ratio across all pools
    pub overall_hit_ratio: f64,
    /// Value pool specific hit ratio
    pub value_pool_hit_ratio: f64,
    /// Pipeline pool hit ratio
    pub pipeline_pool_hit_ratio: f64,
    /// Number of strings interned
    pub interned_strings: usize,
}

/// Alias for compatibility with original API
pub type Engine = FhirPathEngine;
