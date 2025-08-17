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

//! Unified FHIRPath Registry
//!
//! This crate provides a high-performance, async-first unified registry for all
//! FHIRPath functions and operators. The registry combines previously separate
//! function and operator registries into a single, optimized system.

// Core unified system - V2 Architecture
pub use fhirpath_registry::{DispatchKey, FhirPathRegistry};
pub use lambda::{
    ExpressionEvaluator, LambdaContextBuilder, LambdaFunction, LambdaOperationWrapper, LambdaUtils,
};
pub use metadata::{
    Associativity, FhirPathType, FunctionMetadata, MetadataBuilder, OperationMetadata,
    OperationSpecificMetadata, OperationType, OperatorMetadata, PerformanceMetadata,
    TypeConstraint,
};
pub use operation::{
    CollectionOperation, CompilableOperation, CompiledOperation, FhirPathOperation,
    OperationComplexity, OperationSignature, ScalarOperation,
};

// Core system modules
pub mod async_cache;
pub mod fhirpath_registry;
mod lambda;
pub mod metadata;
pub mod operation;

// Operation implementations
pub mod operations;

// Legacy compatibility modules (kept for compatibility - will be removed in future versions)
pub mod registry_config;
pub mod signature;

// Test modules
#[cfg(test)]
mod operation_tests;

// Main unified registry exports
pub use async_cache::{AsyncLruCache, CacheBuilder, CacheMetrics};

// Legacy exports (kept for compatibility - will be removed)
pub use signature::{FunctionSignature, ParameterInfo};

// Re-exports from workspace crates
pub use octofhir_fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};

/// Create a standard registry with all built-in operations
///
/// This creates the new unified FhirPathRegistry with all standard operations.
///
/// # Returns
///
/// A fully configured `FhirPathRegistry` with all standard FHIRPath operations registered.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath_registry::create_standard_registry;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = create_standard_registry().await?;
///     // Use registry for FHIRPath evaluation
///     Ok(())
/// }
/// ```
pub async fn create_standard_registry()
-> std::result::Result<FhirPathRegistry, Box<dyn std::error::Error>> {
    let registry = FhirPathRegistry::new();

    use operations::*;

    CollectionOperations::register_all(&registry).await?;
    StringOperations::register_all(&registry).await?;
    DateTimeOperations::register_all(&registry).await?;
    FhirOperations::register_all(&registry).await?;
    UtilityOperations::register_all(&registry).await?;
    ConversionOperations::register_all(&registry).await?;
    MathOperations::register_all(&registry).await?;
    TypeOperations::register_all(&registry).await?;
    CdaOperations::register_all(&registry).await?;
    LogicalOperations::register_all(&registry).await?;

    Ok(registry)
}
