//! Simplified Registry System
//!
//! This module provides a clean, simple registry system for FHIRPath operations
//! that replaces the over-engineered previous system. It uses HashMap-based
//! registries for fast lookups without unnecessary complexity.
//!
//! # Design Philosophy
//! 
//! - **Simple HashMap storage**: No complex caching or optimization
//! - **Separate sync/async registries**: Clear separation of operation types
//! - **Fast dispatch**: Sync-first lookup for performance
//! - **Easy registration**: Simple function calls, no builders
//! - **Thread-safe**: Uses Arc and RwLock for concurrent access

use crate::traits::{AsyncOperation, EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for synchronous operations
///
/// Stores sync operations in a simple HashMap for O(1) lookup.
/// Thread-safe using RwLock for concurrent read access.
pub struct SyncRegistry {
    operations: RwLock<HashMap<String, Box<dyn SyncOperation>>>,
}

impl SyncRegistry {
    /// Create a new empty sync registry
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
        }
    }

    /// Register a sync operation
    pub async fn register(&self, operation: Box<dyn SyncOperation>) {
        let name = operation.name().to_string();
        let mut ops = self.operations.write().await;
        ops.insert(name, operation);
    }

    /// Register multiple sync operations at once
    pub async fn register_many(&self, operations: Vec<Box<dyn SyncOperation>>) {
        let mut ops = self.operations.write().await;
        for operation in operations {
            let name = operation.name().to_string();
            ops.insert(name, operation);
        }
    }

    /// Check if an operation is registered
    pub async fn contains(&self, name: &str) -> bool {
        let ops = self.operations.read().await;
        ops.contains_key(name)
    }

    /// Get operation names (for debugging/introspection)
    pub async fn get_operation_names(&self) -> Vec<String> {
        let ops = self.operations.read().await;
        ops.keys().cloned().collect()
    }

    /// Execute a sync operation
    pub async fn execute(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let ops = self.operations.read().await;
        
        if let Some(operation) = ops.get(name) {
            // Validate arguments before execution
            operation.validate_args(args)?;
            
            // Execute synchronously (no await needed)
            operation.execute(args, context)
        } else {
            Err(FhirPathError::UnknownFunction { 
                function_name: name.to_string() 
            })
        }
    }

    /// Get the signature of an operation (for validation/documentation)
    pub async fn get_signature(&self, name: &str) -> Option<crate::signature::FunctionSignature> {
        let ops = self.operations.read().await;
        ops.get(name).map(|op| op.signature().clone())
    }
}

impl Default for SyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for asynchronous operations
///
/// Stores async operations in a simple HashMap for O(1) lookup.
/// Thread-safe using RwLock for concurrent read access.
pub struct AsyncRegistry {
    operations: RwLock<HashMap<String, Box<dyn AsyncOperation>>>,
}

impl AsyncRegistry {
    /// Create a new empty async registry
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
        }
    }

    /// Register an async operation
    pub async fn register(&self, operation: Box<dyn AsyncOperation>) {
        let name = operation.name().to_string();
        let mut ops = self.operations.write().await;
        ops.insert(name, operation);
    }

    /// Register multiple async operations at once
    pub async fn register_many(&self, operations: Vec<Box<dyn AsyncOperation>>) {
        let mut ops = self.operations.write().await;
        for operation in operations {
            let name = operation.name().to_string();
            ops.insert(name, operation);
        }
    }

    /// Check if an operation is registered
    pub async fn contains(&self, name: &str) -> bool {
        let ops = self.operations.read().await;
        ops.contains_key(name)
    }

    /// Get operation names (for debugging/introspection)
    pub async fn get_operation_names(&self) -> Vec<String> {
        let ops = self.operations.read().await;
        ops.keys().cloned().collect()
    }

    /// Execute an async operation
    pub async fn execute(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let ops = self.operations.read().await;
        
        if let Some(operation) = ops.get(name) {
            // Validate arguments before execution
            operation.validate_args(args)?;
            
            // Execute asynchronously
            operation.execute(args, context).await
        } else {
            Err(FhirPathError::UnknownFunction { 
                function_name: name.to_string() 
            })
        }
    }

    /// Get the signature of an operation (for validation/documentation)
    pub async fn get_signature(&self, name: &str) -> Option<crate::signature::FunctionSignature> {
        let ops = self.operations.read().await;
        ops.get(name).map(|op| op.signature().clone())
    }
}

impl Default for AsyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified registry that combines sync and async operations
///
/// This registry provides a single interface for both sync and async operations,
/// automatically dispatching to the appropriate registry based on operation type.
/// It tries sync operations first for better performance.
pub struct FunctionRegistry {
    sync_registry: Arc<SyncRegistry>,
    async_registry: Arc<AsyncRegistry>,
}

