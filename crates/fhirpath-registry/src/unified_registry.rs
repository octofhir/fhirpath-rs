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

//! High-performance unified function registry

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern};
use crate::function::{CompletionVisibility, EvaluationContext, FunctionCategory, FunctionError, FunctionResult};
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use octofhir_fhirpath_model::FhirPathValue;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Type applicability cache key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TypeApplicabilityKey {
    function_name: String,
    context_type: Option<String>,
    is_collection: bool,
}

/// Registry compilation and runtime statistics
#[derive(Debug, Clone)]
struct RegistryCompilationStats {
    total_functions: usize,
    sync_functions: usize,
    async_functions: usize,
    sync_first_functions: usize,
    metadata_entries: usize,
    cache_hits: u64,
    cache_misses: u64,
    last_compilation_time: Option<Instant>,
}

impl RegistryCompilationStats {
    fn new() -> Self {
        Self {
            total_functions: 0,
            sync_functions: 0,
            async_functions: 0,
            sync_first_functions: 0,
            metadata_entries: 0,
            cache_hits: 0,
            cache_misses: 0,
            last_compilation_time: None,
        }
    }
}

/// High-performance unified function registry with optimization
#[derive(Clone)]
pub struct UnifiedFunctionRegistry {
    /// Function implementations indexed by name
    functions: Arc<RwLock<FxHashMap<String, Arc<dyn UnifiedFhirPathFunction>>>>,
    
    /// Enhanced metadata for each function
    metadata: Arc<RwLock<FxHashMap<String, EnhancedFunctionMetadata>>>,
    
    /// Execution mode indices for fast dispatch
    sync_functions: Arc<RwLock<FxHashSet<String>>>,
    async_functions: Arc<RwLock<FxHashSet<String>>>,
    sync_first_functions: Arc<RwLock<FxHashSet<String>>>,
    
    /// Type applicability cache
    type_applicability_cache: Arc<RwLock<FxHashMap<TypeApplicabilityKey, bool>>>,
    
    /// Function categorization cache
    category_cache: Arc<RwLock<FxHashMap<FunctionCategory, Vec<String>>>>,
    
    /// LSP completion cache
    completion_cache: Arc<RwLock<FxHashMap<String, Vec<(String, EnhancedFunctionMetadata)>>>>,
    
    /// Registry statistics
    stats: Arc<Mutex<RegistryCompilationStats>>,
}

impl UnifiedFunctionRegistry {
    /// Create a new unified registry
    pub fn new() -> Self {
        Self {
            functions: Arc::new(RwLock::new(FxHashMap::default())),
            metadata: Arc::new(RwLock::new(FxHashMap::default())),
            sync_functions: Arc::new(RwLock::new(FxHashSet::default())),
            async_functions: Arc::new(RwLock::new(FxHashSet::default())),
            sync_first_functions: Arc::new(RwLock::new(FxHashSet::default())),
            type_applicability_cache: Arc::new(RwLock::new(FxHashMap::default())),
            category_cache: Arc::new(RwLock::new(FxHashMap::default())),
            completion_cache: Arc::new(RwLock::new(FxHashMap::default())),
            stats: Arc::new(Mutex::new(RegistryCompilationStats::new())),
        }
    }
    
    /// Register a unified function
    pub fn register<F: UnifiedFhirPathFunction + 'static>(&self, function: F) -> Result<(), RegistryError> {
        let name = function.name().to_string();
        let metadata = function.metadata().clone();
        let execution_mode = function.execution_mode();
        
        // Validate function name
        if name.is_empty() {
            return Err(RegistryError::InvalidFunctionName {
                name: name.clone(),
                reason: "Function name cannot be empty".to_string(),
            });
        }
        
        // Check for duplicates
        if let Ok(functions) = self.functions.read() {
            if functions.contains_key(&name) {
                return Err(RegistryError::DuplicateFunction {
                    name: name.clone(),
                });
            }
        }
        
        let arc_function = Arc::new(function);
        
        // Store function implementation
        if let Ok(mut functions) = self.functions.write() {
            functions.insert(name.clone(), arc_function);
        }
        
        // Store metadata
        if let Ok(mut metadata_map) = self.metadata.write() {
            metadata_map.insert(name.clone(), metadata);
        }
        
