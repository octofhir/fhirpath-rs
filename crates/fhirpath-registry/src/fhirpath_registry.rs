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

//! Unified Registry V2 - Next-generation async-first operation registry
//!
//! This module provides the foundational unified registry architecture that combines
//! function and operator registries into a single, high-performance, async-first system.

use crate::async_cache::AsyncLruCache;
use crate::metadata::{OperationMetadata, OperationType};
use crate::operation::FhirPathOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use rustc_hash::FxHashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache key for operation dispatch optimization
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DispatchKey {
    /// Operation identifier (function name or operator symbol)
    pub identifier: String,
    /// Number of arguments (for overload resolution)
    pub arg_count: usize,
    /// Type signature hash for fast type-based dispatch
    pub type_signature: u64,
}

/// Performance metrics for registry operations
#[derive(Debug, Clone, Default)]
pub struct RegistryMetrics {
    /// Total number of operation lookups
    pub lookups: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Total evaluation time in nanoseconds
    pub total_eval_time_ns: u64,
    /// Number of successful evaluations
    pub successful_evaluations: u64,
    /// Number of failed evaluations
    pub failed_evaluations: u64,
}

impl RegistryMetrics {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.lookups as f64
        }
    }

    /// Calculate average evaluation time in nanoseconds
    pub fn avg_eval_time_ns(&self) -> f64 {
        if self.successful_evaluations == 0 {
            0.0
        } else {
            self.total_eval_time_ns as f64 / self.successful_evaluations as f64
        }
    }

    /// Calculate success ratio
    pub fn success_ratio(&self) -> f64 {
        let total = self.successful_evaluations + self.failed_evaluations;
        if total == 0 {
            0.0
        } else {
            self.successful_evaluations as f64 / total as f64
        }
    }
}

/// LSP provider for operation metadata and completion
#[derive(Debug, Clone)]
pub struct OperationLspProvider {
    /// Fast lookup for completion items
    completion_cache: Arc<FxHashMap<String, Vec<String>>>,
    /// Documentation cache for hover information
    documentation_cache: Arc<FxHashMap<String, String>>,
}

impl OperationLspProvider {
    /// Create new LSP provider
    pub fn new() -> Self {
        Self {
            completion_cache: Arc::new(FxHashMap::default()),
            documentation_cache: Arc::new(FxHashMap::default()),
        }
    }

    /// Get completion suggestions for given prefix
    pub fn get_completions(&self, prefix: &str) -> Vec<String> {
        self.completion_cache
            .get(prefix)
            .cloned()
            .unwrap_or_default()
    }

    /// Get documentation for operation
    pub fn get_documentation(&self, identifier: &str) -> Option<String> {
        self.documentation_cache.get(identifier).cloned()
    }
}

impl Default for OperationLspProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Single unified registry for all FHIRPath operations
///
/// This registry combines functions and operators into a single, async-first,
/// high-performance system with optimized dispatch and caching.
#[derive(Clone)]
pub struct FhirPathRegistry {
    /// All callable items (functions + operators) indexed by symbol
    operations: Arc<RwLock<FxHashMap<String, Arc<dyn FhirPathOperation>>>>,

    /// Enhanced metadata with unified type information
    metadata: Arc<RwLock<FxHashMap<String, OperationMetadata>>>,

    /// Performance-optimized async dispatch cache
    dispatch_cache: Arc<AsyncLruCache<DispatchKey, Arc<dyn FhirPathOperation>>>,

    /// LSP and tooling support
    lsp_provider: Arc<OperationLspProvider>,

    /// Performance statistics and metrics
    metrics: Arc<RwLock<RegistryMetrics>>,
}

