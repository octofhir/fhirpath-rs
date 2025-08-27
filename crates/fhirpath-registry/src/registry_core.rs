//! Core Registry Implementation
//!
//! This module contains the shared implementation logic used by both
//! SyncRegistry and AsyncRegistry to eliminate code duplication.
//!
//! # Design Philosophy
//!
//! - **Shared storage patterns**: Common HashMap management and thread-safety
//! - **Generic operation traits**: Support for both sync and async operations
//! - **Fast lookup**: O(1) HashMap operations with optimal access patterns
//! - **Thread-safe**: RwLock-based concurrent access for high performance
//! - **Type-safe**: Generic bounds ensure operation type safety

use crate::signature::FunctionSignature;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::RwLock;

/// Core trait for registry operations
///
/// This trait provides the common interface that both sync and async
/// operations implement, allowing for shared storage and management logic.
pub trait RegistryOperation: Send + Sync + Debug {
    /// Get the operation name for registry lookup
    fn name(&self) -> &'static str;

    /// Get the operation signature for validation/documentation
    fn signature(&self) -> &FunctionSignature;

    /// Validate arguments against the operation signature
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        let signature = self.signature();

        // Check argument count for non-variadic functions
        if !signature.variadic && args.len() != signature.parameters.len() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: signature.name.to_string(),
                expected: signature.parameters.len(),
                actual: args.len(),
            });
        }

        // For variadic functions, ensure we have at least the minimum required args
        if signature.variadic && args.len() < signature.parameters.len() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: signature.name.to_string(),
                expected: signature.parameters.len(),
                actual: args.len(),
            });
        }

        // Type validation would go here if needed
        // For now, we trust the operation implementations to handle type validation
        Ok(())
    }
}

/// Core registry storage and management
///
/// This struct provides the shared storage and management logic for both
/// sync and async registries, eliminating code duplication.
pub struct RegistryCore<T: RegistryOperation + ?Sized> {
    operations: RwLock<HashMap<String, Box<T>>>,
}

impl<T: RegistryOperation + ?Sized> RegistryCore<T> {
    /// Create a new empty registry core
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
        }
    }

    /// Register a single operation
    pub async fn register(&self, operation: Box<T>) {
        let name = operation.name().to_string();
        let mut ops = self.operations.write().await;
        ops.insert(name, operation);
    }

    /// Register multiple operations at once
    ///
    /// This is more efficient than calling register() multiple times
    /// as it only acquires the write lock once.
    pub async fn register_many(&self, operations: Vec<Box<T>>) {
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

    /// Get all registered operation names
    ///
    /// Returns a sorted list of operation names for consistent output.
    pub async fn get_operation_names(&self) -> Vec<String> {
        let ops = self.operations.read().await;
        let mut names: Vec<String> = ops.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get the signature of an operation
    pub async fn get_signature(&self, name: &str) -> Option<FunctionSignature> {
        let ops = self.operations.read().await;
        ops.get(name).map(|op| op.signature().clone())
    }

    /// Get the total number of registered operations
    pub async fn operation_count(&self) -> usize {
        let ops = self.operations.read().await;
        ops.len()
    }

    /// Check if the registry is empty
    pub async fn is_empty(&self) -> bool {
        let ops = self.operations.read().await;
        ops.is_empty()
    }

    /// Get statistics about the registry
    pub async fn get_stats(&self) -> RegistryStats {
        let ops = self.operations.read().await;
        let count = ops.len();

        RegistryStats {
            operation_count: count,
            memory_usage_estimate: count * std::mem::size_of::<String>()
                + count * std::mem::size_of::<Box<T>>(),
        }
    }

    /// Execute operation lookup and validation
    ///
    /// This method handles the common lookup and validation logic,
    /// returning the operation for execution by the specific registry type.
    pub async fn lookup_and_validate(
        &self,
        name: &str,
        args: &[FhirPathValue],
    ) -> Result<OperationLookupResult> {
        let ops = self.operations.read().await;

        if let Some(operation) = ops.get(name) {
            // Validate arguments before returning
            operation.validate_args(args)?;

            Ok(OperationLookupResult::Found)
        } else {
            Ok(OperationLookupResult::NotFound)
        }
    }

    /// Get an operation reference for execution
    ///
    /// This method provides access to the operation while holding the read lock.
    /// The caller should execute the operation and release the lock quickly.
    pub async fn with_operation<F, R>(&self, name: &str, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        let ops = self.operations.read().await;
        ops.get(name).map(|op| f(op.as_ref()))
    }

    /// Get direct access to the operations map for complex operations
    ///
    /// This method provides access to the underlying operations HashMap
    /// for cases where async execution requires direct access.
    pub fn operations(&self) -> &RwLock<HashMap<String, Box<T>>> {
        &self.operations
    }

    /// Clear all operations from the registry
    #[cfg(test)]
    pub async fn clear(&self) {
        let mut ops = self.operations.write().await;
        ops.clear();
    }
}

impl<T: RegistryOperation + ?Sized> Default for RegistryCore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RegistryOperation + ?Sized> std::fmt::Debug for RegistryCore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We can't easily get the operations count without async, so just show type
        f.debug_struct("RegistryCore")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

/// Result of operation lookup and validation
pub enum OperationLookupResult {
    /// Operation was found and validated
    Found,
    /// Operation was not found
    NotFound,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Number of registered operations
    pub operation_count: usize,
    /// Estimated memory usage in bytes
    pub memory_usage_estimate: usize,
}

impl std::fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registry: {} operations, ~{} bytes",
            self.operation_count, self.memory_usage_estimate
        )
    }
}

