// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simplified FHIRPath Registry
//!
//! This crate provides a clean, simplified registry system that splits operations into
//! sync/async based on their actual needs, not artificial complexity.
//! # Usage
//! ```rust
//! use octofhir_fhirpath_registry::{
//!     traits::{SyncOperation, AsyncOperation, EvaluationContext},
//!     signature::{FunctionSignature, ValueType, ParameterType},
//!     FunctionRegistry,
//! };
//! ```
// Simplified system modules
pub mod function_registry;
pub mod macros;
pub mod registry;
pub mod registry_core;
pub mod signature;
pub mod traits;
// Operation implementations
pub mod operations;
// Bridge support modules
pub mod package_manager;
pub mod schema_aware_registry;
pub mod type_registry;
// Test modules
#[cfg(test)]
mod integration_test;
#[cfg(test)]
mod tests;
// Main function registry exports
pub use function_registry::FunctionRegistry;
// Bridge support exports
pub use package_manager::{PackageError, PackageInfo, RefreshableRegistry, RegistryPackageManager};
pub use schema_aware_registry::SchemaAwareFunctionRegistry;
pub use type_registry::{FhirPathTypeRegistry, RegistryError};
// Re-exports from workspace crates
pub use octofhir_fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};
/// Create a standard unified registry with all built-in operations
///
/// This creates the unified registry with optimized sync/async dispatch and pre-warmed cache.
/// This is the recommended way to create a registry for projects.
/// # Returns
/// A fully configured `FunctionRegistry` with all standard FHIRPath operations registered.
/// # Examples
/// ```rust
/// use octofhir_fhirpath_registry::create_standard_registry;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = create_standard_registry().await;
///     
///     // Evaluate expressions with smart dispatch
///     // let context = EvaluationContext { input: my_data, model_provider, variables };
///     // let result = registry.evaluate("count", &[], &context).await?;
///     Ok(())
/// }
/// ```
pub async fn create_standard_registry() -> FunctionRegistry {
    crate::function_registry::create_standard_registry().await
}

/// Create a schema-aware registry with bridge support
///
/// This creates a registry with full schema awareness and O(1) type checking.
/// Recommended for production applications requiring full FHIR compliance.
/// # Arguments
/// * `schema_manager` - The schema manager for bridge API access
/// # Returns
/// A fully configured `SchemaAwareFunctionRegistry` with schema-aware operations.
/// # Examples
/// ```rust
/// use octofhir_fhirpath_registry::create_schema_aware_registry;
/// use octofhir_fhirschema::package::FhirSchemaPackageManager;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let schema_manager = Arc::new(FhirSchemaPackageManager::new(config).await?);
///     let registry = create_schema_aware_registry(schema_manager).await?;
///     
///     // Use schema-aware functions with O(1) type checking
///     // let result = registry.evaluate_function("ofType", &args, &context).await?;
///     Ok(())
/// }
/// ```
pub async fn create_schema_aware_registry(
    schema_manager: std::sync::Arc<octofhir_fhirschema::package::FhirSchemaPackageManager>,
) -> std::result::Result<SchemaAwareFunctionRegistry, RegistryError> {
    SchemaAwareFunctionRegistry::new(schema_manager).await
}