impl FunctionRegistry {
    /// Create a new unified registry
    pub fn new() -> Self {
        Self {
            sync_registry: Arc::new(SyncRegistry::new()),
            async_registry: Arc::new(AsyncRegistry::new()),
        }
    }

    /// Create a unified registry from existing sync and async registries
    pub fn from_registries(sync_registry: Arc<SyncRegistry>, async_registry: Arc<AsyncRegistry>) -> Self {
        Self {
            sync_registry,
            async_registry,
        }
    }

    /// Register a sync operation
    pub async fn register_sync(&self, operation: Box<dyn SyncOperation>) {
        self.sync_registry.register(operation).await;
    }

    /// Register an async operation
    pub async fn register_async(&self, operation: Box<dyn AsyncOperation>) {
        self.async_registry.register(operation).await;
    }

    /// Register multiple sync operations at once
    pub async fn register_sync_many(&self, operations: Vec<Box<dyn SyncOperation>>) {
        self.sync_registry.register_many(operations).await;
    }

    /// Register multiple async operations at once
    pub async fn register_async_many(&self, operations: Vec<Box<dyn AsyncOperation>>) {
        self.async_registry.register_many(operations).await;
    }

    /// Execute an operation (tries sync first, then async)
    ///
    /// This method provides sync-first dispatch for optimal performance:
    /// 1. Try sync registry first (faster execution)
    /// 2. Fall back to async registry if not found
    /// 3. Return error if operation not found in either registry
    pub async fn execute(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync first for performance
        if self.sync_registry.contains(name).await {
            return self.sync_registry.execute(name, args, context).await;
        }

        // Fall back to async
        if self.async_registry.contains(name).await {
            return self.async_registry.execute(name, args, context).await;
        }

        // Operation not found in either registry
        Err(FhirPathError::UnknownFunction { 
            function_name: name.to_string() 
        })
    }

    /// Check if an operation is registered (in either registry)
    pub async fn contains(&self, name: &str) -> bool {
        self.sync_registry.contains(name).await || self.async_registry.contains(name).await
    }

    /// Check if an operation is sync
    pub async fn is_sync(&self, name: &str) -> bool {
        self.sync_registry.contains(name).await
    }

    /// Check if an operation is async
    pub async fn is_async(&self, name: &str) -> bool {
        self.async_registry.contains(name).await
    }

    /// Get all operation names from both registries
    pub async fn get_all_operation_names(&self) -> Vec<String> {
        let mut names = self.sync_registry.get_operation_names().await;
        let async_names = self.async_registry.get_operation_names().await;
        names.extend(async_names);
        names.sort();
        names
    }

    /// Get sync operation names only
    pub async fn get_sync_operation_names(&self) -> Vec<String> {
        self.sync_registry.get_operation_names().await
    }

    /// Get async operation names only
    pub async fn get_async_operation_names(&self) -> Vec<String> {
        self.async_registry.get_operation_names().await
    }

    /// Get operation signature (from either registry)
    pub async fn get_signature(&self, name: &str) -> Option<crate::signature::FunctionSignature> {
        // Try sync first
        if let Some(signature) = self.sync_registry.get_signature(name).await {
            return Some(signature);
        }

        // Try async
        self.async_registry.get_signature(name).await
    }

    /// Get statistics about the registry
    pub async fn get_stats(&self) -> RegistryStats {
        let sync_count = self.sync_registry.get_operation_names().await.len();
        let async_count = self.async_registry.get_operation_names().await.len();

        RegistryStats {
            sync_operations: sync_count,
            async_operations: async_count,
            total_operations: sync_count + async_count,
            sync_percentage: if sync_count + async_count > 0 {
                (sync_count as f64 / (sync_count + async_count) as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub sync_operations: usize,
    pub async_operations: usize,
    pub total_operations: usize,
    pub sync_percentage: f64,
}

impl std::fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registry Stats: {} sync, {} async, {} total ({:.1}% sync)",
            self.sync_operations, self.async_operations, self.total_operations, self.sync_percentage
        )
    }
}

/// Builder for creating pre-configured registries
pub struct RegistryBuilder {
    sync_operations: Vec<Box<dyn SyncOperation>>,
    async_operations: Vec<Box<dyn AsyncOperation>>,
}

impl RegistryBuilder {
    /// Create a new registry builder
    pub fn new() -> Self {
        Self {
            sync_operations: Vec::new(),
            async_operations: Vec::new(),
        }
    }

