//! Function Registry for FHIRPath Operations
//!
//! This module provides a function registry that can handle both synchronous and asynchronous
//! FHIRPath operations, automatically dispatching to the appropriate implementation for
//! optimal performance. Uses the new RegistryCore-based architecture for better performance
//! and caching.

use crate::registry::{SyncRegistry, AsyncRegistry};
use crate::signature::FunctionSignature;
use crate::traits::{AsyncOperation, EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Function type for caching dispatch decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    Sync,
    Async,
}

/// Function registry that manages both sync and async FHIRPath operations
///
/// The registry automatically dispatches to sync operations when possible for better
/// performance, falling back to async operations when necessary. Now uses the optimized
/// RegistryCore architecture with caching and fast lookups.
pub struct FunctionRegistry {
    sync_registry: Arc<SyncRegistry>,
    async_registry: Arc<AsyncRegistry>,
    // Cached function lookup for faster dispatch
    function_cache: Arc<RwLock<HashMap<String, FunctionType>>>,
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self {
            sync_registry: Arc::new(SyncRegistry::new()),
            async_registry: Arc::new(AsyncRegistry::new()),
            function_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a function registry from existing registries
    pub fn from_registries(
        sync_registry: Arc<SyncRegistry>,
        async_registry: Arc<AsyncRegistry>,
    ) -> Self {
        Self {
            sync_registry,
            async_registry,
            function_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a synchronous operation
    pub async fn register_sync(&self, operation: Box<dyn SyncOperation>) {
        let name = operation.name().to_string();
        self.sync_registry.register(operation).await;
        
        // Update cache
        let mut cache = self.function_cache.write().await;
        cache.insert(name, FunctionType::Sync);
    }

    /// Register an asynchronous operation
    pub async fn register_async(&self, operation: Box<dyn AsyncOperation>) {
        let name = operation.name().to_string();
        self.async_registry.register(operation).await;
        
        // Update cache
        let mut cache = self.function_cache.write().await;
        cache.insert(name, FunctionType::Async);
    }

    /// Register multiple synchronous operations at once
    pub async fn register_sync_many(&self, operations: Vec<Box<dyn SyncOperation>>) {
        let mut cache = self.function_cache.write().await;
        for operation in &operations {
            let name = operation.name().to_string();
            cache.insert(name, FunctionType::Sync);
        }
        drop(cache); // Release lock early
        
        self.sync_registry.register_many(operations).await;
    }

    /// Register multiple asynchronous operations at once  
    pub async fn register_async_many(&self, operations: Vec<Box<dyn AsyncOperation>>) {
        let mut cache = self.function_cache.write().await;
        for operation in &operations {
            let name = operation.name().to_string();
            cache.insert(name, FunctionType::Async);
        }
        drop(cache); // Release lock early
        
        self.async_registry.register_many(operations).await;
    }

    /// Evaluate a function by name with smart dispatch and caching
    ///
    /// This method uses a cached lookup to quickly determine if a function is sync or async,
    /// then dispatches to the appropriate registry for optimal performance.
    pub async fn evaluate(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Fast path: check cache first
        {
            let cache = self.function_cache.read().await;
            if let Some(function_type) = cache.get(name) {
                return match function_type {
                    FunctionType::Sync => self.sync_registry.execute(name, args, context).await,
                    FunctionType::Async => self.async_registry.execute(name, args, context).await,
                };
            }
        }

        // Slow path: function not in cache, check registries and update cache
        if self.sync_registry.contains(name).await {
            // Update cache for future calls
            {
                let mut cache = self.function_cache.write().await;
                cache.insert(name.to_string(), FunctionType::Sync);
            }
            return self.sync_registry.execute(name, args, context).await;
        }

        if self.async_registry.contains(name).await {
            // Update cache for future calls
            {
                let mut cache = self.function_cache.write().await;
                cache.insert(name.to_string(), FunctionType::Async);
            }
            return self.async_registry.execute(name, args, context).await;
        }

        // Function not found in either registry
        Err(FhirPathError::UnknownFunction {
            function_name: name.to_string(),
        })
    }

    /// Try to evaluate synchronously only
    ///
    /// Returns None if the operation requires async execution or doesn't exist
    pub async fn try_evaluate_sync(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Check cache first
        {
            let cache = self.function_cache.read().await;
            if let Some(function_type) = cache.get(name) {
                return match function_type {
                    FunctionType::Sync => Some(self.sync_registry.execute(name, args, context).await),
                    FunctionType::Async => None,
                };
            }
        }

        // Check if sync operation exists
        if self.sync_registry.contains(name).await {
            // Update cache
            {
                let mut cache = self.function_cache.write().await;
                cache.insert(name.to_string(), FunctionType::Sync);
            }
            Some(self.sync_registry.execute(name, args, context).await)
        } else {
            None
        }
    }

    /// Check if a function exists (sync or async)
    pub async fn has_function(&self, name: &str) -> bool {
        // Check cache first
        {
            let cache = self.function_cache.read().await;
            if cache.contains_key(name) {
                return true;
            }
        }

        // Check registries
        self.sync_registry.contains(name).await || self.async_registry.contains(name).await
    }

    /// Check if a function supports synchronous execution
    pub async fn supports_sync(&self, name: &str) -> bool {
        // Check cache first
        {
            let cache = self.function_cache.read().await;
            if let Some(function_type) = cache.get(name) {
                return matches!(function_type, FunctionType::Sync);
            }
        }

        // Check sync registry directly
        self.sync_registry.contains(name).await
    }

    /// Get list of all function names
    pub async fn function_names(&self) -> Vec<String> {
        let mut names = self.sync_registry.get_operation_names().await;
        let async_names = self.async_registry.get_operation_names().await;
        names.extend(async_names);
        names.sort();
        names.dedup();
        names
    }

    /// Get function signature by name  
    pub async fn get_function_signature(&self, name: &str) -> Option<FunctionSignature> {
        // Try sync registry first
        if let Some(signature) = self.sync_registry.get_signature(name).await {
            return Some(signature);
        }

        // Try async registry
        self.async_registry.get_signature(name).await
    }

    /// Get statistics about the registry
    pub async fn stats(&self) -> RegistryStats {
        let sync_names = self.sync_registry.get_operation_names().await;
        let async_names = self.async_registry.get_operation_names().await;
        let cache = self.function_cache.read().await;
        
        RegistryStats {
            sync_operations: sync_names.len(),
            async_operations: async_names.len(),
            total_operations: sync_names.len() + async_names.len(),
            cached_functions: cache.len(),
            cache_hit_potential: if sync_names.len() + async_names.len() > 0 {
                (cache.len() as f64 / (sync_names.len() + async_names.len()) as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Clear the function cache (useful for testing or reconfiguration)
    pub async fn clear_cache(&self) {
        let mut cache = self.function_cache.write().await;
        cache.clear();
    }

    /// Warm the cache by pre-loading function types
    /// 
    /// This method populates the cache with all available functions for optimal performance
    pub async fn warm_cache(&self) {
        let sync_names = self.sync_registry.get_operation_names().await;
        let async_names = self.async_registry.get_operation_names().await;
        
        let mut cache = self.function_cache.write().await;
        for name in sync_names {
            cache.insert(name, FunctionType::Sync);
        }
        for name in async_names {
            cache.insert(name, FunctionType::Async);
        }
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about operations in the registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub sync_operations: usize,
    pub async_operations: usize,
    pub total_operations: usize,
    pub cached_functions: usize,
    pub cache_hit_potential: f64,
}

impl RegistryStats {
    pub fn sync_percentage(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.sync_operations as f64 / self.total_operations as f64) * 100.0
        }
    }
}

impl std::fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registry: {} sync, {} async, {} total ({:.1}% sync), {} cached ({:.1}% cache coverage)",
            self.sync_operations,
            self.async_operations,
            self.total_operations,
            self.sync_percentage(),
            self.cached_functions,
            self.cache_hit_potential
        )
    }
}

/// Create a registry with all standard FHIRPath operations
pub async fn create_standard_registry() -> FunctionRegistry {
    let registry = FunctionRegistry::new();

    // Register sync string operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::string_sync::SimpleLengthFunction),
        Box::new(crate::operations::string_sync::SimpleUpperFunction),
        Box::new(crate::operations::string_sync::SimpleLowerFunction),
        Box::new(crate::operations::string_sync::SimpleContainsFunction),
        Box::new(crate::operations::string_sync::SimpleStartsWithFunction),
        Box::new(crate::operations::string_sync::SimpleEndsWithFunction),
        Box::new(crate::operations::string_sync::SimpleIndexOfFunction),
        Box::new(crate::operations::string_sync::SimpleLastIndexOfFunction),
        Box::new(crate::operations::string_sync::SimpleSubstringFunction),
        Box::new(crate::operations::string_sync::SimpleReplaceFunction),
        Box::new(crate::operations::string_sync::SimpleSplitFunction),
        Box::new(crate::operations::string_sync::SimpleJoinFunction),
        Box::new(crate::operations::string_sync::SimpleTrimFunction),
        Box::new(crate::operations::string_sync::SimpleToCharsFunction),
        Box::new(crate::operations::string_sync::SimpleMatchesFunction),
        Box::new(crate::operations::string_sync::SimpleMatchesFullFunction),
        Box::new(crate::operations::string_sync::SimpleReplaceMatchesFunction),
    ]).await;

    // Register sync math operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::math_sync::SimpleAbsFunction),
        Box::new(crate::operations::math_sync::SimpleCeilingFunction),
        Box::new(crate::operations::math_sync::SimpleFloorFunction),
        Box::new(crate::operations::math_sync::SimpleRoundFunction),
        Box::new(crate::operations::math_sync::SimpleTruncateFunction),
        Box::new(crate::operations::math_sync::SimpleSqrtFunction),
        Box::new(crate::operations::math_sync::SimplePowerFunction),
        Box::new(crate::operations::math_sync::SimpleLnFunction),
        Box::new(crate::operations::math_sync::SimpleLogFunction),
        Box::new(crate::operations::math_sync::SimpleExpFunction),
        Box::new(crate::operations::math_sync::SimplePrecisionFunction),
        Box::new(crate::operations::math_sync::SimpleAddFunction),
        Box::new(crate::operations::math_sync::SimpleSubtractFunction),
        Box::new(crate::operations::math_sync::SimpleMultiplyFunction),
        Box::new(crate::operations::math_sync::SimpleDivideFunction),
        Box::new(crate::operations::math_sync::SimpleModuloFunction),
    ]).await;

    // Register sync collection operations using batch registration
    registry.register_sync_many(vec![
        Box::new(crate::operations::collection_sync::SimpleCountFunction),
        Box::new(crate::operations::collection_sync::SimpleEmptyFunction),
        // Box::new(crate::operations::collection_sync::SimpleExistsFunction::default()), // Disabled: use lambda version
        Box::new(crate::operations::collection_sync::SimpleFirstFunction),
        Box::new(crate::operations::collection_sync::SimpleLastFunction),
        Box::new(crate::operations::collection_sync::SimpleTailFunction),
        Box::new(crate::operations::collection_sync::SimpleSkipFunction),
        Box::new(crate::operations::collection_sync::SimpleTakeFunction),
        Box::new(crate::operations::collection_sync::SimpleSingleFunction),
        Box::new(crate::operations::collection_sync::SimpleDistinctFunction),
        Box::new(crate::operations::collection_sync::SimpleIsDistinctFunction),
        Box::new(crate::operations::collection_sync::SimpleUnionFunction),
        Box::new(crate::operations::collection_sync::SimpleIntersectFunction),
        Box::new(crate::operations::collection_sync::SimpleExcludeFunction),
        Box::new(crate::operations::collection_sync::SimpleSubsetOfFunction),
        Box::new(crate::operations::collection_sync::SimpleSupersetOfFunction),
        Box::new(crate::operations::collection_sync::SimpleAllTrueFunction),
        Box::new(crate::operations::collection_sync::SimpleAnyTrueFunction),
        Box::new(crate::operations::collection_sync::SimpleAllFalseFunction),
        Box::new(crate::operations::collection_sync::SimpleAnyFalseFunction),
        Box::new(crate::operations::collection_sync::SimpleCombineFunction),
    ]).await;

    // Register sync datetime extraction operations (from Task 24)
    registry.register_sync_many(vec![
        Box::new(crate::operations::datetime_sync::DayOfFunction),
        Box::new(crate::operations::datetime_sync::HourOfFunction),
        Box::new(crate::operations::datetime_sync::MinuteOfFunction),
        Box::new(crate::operations::datetime_sync::SecondOfFunction),
        Box::new(crate::operations::datetime_sync::MillisecondOfFunction),
        Box::new(crate::operations::datetime_sync::MonthOfFunction),
        Box::new(crate::operations::datetime_sync::YearOfFunction),
        Box::new(crate::operations::datetime_sync::TimezoneOffsetOfFunction),
        Box::new(crate::operations::datetime_sync::TimeOfDayFunction),
        Box::new(crate::operations::datetime_sync::HighBoundaryFunction),
        Box::new(crate::operations::datetime_sync::LowBoundaryFunction),
    ]).await;

    // Register sync FHIR data traversal operations (from Task 16)
    registry.register_sync_many(vec![
        Box::new(crate::operations::fhir_sync::ChildrenFunction),
        Box::new(crate::operations::fhir_sync::DescendantsFunction),
    ]).await;

    // Register sync utility operations (from Task 23)
    registry.register_sync_many(vec![
        Box::new(crate::operations::utility_sync::HasValueFunction),
        Box::new(crate::operations::utility_sync::ComparableFunction),
        Box::new(crate::operations::utility_sync::EncodeFunction),
        Box::new(crate::operations::utility_sync::DecodeFunction),
        Box::new(crate::operations::utility_sync::EscapeFunction),
        Box::new(crate::operations::utility_sync::UnescapeFunction),
        Box::new(crate::operations::utility_sync::TraceFunction),
        Box::new(crate::operations::utility_sync::DefineVariableFunction),
    ]).await;

    // Register sync logical operations (from Task 23)
    registry.register_sync_many(vec![Box::new(
        crate::operations::logical_sync::NotOperation,
    )]).await;

    // Register async datetime system call operations (from Task 24) using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::datetime_async::NowFunction),
        Box::new(crate::operations::datetime_async::TodayFunction),
    ]).await;

    // Register async FHIR ModelProvider operations (from Task 16) using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::fhir_async::ResolveFunction),
        Box::new(crate::operations::fhir_async::ConformsToFunction),
        Box::new(crate::operations::fhir_async::ExtensionFunction),
    ]).await;

    // Register async type operations using batch registration
    registry.register_async_many(vec![
        Box::new(crate::operations::types_async::TypeFunction),
        Box::new(crate::operations::types_async::IsOperation),
        Box::new(crate::operations::types_async::OfTypeFunction),
        Box::new(crate::operations::types_async::AsOperation),
    ]).await;

    // Register sync CDA operations
    registry.register_sync_many(vec![Box::new(
        crate::operations::cda_sync::HasTemplateIdOfFunction,
    )]).await;

    // Register sync conversion operations using batch registration
    registry.register_sync_many(vec![
        // Type checking operations (converts_to_*)
        Box::new(crate::operations::conversion_sync::ConvertsToBooleanFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDateFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDateTimeFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToDecimalFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToIntegerFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToLongFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToQuantityFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToStringFunction),
        Box::new(crate::operations::conversion_sync::ConvertsToTimeFunction),
        // Type conversion operations (to_*)
        Box::new(crate::operations::conversion_sync::ToBooleanFunction),
        Box::new(crate::operations::conversion_sync::ToDateFunction),
        Box::new(crate::operations::conversion_sync::ToDateTimeFunction),
        Box::new(crate::operations::conversion_sync::ToDecimalFunction),
        Box::new(crate::operations::conversion_sync::ToIntegerFunction),
        Box::new(crate::operations::conversion_sync::ToLongFunction),
        Box::new(crate::operations::conversion_sync::ToQuantityFunction),
        Box::new(crate::operations::conversion_sync::ToStringFunction),
        Box::new(crate::operations::conversion_sync::ToTimeFunction),
    ]).await;

    // Warm the cache for optimal performance
    registry.warm_cache().await;

    registry
}