impl FhirPathRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(FxHashMap::default())),
            metadata: Arc::new(RwLock::new(FxHashMap::default())),
            dispatch_cache: Arc::new(AsyncLruCache::new(1000)), // 1000 entry cache
            lsp_provider: Arc::new(OperationLspProvider::new()),
            metrics: Arc::new(RwLock::new(RegistryMetrics::default())),
        }
    }

    /// Register a new operation in the registry
    pub async fn register<T>(&self, operation: T) -> Result<()>
    where
        T: FhirPathOperation + 'static,
    {
        let operation = Arc::new(operation);
        let identifier = operation.identifier().to_string();
        let metadata = operation.metadata().clone();

        // Register operation
        {
            let mut ops = self.operations.write().await;
            ops.insert(identifier.clone(), operation);
        }

        // Register metadata
        {
            let mut meta = self.metadata.write().await;
            meta.insert(identifier.clone(), metadata);
        }

        // Clear dispatch cache to ensure new operation is available
        self.dispatch_cache.clear().await;

        Ok(())
    }

    /// Register multiple operations at once
    pub async fn register_bulk<I, T>(&self, operations: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: FhirPathOperation + 'static,
    {
        let mut ops_map = FxHashMap::default();
        let mut meta_map = FxHashMap::default();

        // Prepare all operations
        for operation in operations {
            let operation = Arc::new(operation);
            let identifier = operation.identifier().to_string();
            let metadata = operation.metadata().clone();

            ops_map.insert(identifier.clone(), operation);
            meta_map.insert(identifier, metadata);
        }

        // Batch insert operations
        {
            let mut ops = self.operations.write().await;
            for (id, op) in ops_map {
                ops.insert(id, op);
            }
        }

        // Batch insert metadata
        {
            let mut meta = self.metadata.write().await;
            for (id, metadata) in meta_map {
                meta.insert(id, metadata);
            }
        }

        // Clear dispatch cache
        self.dispatch_cache.clear().await;

        Ok(())
    }

    /// Check if an operation is registered
    pub async fn contains(&self, identifier: &str) -> bool {
        let ops = self.operations.read().await;
        ops.contains_key(identifier)
    }

    /// Get operation by identifier
    pub async fn get_operation(&self, identifier: &str) -> Option<Arc<dyn FhirPathOperation>> {
        // Try cache first
        let cache_key = DispatchKey {
            identifier: identifier.to_string(),
            arg_count: 0,      // Will be refined for actual dispatch
            type_signature: 0, // Will be refined for actual dispatch
        };

        if let Some(cached) = self.dispatch_cache.get(&cache_key).await {
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.lookups += 1;
                metrics.cache_hits += 1;
            }
            return Some(cached);
        }

        // Cache miss - lookup in main registry
        let ops = self.operations.read().await;
        if let Some(operation) = ops.get(identifier).cloned() {
            // Cache for future use
            self.dispatch_cache
                .insert(cache_key, operation.clone())
                .await;

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.lookups += 1;
                metrics.cache_misses += 1;
            }

            Some(operation)
        } else {
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.lookups += 1;
                metrics.cache_misses += 1;
            }
            None
        }
    }

    /// Get operation metadata
    pub async fn get_metadata(&self, identifier: &str) -> Option<OperationMetadata> {
        let meta = self.metadata.read().await;
        meta.get(identifier).cloned()
    }

    /// Check if an operation is a lambda function
    ///
    /// Lambda functions require raw expressions instead of pre-evaluated arguments
    /// and support lambda-specific variables like $this, $index, $total.
    pub async fn is_lambda_function(&self, identifier: &str) -> bool {
        // Fast path: check known lambda function names
        if matches!(
            identifier,
            "where" | "select" | "all" | "aggregate" | "repeat" | "sort" | "iif"
        ) {
            return true;
        }

        // Main lambda functions are now handled directly in the engine
        match identifier {
            "where" | "select" | "sort" | "repeat" | "aggregate" | "all" | "iif" => true,
            _ => {
                // Check remaining registry lambda functions
                if let Some(_operation) = self.get_operation(identifier).await {
                    // For now, assume all remaining operations in registry might be lambda functions
                    // This will be refined as we add specific lambda functions to engine
                    false // No specific lambda functions left in registry for now
                } else {
                    false
                }
            }
        }
    }

    /// List all registered operations
    pub async fn list_operations(&self) -> Vec<String> {
        let ops = self.operations.read().await;
        ops.keys().cloned().collect()
    }

    /// List operations by type
    pub async fn list_operations_by_type(&self, operation_type: OperationType) -> Vec<String> {
        let meta = self.metadata.read().await;
        meta.iter()
            .filter(|(_, metadata)| {
                std::mem::discriminant(&metadata.basic.operation_type)
                    == std::mem::discriminant(&operation_type)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> RegistryMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Reset performance metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = RegistryMetrics::default();
    }

    /// Get LSP provider for tooling support
    pub fn get_lsp_provider(&self) -> Arc<OperationLspProvider> {
        self.lsp_provider.clone()
    }

    /// Clear dispatch cache
    pub async fn clear_cache(&self) {
        self.dispatch_cache.clear().await;
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        // Cache size and capacity would be exposed by AsyncLruCache
        // For now return placeholder values
        (0, 1000)
    }

    /// Validate registry consistency
    pub async fn validate(&self) -> Result<()> {
        let ops = self.operations.read().await;
        let meta = self.metadata.read().await;

        // Check that every operation has metadata
        for identifier in ops.keys() {
            if !meta.contains_key(identifier) {
                return Err(FhirPathError::evaluation_error(format!(
                    "Operation '{identifier}' missing metadata"
                )));
            }
        }

        // Check that every metadata entry has an operation
        for identifier in meta.keys() {
            if !ops.contains_key(identifier) {
                return Err(FhirPathError::evaluation_error(format!(
                    "Metadata for '{identifier}' has no corresponding operation"
                )));
            }
        }

        Ok(())
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let ops = self.operations.read().await;
        let meta = self.metadata.read().await;
        let metrics = self.metrics.read().await;

        let mut function_count = 0;
        let mut operator_count = 0;
        let mut sync_count = 0;
        let mut async_count = 0;

        for (_, metadata) in meta.iter() {
            match metadata.basic.operation_type {
                OperationType::Function => function_count += 1,
                OperationType::BinaryOperator { .. } | OperationType::UnaryOperator => {
                    operator_count += 1
                }
            }

            if metadata.performance.supports_sync {
                sync_count += 1;
            } else {
                async_count += 1;
            }
        }

        RegistryStats {
            total_operations: ops.len(),
            function_count,
            operator_count,
            sync_operations: sync_count,
            async_operations: async_count,
            cache_hit_ratio: metrics.cache_hit_ratio(),
            avg_eval_time_ns: metrics.avg_eval_time_ns(),
            success_ratio: metrics.success_ratio(),
        }
    }
}

impl Default for FhirPathRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of registered operations
    pub total_operations: usize,
    /// Number of functions
    pub function_count: usize,
    /// Number of operators
    pub operator_count: usize,
    /// Number of operations supporting sync evaluation
    pub sync_operations: usize,
    /// Number of async-only operations
    pub async_operations: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Average evaluation time in nanoseconds
    pub avg_eval_time_ns: f64,
    /// Success ratio (0.0 to 1.0)
    pub success_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{
        BasicOperationInfo, FunctionMetadata, OperationSpecificMetadata, PerformanceMetadata,
        TypeConstraints,
    };
    use crate::operation::FhirPathOperation;
    use crate::operations::EvaluationContext;
    use async_trait::async_trait;
    use octofhir_fhirpath_model::FhirPathValue;
    use std::any::Any;

    // Mock operation for testing
    struct TestFunction;

    #[async_trait]
    impl FhirPathOperation for TestFunction {
        fn identifier(&self) -> &str {
            "test"
        }

        fn operation_type(&self) -> OperationType {
            OperationType::Function
        }

        fn metadata(&self) -> &OperationMetadata {
            static METADATA: once_cell::sync::Lazy<OperationMetadata> =
                once_cell::sync::Lazy::new(|| OperationMetadata {
                    basic: BasicOperationInfo {
                        name: "test".to_string(),
                        operation_type: OperationType::Function,
                        description: "Test function".to_string(),
                        examples: vec!["test()".to_string()],
                    },
                    types: TypeConstraints::default(),
                    performance: PerformanceMetadata {
                        complexity: crate::metadata::PerformanceComplexity::Constant,
                        supports_sync: true,
                        avg_time_ns: 100,
                        memory_usage: 64,
                    },
                    specific: OperationSpecificMetadata::Function(FunctionMetadata::default()),
                });
            &METADATA
        }

        async fn evaluate(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            Ok(FhirPathValue::String("test".into()))
        }

        fn validate_args(&self, _args: &[FhirPathValue]) -> Result<()> {
            Ok(())
        }

        fn supports_sync(&self) -> bool {
            true
        }

        fn try_evaluate_sync(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Option<Result<FhirPathValue>> {
            Some(Ok(FhirPathValue::String("test".into())))
        }

        fn as_any(&self) -> &dyn Any {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = FhirPathRegistry::new();
        assert_eq!(registry.list_operations().await.len(), 0);
    }

    #[tokio::test]
    async fn test_operation_registration() {
        let registry = FhirPathRegistry::new();

        // Register test function
        registry.register(TestFunction).await.unwrap();

        // Verify registration
        assert!(registry.contains("test").await);
        assert_eq!(registry.list_operations().await.len(), 1);

        // Verify operation retrieval
        let operation = registry.get_operation("test").await;
        assert!(operation.is_some());

        // Verify metadata retrieval
        let metadata = registry.get_metadata("test").await;
        assert!(metadata.is_some());
    }

    #[tokio::test]
    async fn test_bulk_registration() {
        let registry = FhirPathRegistry::new();

        let operations = vec![TestFunction];
        registry.register_bulk(operations).await.unwrap();

        assert!(registry.contains("test").await);
        assert_eq!(registry.list_operations().await.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_validation() {
        let registry = FhirPathRegistry::new();
        registry.register(TestFunction).await.unwrap();

        // Should validate successfully
        registry.validate().await.unwrap();
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let registry = FhirPathRegistry::new();
        registry.register(TestFunction).await.unwrap();

        let stats = registry.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.function_count, 1);
        assert_eq!(stats.operator_count, 0);
        assert_eq!(stats.sync_operations, 1);
        assert_eq!(stats.async_operations, 0);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let registry = FhirPathRegistry::new();
        registry.register(TestFunction).await.unwrap();

        // First lookup - cache miss
        let op1 = registry.get_operation("test").await;
        assert!(op1.is_some());

        // Second lookup - should be cache hit
        let op2 = registry.get_operation("test").await;
        assert!(op2.is_some());

        let metrics = registry.get_metrics().await;
        assert_eq!(metrics.lookups, 2);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
    }

    #[tokio::test]
    async fn test_operations_by_type() {
        let registry = FhirPathRegistry::new();
        registry.register(TestFunction).await.unwrap();

        let functions = registry
            .list_operations_by_type(OperationType::Function)
            .await;
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], "test");

        let operators = registry
            .list_operations_by_type(OperationType::UnaryOperator)
            .await;
        assert_eq!(operators.len(), 0);
    }
}