        // Index by execution mode for fast dispatch
        match execution_mode {
            ExecutionMode::Sync => {
                if let Ok(mut sync_funcs) = self.sync_functions.write() {
                    sync_funcs.insert(name.clone());
                }
            }
            ExecutionMode::Async => {
                if let Ok(mut async_funcs) = self.async_functions.write() {
                    async_funcs.insert(name.clone());
                }
            }
            ExecutionMode::SyncFirst => {
                if let Ok(mut sync_first_funcs) = self.sync_first_functions.write() {
                    sync_first_funcs.insert(name.clone());
                }
            }
        }
        
        // Update statistics
        self.update_compilation_stats();
        
        // Clear relevant caches
        self.clear_function_caches(&name);
        
        Ok(())
    }
    
    /// Evaluate function with optimal sync/async dispatch
    pub async fn evaluate_function(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
        prefer_sync: bool,
    ) -> FunctionResult<FhirPathValue> {
        let function = self.get_function(name)
            .ok_or_else(|| FunctionError::FunctionNotFound {
                name: name.to_string(),
            })?;
        
        // Validate arguments
        function.validate_args(args)?;
        
        // Determine execution path based on preference and capability
        let execution_mode = function.execution_mode();
        
        match (execution_mode, prefer_sync) {
            // Pure sync function - always use sync path
            (ExecutionMode::Sync, _) => function.evaluate_sync(args, context),
            
            // Pure async function - always use async path
            (ExecutionMode::Async, _) => function.evaluate_async(args, context).await,
            
            // SyncFirst function - use sync if preferred, async otherwise
            (ExecutionMode::SyncFirst, true) => {
                match function.evaluate_sync(args, context) {
                    Ok(result) => Ok(result),
                    Err(FunctionError::ExecutionModeNotSupported { .. }) => {
                        // Fallback to async
                        function.evaluate_async(args, context).await
                    }
                    Err(e) => Err(e),
                }
            }
            (ExecutionMode::SyncFirst, false) => function.evaluate_async(args, context).await,
        }
    }
    
    /// Synchronous-only evaluation (fails for pure async functions)
    pub fn evaluate_function_sync(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        let function = self.get_function(name)
            .ok_or_else(|| FunctionError::FunctionNotFound {
                name: name.to_string(),
            })?;
        
        // Check if function supports sync execution
        match function.execution_mode() {
            ExecutionMode::Async => {
                return Err(FunctionError::ExecutionModeNotSupported {
                    function: name.to_string(),
                    requested_mode: "sync".to_string(),
                });
            }
            _ => {}
        }
        
        function.evaluate_sync(args, context)
    }
    
    /// Get function by name
    pub fn get_function(&self, name: &str) -> Option<Arc<dyn UnifiedFhirPathFunction>> {
        self.functions.read().ok()?.get(name).cloned()
    }
    
    /// Check if function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.read()
            .map(|functions| functions.contains_key(name))
            .unwrap_or(false)
    }
    
    /// Get enhanced metadata for a function
    pub fn get_metadata(&self, name: &str) -> Option<EnhancedFunctionMetadata> {
        self.metadata.read().ok()?.get(name).cloned()
    }
    
    /// Get functions applicable to a specific type context with caching
    pub fn get_functions_for_type_cached(
        &self,
        context_type: Option<&str>,
        is_collection: bool,
    ) -> Vec<String> {
        let cache_key = format!("{}:{}", 
            context_type.unwrap_or("None"), 
            is_collection
        );
        
        // Check cache first
        if let Ok(cache) = self.completion_cache.read() {
            if let Some(cached_functions) = cache.get(&cache_key) {
                self.increment_cache_hits();
                return cached_functions.iter().map(|(name, _)| name.clone()).collect();
            }
        }
        
        self.increment_cache_misses();
        
        let mut applicable_functions = Vec::new();
        
        if let Ok(metadata_map) = self.metadata.read() {
            for (name, metadata) in metadata_map.iter() {
                if self.is_function_applicable_to_type_metadata(
                    name,
                    context_type,
                    is_collection,
                    metadata,
                ) {
                    applicable_functions.push(name.clone());
                }
            }
        }
        
        // Cache the result
        if let Ok(mut cache) = self.completion_cache.write() {
            let cached_data: Vec<(String, EnhancedFunctionMetadata)> = applicable_functions
                .iter()
                .filter_map(|name| {
                    self.get_metadata(name).map(|metadata| (name.clone(), metadata))
                })
                .collect();
            cache.insert(cache_key, cached_data);
        }
        
        applicable_functions
    }
    
    /// Check function applicability using enhanced metadata
    fn is_function_applicable_to_type_metadata(
        &self,
        _function_name: &str,
        context_type: Option<&str>,
        is_collection: bool,
        metadata: &EnhancedFunctionMetadata,
    ) -> bool {
        let constraints = &metadata.type_constraints;
        
        // Check collection requirements
        if constraints.requires_collection && !is_collection {
            return false;
        }
        
        // Check if function supports collections when in collection context
        if is_collection && !constraints.supports_collections {
            return false;
        }
        
        // Check input type constraints
        if let Some(context_type_name) = context_type {
            if !self.type_matches_patterns(context_type_name, &constraints.input_types) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a type matches any of the type patterns
    fn type_matches_patterns(
        &self,
        type_name: &str,
        patterns: &[TypePattern],
    ) -> bool {
        if patterns.is_empty() {
            return true; // No constraints means any type is allowed
        }
        
        for pattern in patterns {
            if self.type_matches_single_pattern(type_name, pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a type matches a single type pattern
    fn type_matches_single_pattern(
        &self,
        type_name: &str,
        pattern: &TypePattern,
    ) -> bool {
        match pattern {
            TypePattern::Any => true,
            TypePattern::Exact(type_info) => type_info.to_string() == type_name,
            TypePattern::OneOf(types) => {
                types.iter().any(|t| t.to_string() == type_name)
            }
            TypePattern::CollectionOf(inner_pattern) => {
                // For collection patterns, we need to extract the element type
                if type_name.starts_with("Collection<") && type_name.ends_with('>') {
                    let element_type = &type_name[11..type_name.len()-1];
                    self.type_matches_single_pattern(element_type, inner_pattern)
                } else {
                    false
                }
            }
            TypePattern::Numeric => {
                matches!(type_name, "Integer" | "Decimal")
            }
            TypePattern::StringLike => {
                matches!(type_name, "String" | "Code" | "Id" | "Uri" | "Url" | "Canonical")
            }
            TypePattern::DateTime => {
                matches!(type_name, "DateTime" | "Date" | "Time" | "Instant")
            }
            TypePattern::Boolean => type_name == "Boolean",
            TypePattern::Resource => type_name.starts_with("Resource") || type_name == "Resource",
            TypePattern::Quantity => type_name == "Quantity",
        }
    }
    
    /// Get functions by category
    pub fn get_functions_by_category(&self) -> FxHashMap<FunctionCategory, Vec<String>> {
        // Check cache first
        if let Ok(cache) = self.category_cache.read() {
            if !cache.is_empty() {
                self.increment_cache_hits();
                return cache.clone();
            }
        }
        
        self.increment_cache_misses();
        
        let mut categorized: FxHashMap<FunctionCategory, Vec<String>> = FxHashMap::default();
        
        if let Ok(metadata_map) = self.metadata.read() {
            for (name, metadata) in metadata_map.iter() {
                match categorized.entry(metadata.basic.category.clone()) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(name.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![name.clone()]);
                    }
                }
            }
        }
        
        // Cache the result
        if let Ok(mut cache) = self.category_cache.write() {
            *cache = categorized.clone();
        }
        
        categorized
    }
    
    /// Get functions suitable for LSP completion
    pub fn get_completion_functions(
        &self,
        context_type: Option<&str>,
        is_collection: bool,
    ) -> Vec<(String, EnhancedFunctionMetadata)> {
        let mut completion_functions = Vec::new();
        
        if let Ok(metadata_map) = self.metadata.read() {
            for (name, metadata) in metadata_map.iter() {
                match metadata.basic.lsp_info.completion_visibility {
                    CompletionVisibility::Never => continue,
                    CompletionVisibility::Always => {
                        completion_functions.push((name.clone(), metadata.clone()));
                    }
                    CompletionVisibility::Contextual => {
                        if self.is_function_applicable_to_type_metadata(
                            name,
                            context_type,
                            is_collection,
                            metadata,
                        ) {
                            completion_functions.push((name.clone(), metadata.clone()));
                        }
                    }
                }
            }
        }
        
        // Sort by LSP priority
        completion_functions.sort_by(|a, b| {
            a.1.basic.lsp_info.sort_priority.cmp(&b.1.basic.lsp_info.sort_priority)
        });
        
        completion_functions
    }
    
    /// Get registry statistics
    pub fn get_stats(&self) -> RegistryStats {
        let compilation_stats = self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| RegistryCompilationStats::new());
        
        RegistryStats {
            total_functions: compilation_stats.total_functions,
            sync_functions: compilation_stats.sync_functions,
            async_functions: compilation_stats.async_functions,
            sync_first_functions: compilation_stats.sync_first_functions,
            metadata_entries: compilation_stats.metadata_entries,
            cache_hit_ratio: if compilation_stats.cache_misses + compilation_stats.cache_hits > 0 {
                compilation_stats.cache_hits as f64 / 
                (compilation_stats.cache_hits + compilation_stats.cache_misses) as f64
            } else {
                0.0
            },
            compilation_time: compilation_stats.last_compilation_time,
        }
    }
    
    /// Clear all caches
    pub fn clear_caches(&self) {
        let _ = self.type_applicability_cache.write().map(|mut cache| cache.clear());
        let _ = self.category_cache.write().map(|mut cache| cache.clear());
        let _ = self.completion_cache.write().map(|mut cache| cache.clear());
    }
    
    /// Clear caches related to a specific function
    fn clear_function_caches(&self, _function_name: &str) {
        // For now, clear all caches when a function is added/removed
        // In the future, we could be more selective
        self.clear_caches();
    }
    
    /// Update compilation statistics
    fn update_compilation_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_functions = self.functions.read()
                .map(|f| f.len())
                .unwrap_or(0);
            stats.sync_functions = self.sync_functions.read()
                .map(|f| f.len())
                .unwrap_or(0);
            stats.async_functions = self.async_functions.read()
                .map(|f| f.len())
                .unwrap_or(0);
            stats.sync_first_functions = self.sync_first_functions.read()
                .map(|f| f.len())
                .unwrap_or(0);
            stats.metadata_entries = self.metadata.read()
                .map(|m| m.len())
                .unwrap_or(0);
            stats.last_compilation_time = Some(Instant::now());
        }
    }
    
    /// Increment cache hits counter
    fn increment_cache_hits(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.cache_hits += 1;
        }
    }
    
    /// Increment cache misses counter
    fn increment_cache_misses(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.cache_misses += 1;
        }
    }
    
    /// Get all function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.read()
            .map(|functions| functions.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get function count
    pub fn function_count(&self) -> usize {
        self.functions.read()
            .map(|functions| functions.len())
            .unwrap_or(0)
    }
    
    /// Check if a function supports lambda expressions
    pub fn is_lambda_function(&self, name: &str) -> bool {
        if let Some(function) = self.get_function(name) {
            return function.metadata().lambda.supports_lambda_evaluation;
        }
        false
    }
    
    /// Get lambda argument indices for a function
    pub fn get_lambda_argument_indices(&self, name: &str) -> Vec<usize> {
        if let Some(function) = self.get_function(name) {
            return function.metadata().lambda.lambda_argument_indices.clone();
        }
        Vec::new()
    }
    
    /// Check if function requires expression arguments (not pre-evaluated)
    pub fn requires_expression_arguments(&self, name: &str) -> bool {
        if let Some(function) = self.get_function(name) {
            return function.metadata().lambda.requires_lambda_evaluation;
        }
        false
    }
    
    /// Get functions that support lambda expressions
    pub fn get_lambda_functions(&self) -> Vec<String> {
        let mut lambda_functions = Vec::new();
        
        if let Ok(functions) = self.functions.read() {
            for (name, function) in functions.iter() {
                if function.metadata().lambda.supports_lambda_evaluation {
                    lambda_functions.push(name.clone());
                }
            }
        }
        
        lambda_functions
    }
    
    /// Get enhanced lambda function statistics
    pub fn get_lambda_stats(&self) -> LambdaFunctionStats {
        let mut stats = LambdaFunctionStats::default();
        
        if let Ok(functions) = self.functions.read() {
            for function in functions.values() {
                let lambda_metadata = &function.metadata().lambda;
                if lambda_metadata.supports_lambda_evaluation {
                    stats.total_lambda_functions += 1;
                    
                    if lambda_metadata.requires_lambda_evaluation {
                        stats.requires_lambda += 1;
                    }
                    
                    if !lambda_metadata.lambda_argument_indices.is_empty() {
                        stats.has_lambda_indices += 1;
                    }
                }
            }
        }
        
        stats
    }
}