/// Batch operation registration helper
///
/// This struct helps with efficient batch registration of operations.
pub struct BatchRegistrar<T: RegistryOperation> {
    operations: Vec<Box<T>>,
}

impl<T: RegistryOperation> BatchRegistrar<T> {
    /// Create a new batch registrar
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add an operation to the batch
    pub fn add(mut self, operation: Box<T>) -> Self {
        self.operations.push(operation);
        self
    }

    /// Register all operations in the batch
    pub async fn register_all(self, core: &RegistryCore<T>) {
        core.register_many(self.operations).await;
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl<T: RegistryOperation> Default for BatchRegistrar<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{FunctionSignature, ValueType};
    

    // Test operation for registry core testing
    #[derive(Debug)]
    struct TestOperation {
        name: &'static str,
        signature: FunctionSignature,
    }

    impl RegistryOperation for TestOperation {
        fn name(&self) -> &'static str {
            self.name
        }

        fn signature(&self) -> &FunctionSignature {
            &self.signature
        }
    }

    fn create_test_operation(name: &'static str) -> Box<TestOperation> {
        Box::new(TestOperation {
            name,
            signature: FunctionSignature {
                name,
                parameters: vec![],
                return_type: ValueType::String,
                variadic: false,
                category: crate::signature::FunctionCategory::Universal,
                cardinality_requirement: crate::signature::CardinalityRequirement::AcceptsBoth,
            },
        })
    }

    #[tokio::test]
    async fn test_registry_core_basic_operations() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        // Initially empty
        assert!(core.is_empty().await);
        assert_eq!(core.operation_count().await, 0);

        // Register operation
        let op = create_test_operation("test");
        core.register(op).await;

        // Check registration
        assert!(!core.is_empty().await);
        assert_eq!(core.operation_count().await, 1);
        assert!(core.contains("test").await);
        assert!(!core.contains("nonexistent").await);

        // Check operation names
        let names = core.get_operation_names().await;
        assert_eq!(names, vec!["test"]);
    }

    #[tokio::test]
    async fn test_registry_core_batch_registration() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        // Create batch
        let operations = vec![
            create_test_operation("op1"),
            create_test_operation("op2"),
            create_test_operation("op3"),
        ];

        // Register batch
        core.register_many(operations).await;

        // Check all registered
        assert_eq!(core.operation_count().await, 3);
        assert!(core.contains("op1").await);
        assert!(core.contains("op2").await);
        assert!(core.contains("op3").await);

        // Check sorted names
        let names = core.get_operation_names().await;
        assert_eq!(names, vec!["op1", "op2", "op3"]);
    }

    #[tokio::test]
    async fn test_batch_registrar() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        let batch = BatchRegistrar::new()
            .add(create_test_operation("batch1"))
            .add(create_test_operation("batch2"));

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());

        batch.register_all(&core).await;

        assert_eq!(core.operation_count().await, 2);
        assert!(core.contains("batch1").await);
        assert!(core.contains("batch2").await);
    }

    #[tokio::test]
    async fn test_operation_lookup_and_validate() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        let op = create_test_operation("test");
        core.register(op).await;

        // Test successful lookup
        let result = core.lookup_and_validate("test", &[]).await.unwrap();
        matches!(result, OperationLookupResult::Found);

        // Test not found
        let result = core.lookup_and_validate("nonexistent", &[]).await.unwrap();
        matches!(result, OperationLookupResult::NotFound);
    }

    #[tokio::test]
    async fn test_with_operation() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        let op = create_test_operation("test");
        core.register(op).await;

        // Test with_operation
        let name = core.with_operation("test", |op| op.name()).await;
        assert_eq!(name, Some("test"));

        let name = core.with_operation("nonexistent", |op| op.name()).await;
        assert_eq!(name, None);
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let core: RegistryCore<TestOperation> = RegistryCore::new();

        let stats = core.get_stats().await;
        assert_eq!(stats.operation_count, 0);

        core.register(create_test_operation("test")).await;

        let stats = core.get_stats().await;
        assert_eq!(stats.operation_count, 1);
        assert!(stats.memory_usage_estimate > 0);
    }
}