    /// Add a sync operation to the builder
    pub fn with_sync(mut self, operation: Box<dyn SyncOperation>) -> Self {
        self.sync_operations.push(operation);
        self
    }

    /// Add an async operation to the builder
    pub fn with_async(mut self, operation: Box<dyn AsyncOperation>) -> Self {
        self.async_operations.push(operation);
        self
    }

    /// Build the unified registry
    pub async fn build(self) -> FunctionRegistry {
        let registry = FunctionRegistry::new();
        
        registry.register_sync_many(self.sync_operations).await;
        registry.register_async_many(self.async_operations).await;
        
        registry
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{FunctionSignature, ValueType};
    use crate::traits::{AsyncOperation, SyncOperation};
    use async_trait::async_trait;

    // Test sync operation
    struct TestSyncOp;
    impl SyncOperation for TestSyncOp {
        fn name(&self) -> &'static str { "testSync" }
        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: FunctionSignature = FunctionSignature {
                name: "testSync",
                parameters: vec![],
                return_type: ValueType::String,
                variadic: false,
            };
            &SIGNATURE
        }
        fn execute(&self, _args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
            Ok(FhirPathValue::String("sync result".into()))
        }
    }

    // Test async operation
    struct TestAsyncOp;
    #[async_trait]
    impl AsyncOperation for TestAsyncOp {
        fn name(&self) -> &'static str { "testAsync" }
        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: FunctionSignature = FunctionSignature {
                name: "testAsync",
                parameters: vec![],
                return_type: ValueType::String,
                variadic: false,
            };
            &SIGNATURE
        }
        async fn execute(&self, _args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
            Ok(FhirPathValue::String("async result".into()))
        }
    }

    #[tokio::test]
    async fn test_sync_registry() {
        let registry = SyncRegistry::new();
        
        // Register operation
        registry.register(Box::new(TestSyncOp)).await;
        
        // Check registration
        assert!(registry.contains("testSync").await);
        assert!(!registry.contains("nonexistent").await);
        
        // Execute operation
        let context = create_test_context();
        let result = registry.execute("testSync", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("sync result".into()));
        
        // Test unknown operation
        let error = registry.execute("unknown", &[], &context).await;
        assert!(error.is_err());
    }

    #[tokio::test]
    async fn test_async_registry() {
        let registry = AsyncRegistry::new();
        
        // Register operation
        registry.register(Box::new(TestAsyncOp)).await;
        
        // Check registration
        assert!(registry.contains("testAsync").await);
        assert!(!registry.contains("nonexistent").await);
        
        // Execute operation
        let context = create_test_context();
        let result = registry.execute("testAsync", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("async result".into()));
    }

    #[tokio::test]
    async fn test_unified_registry() {
        let registry = FunctionRegistry::new();
        
        // Register both types
        registry.register_sync(Box::new(TestSyncOp)).await;
        registry.register_async(Box::new(TestAsyncOp)).await;
        
        // Test operations
        let context = create_test_context();
        
        let sync_result = registry.execute("testSync", &[], &context).await.unwrap();
        assert_eq!(sync_result, FhirPathValue::String("sync result".into()));
        
        let async_result = registry.execute("testAsync", &[], &context).await.unwrap();
        assert_eq!(async_result, FhirPathValue::String("async result".into()));
        
        // Test type checking
        assert!(registry.is_sync("testSync").await);
        assert!(!registry.is_sync("testAsync").await);
        assert!(!registry.is_async("testSync").await);
        assert!(registry.is_async("testAsync").await);
        
        // Test stats
        let stats = registry.get_stats().await;
        assert_eq!(stats.sync_operations, 1);
        assert_eq!(stats.async_operations, 1);
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.sync_percentage, 50.0);
    }

    #[tokio::test]
    async fn test_registry_builder() {
        let registry = RegistryBuilder::new()
            .with_sync(Box::new(TestSyncOp))
            .with_async(Box::new(TestAsyncOp))
            .build()
            .await;
        
        let context = create_test_context();
        
        // Both operations should be registered
        let sync_result = registry.execute("testSync", &[], &context).await.unwrap();
        assert_eq!(sync_result, FhirPathValue::String("sync result".into()));
        
        let async_result = registry.execute("testAsync", &[], &context).await.unwrap();
        assert_eq!(async_result, FhirPathValue::String("async result".into()));
    }

    fn create_test_context() -> EvaluationContext {
        use octofhir_fhirpath_model::MockModelProvider;
        EvaluationContext::new(
            FhirPathValue::Empty,
            std::sync::Arc::new(MockModelProvider::new()),
        )
    }
}