impl Default for UnifiedFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics summary
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of functions registered
    pub total_functions: usize,
    
    /// Number of sync functions
    pub sync_functions: usize,
    
    /// Number of async functions
    pub async_functions: usize,
    
    /// Number of sync-first functions
    pub sync_first_functions: usize,
    
    /// Number of metadata entries
    pub metadata_entries: usize,
    
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    
    /// Last compilation time
    pub compilation_time: Option<Instant>,
}

/// Lambda function statistics
#[derive(Debug, Clone, Default)]
pub struct LambdaFunctionStats {
    /// Total number of lambda functions
    pub total_lambda_functions: usize,
    
    /// Functions that require lambda evaluation
    pub requires_lambda: usize,
    
    /// Functions with lambda argument indices defined
    pub has_lambda_indices: usize,
}

impl std::fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registry Stats: {} functions ({} sync, {} async, {} sync-first) - {:.1}% cache hit ratio",
            self.total_functions,
            self.sync_functions,
            self.async_functions,
            self.sync_first_functions,
            self.cache_hit_ratio * 100.0
        )
    }
}

impl std::fmt::Display for LambdaFunctionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lambda Stats: {} lambda functions ({} require lambda, {} have indices)",
            self.total_lambda_functions,
            self.requires_lambda,
            self.has_lambda_indices
        )
    }
}

/// Registry operation errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// Invalid function name
    #[error("Invalid function name '{name}': {reason}")]
    InvalidFunctionName {
        name: String,
        reason: String,
    },
    
    /// Duplicate function registration
    #[error("Function '{name}' is already registered")]
    DuplicateFunction {
        name: String,
    },
    
    /// Registry locked (concurrent access issue)
    #[error("Registry is temporarily locked due to concurrent access")]
    RegistryLocked,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_builder::MetadataBuilder;
    use crate::function::FunctionCategory;
    
    // Test function implementation
    struct TestFunction {
        name: String,
        metadata: EnhancedFunctionMetadata,
        execution_mode: ExecutionMode,
    }
    
    impl TestFunction {
        fn new(name: &str, execution_mode: ExecutionMode) -> Self {
            let metadata = MetadataBuilder::new(name, FunctionCategory::Utilities)
                .description(&format!("Test function {}", name))
                .execution_mode(execution_mode)
                .pure(true)
                .build();
            
            Self {
                name: name.to_string(),
                metadata,
                execution_mode,
            }
        }
    }
    
    #[async_trait::async_trait]
    impl UnifiedFhirPathFunction for TestFunction {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn metadata(&self) -> &EnhancedFunctionMetadata {
            &self.metadata
        }
        
        fn execution_mode(&self) -> ExecutionMode {
            self.execution_mode
        }
        
        fn evaluate_sync(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            match self.execution_mode {
                ExecutionMode::Async => Err(FunctionError::ExecutionModeNotSupported {
                    function: self.name.clone(),
                    requested_mode: "sync".to_string(),
                }),
                _ => Ok(FhirPathValue::String(format!("sync_{}", self.name).into())),
            }
        }
        
        async fn evaluate_async(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            Ok(FhirPathValue::String(format!("async_{}", self.name).into()))
        }
    }
    
    #[tokio::test]
    async fn test_registry_basic_operations() {
        let registry = UnifiedFunctionRegistry::new();
        
        // Register test functions
        let sync_func = TestFunction::new("syncTest", ExecutionMode::Sync);
        let async_func = TestFunction::new("asyncTest", ExecutionMode::Async);
        let sync_first_func = TestFunction::new("syncFirstTest", ExecutionMode::SyncFirst);
        
        assert!(registry.register(sync_func).is_ok());
        assert!(registry.register(async_func).is_ok());
        assert!(registry.register(sync_first_func).is_ok());
        
        // Check function existence
        assert!(registry.contains("syncTest"));
        assert!(registry.contains("asyncTest"));
        assert!(registry.contains("syncFirstTest"));
        assert!(!registry.contains("nonExistent"));
        
        // Check stats
        let stats = registry.get_stats();
        assert_eq!(stats.total_functions, 3);
        assert_eq!(stats.sync_functions, 1);
        assert_eq!(stats.async_functions, 1);
        assert_eq!(stats.sync_first_functions, 1);
    }
    
    #[tokio::test]
    async fn test_function_evaluation() {
        let registry = UnifiedFunctionRegistry::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        // Register test functions
        registry.register(TestFunction::new("syncTest", ExecutionMode::Sync)).unwrap();
        registry.register(TestFunction::new("asyncTest", ExecutionMode::Async)).unwrap();
        
        // Test sync function evaluation
        let result = registry.evaluate_function_sync("syncTest", &[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("sync_syncTest".into()));
        
        // Test async function evaluation
        let result = registry.evaluate_function("asyncTest", &[], &context, false).await.unwrap();
        assert_eq!(result, FhirPathValue::String("async_asyncTest".into()));
        
        // Test sync function called async
        let result = registry.evaluate_function("syncTest", &[], &context, true).await.unwrap();
        assert_eq!(result, FhirPathValue::String("sync_syncTest".into()));
    }
    
    #[test]
    fn test_duplicate_function_registration() {
        let registry = UnifiedFunctionRegistry::new();
        
        let func1 = TestFunction::new("duplicate", ExecutionMode::Sync);
        let func2 = TestFunction::new("duplicate", ExecutionMode::Async);
        
        assert!(registry.register(func1).is_ok());
        assert!(matches!(
            registry.register(func2),
            Err(RegistryError::DuplicateFunction { .. })
        ));
    }
    
    #[test]
    fn test_metadata_retrieval() {
        let registry = UnifiedFunctionRegistry::new();
        
        let func = TestFunction::new("metadataTest", ExecutionMode::Sync);
        registry.register(func).unwrap();
        
        let metadata = registry.get_metadata("metadataTest").unwrap();
        assert_eq!(metadata.basic.name, "metadataTest");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        
        assert!(registry.get_metadata("nonExistent").is_none());
    }
    
    #[test]
    fn test_category_grouping() {
        let registry = UnifiedFunctionRegistry::new();
        
        // Register functions in different categories
        let util_func = TestFunction::new("utilFunc", ExecutionMode::Sync);
        let math_func = TestFunction {
            name: "mathFunc".to_string(),
            metadata: MetadataBuilder::math_function("mathFunc").build(),
            execution_mode: ExecutionMode::Sync,
        };
        
        registry.register(util_func).unwrap();
        registry.register(math_func).unwrap();
        
        let categories = registry.get_functions_by_category();
        
        assert!(categories.contains_key(&FunctionCategory::Utilities));
        assert!(categories.contains_key(&FunctionCategory::MathNumbers));
        assert!(categories[&FunctionCategory::Utilities].contains(&"utilFunc".to_string()));
        assert!(categories[&FunctionCategory::MathNumbers].contains(&"mathFunc".to_string()));
    }
    
    #[test]
    fn test_type_applicability_caching() {
        let registry = UnifiedFunctionRegistry::new();
        
        // Register a string function
        let string_func = TestFunction {
            name: "stringTest".to_string(),
            metadata: MetadataBuilder::string_function("stringTest").build(),
            execution_mode: ExecutionMode::Sync,
        };
        registry.register(string_func).unwrap();
        
        // First call should miss cache
        let functions1 = registry.get_functions_for_type_cached(Some("String"), false);
        assert!(functions1.contains(&"stringTest".to_string()));
        
        // Second call should hit cache
        let functions2 = registry.get_functions_for_type_cached(Some("String"), false);
        assert_eq!(functions1, functions2);
        
        // Check cache statistics
        let stats = registry.get_stats();
        assert!(stats.cache_hit_ratio > 0.0);
    }
    
    #[tokio::test]
    async fn test_performance_characteristics() {
        let registry = UnifiedFunctionRegistry::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        // Register a sync function
        let sync_func = TestFunction::new("perfTest", ExecutionMode::Sync);
        registry.register(sync_func).unwrap();
        
        // Measure sync dispatch performance (rough timing)
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = registry.evaluate_function_sync("perfTest", &[], &context).unwrap();
        }
        let sync_time = start.elapsed();
        
        // Measure async dispatch performance
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = registry.evaluate_function("perfTest", &[], &context, true).await.unwrap();
        }
        let async_time = start.elapsed();
        
        // Both should work correctly
        assert!(sync_time.as_nanos() > 0);
        assert!(async_time.as_nanos() > 0);
        
        // Verify cache performance
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = registry.get_functions_for_type_cached(Some("String"), false);
        }
        let cache_time = start.elapsed();
        
        // Multiple cache hits should be very fast
        assert!(cache_time.as_millis() < 100); // Should be much faster than 100ms
        
        let stats = registry.get_stats();
        assert!(stats.cache_hit_ratio > 0.8); // Most calls should hit cache
    }
}