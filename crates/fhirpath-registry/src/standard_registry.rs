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

//! Standard FHIRPath Registry Builder
//!
//! This module provides a configurable builder for creating FHIRPath registries
//! with different combinations of operations. It allows fine-grained control over
//! which operations are included, performance settings, and validation options.

use crate::{FhirPathRegistry, FhirPathOperation};
use crate::operations::*;
use octofhir_fhirpath_core::Result;
use std::collections::HashSet;
use thiserror::Error;

/// Builder errors
#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("Registry operation failed: {message}")]
    RegistryError { message: String },
    
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
    
    #[error("Operation '{name}' is not available")]
    OperationNotAvailable { name: String },
    
    #[error("Conflicting configuration: {message}")]
    ConflictingConfiguration { message: String },
}

/// Configuration options for the standard registry builder
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Include arithmetic operators (+, -, *, /, mod, div)
    pub include_arithmetic: bool,
    
    /// Include collection functions (count, empty, exists, first, last, single)
    pub include_collection_functions: bool,
    
    /// Include string functions (length, contains, startsWith, endsWith, substring)
    pub include_string_functions: bool,
    
    /// Include comparison operators (=, !=, <, >, <=, >=)
    pub include_comparison_operators: bool,
    
    /// Include logical operators (and, or, not, xor, implies)
    pub include_logical_operators: bool,
    
    /// Include type checking operators (is, as)
    pub include_type_operators: bool,
    
    /// Include math functions (abs, ceiling, floor, round, sqrt, etc.)
    pub include_math_functions: bool,
    
    /// Include date/time functions (now, today, timeOfDay)
    pub include_datetime_functions: bool,
    
    /// Include FHIR-specific functions (resolve, extension, conformsTo)
    pub include_fhir_functions: bool,
    
    /// Include utility functions (trace, iif, hasValue)
    pub include_utility_functions: bool,
    
    /// Specific operations to exclude (overrides include settings)
    pub excluded_operations: HashSet<String>,
    
    /// Specific operations to include (in addition to category includes)
    pub additional_operations: Vec<Box<dyn FhirPathOperation>>,
    
    /// Enable performance optimizations
    pub enable_optimizations: bool,
    
    /// Enable operation caching
    pub enable_caching: bool,
    
    /// Validate operations after registration
    pub validate_operations: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            include_arithmetic: true,
            include_collection_functions: true,
            include_string_functions: true,
            include_comparison_operators: true,
            include_logical_operators: true,
            include_type_operators: true,
            include_math_functions: true,
            include_datetime_functions: true,
            include_fhir_functions: true,
            include_utility_functions: true,
            excluded_operations: HashSet::new(),
            additional_operations: Vec::new(),
            enable_optimizations: true,
            enable_caching: true,
            validate_operations: true,
        }
    }
}

impl RegistryConfig {
    /// Create a minimal configuration with only basic operations
    pub fn minimal() -> Self {
        Self {
            include_arithmetic: true,
            include_collection_functions: true,
            include_string_functions: false,
            include_comparison_operators: true,
            include_logical_operators: false,
            include_type_operators: false,
            include_math_functions: false,
            include_datetime_functions: false,
            include_fhir_functions: false,
            include_utility_functions: false,
            excluded_operations: HashSet::new(),
            additional_operations: Vec::new(),
            enable_optimizations: true,
            enable_caching: false,
            validate_operations: true,
        }
    }

    /// Create a performance-focused configuration
    pub fn performance() -> Self {
        Self {
            include_arithmetic: true,
            include_collection_functions: true,
            include_string_functions: true,
            include_comparison_operators: true,
            include_logical_operators: true,
            include_type_operators: false,
            include_math_functions: true,
            include_datetime_functions: false,
            include_fhir_functions: false,
            include_utility_functions: false,
            excluded_operations: HashSet::new(),
            additional_operations: Vec::new(),
            enable_optimizations: true,
            enable_caching: true,
            validate_operations: false, // Skip validation for performance
        }
    }

    /// Create a FHIR-focused configuration
    pub fn fhir_complete() -> Self {
        Self {
            include_arithmetic: true,
            include_collection_functions: true,
            include_string_functions: true,
            include_comparison_operators: true,
            include_logical_operators: true,
            include_type_operators: true,
            include_math_functions: true,
            include_datetime_functions: true,
            include_fhir_functions: true,
            include_utility_functions: true,
            excluded_operations: HashSet::new(),
            additional_operations: Vec::new(),
            enable_optimizations: true,
            enable_caching: true,
            validate_operations: true,
        }
    }

    /// Exclude specific operations by name
    pub fn exclude_operation(mut self, operation_name: &str) -> Self {
        self.excluded_operations.insert(operation_name.to_string());
        self
    }

    /// Exclude multiple operations
    pub fn exclude_operations(mut self, operation_names: &[&str]) -> Self {
        for name in operation_names {
            self.excluded_operations.insert(name.to_string());
        }
        self
    }

    /// Add a custom operation
    pub fn add_operation(mut self, operation: Box<dyn FhirPathOperation>) -> Self {
        self.additional_operations.push(operation);
        self
    }

    /// Disable optimizations for debugging
    pub fn without_optimizations(mut self) -> Self {
        self.enable_optimizations = false;
        self.enable_caching = false;
        self
    }

    /// Enable strict validation
    pub fn with_strict_validation(mut self) -> Self {
        self.validate_operations = true;
        self
    }
}

/// Builder for creating configured FHIRPath registries
pub struct StandardRegistryBuilder {
    config: RegistryConfig,
}

impl StandardRegistryBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: RegistryConfig::default(),
        }
    }

    /// Create a builder with a specific configuration
    pub fn with_config(config: RegistryConfig) -> Self {
        Self { config }
    }

    /// Create a minimal registry
    pub fn minimal() -> Self {
        Self {
            config: RegistryConfig::minimal(),
        }
    }

    /// Create a performance-optimized registry
    pub fn performance() -> Self {
        Self {
            config: RegistryConfig::performance(),
        }
    }

    /// Create a complete FHIR registry
    pub fn fhir_complete() -> Self {
        Self {
            config: RegistryConfig::fhir_complete(),
        }
    }

    /// Update the configuration
    pub fn configure(mut self, f: impl FnOnce(RegistryConfig) -> RegistryConfig) -> Self {
        self.config = f(self.config);
        self
    }

    /// Include arithmetic operations
    pub fn with_arithmetic(mut self, enabled: bool) -> Self {
        self.config.include_arithmetic = enabled;
        self
    }

    /// Include collection functions
    pub fn with_collection_functions(mut self, enabled: bool) -> Self {
        self.config.include_collection_functions = enabled;
        self
    }

    /// Include string functions
    pub fn with_string_functions(mut self, enabled: bool) -> Self {
        self.config.include_string_functions = enabled;
        self
    }

    /// Include comparison operators
    pub fn with_comparison_operators(mut self, enabled: bool) -> Self {
        self.config.include_comparison_operators = enabled;
        self
    }

    /// Include logical operators
    pub fn with_logical_operators(mut self, enabled: bool) -> Self {
        self.config.include_logical_operators = enabled;
        self
    }

    /// Include math functions
    pub fn with_math_functions(mut self, enabled: bool) -> Self {
        self.config.include_math_functions = enabled;
        self
    }

    /// Include FHIR-specific functions
    pub fn with_fhir_functions(mut self, enabled: bool) -> Self {
        self.config.include_fhir_functions = enabled;
        self
    }

    /// Exclude specific operations
    pub fn exclude_operation(mut self, operation_name: &str) -> Self {
        self.config.excluded_operations.insert(operation_name.to_string());
        self
    }

    /// Add a custom operation
    pub fn add_operation(mut self, operation: Box<dyn FhirPathOperation>) -> Self {
        self.config.additional_operations.push(operation);
        self
    }

    /// Enable or disable optimizations
    pub fn with_optimizations(mut self, enabled: bool) -> Self {
        self.config.enable_optimizations = enabled;
        self
    }

    /// Enable or disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.config.enable_caching = enabled;
        self
    }

    /// Build the registry asynchronously
    pub async fn build(self) -> Result<FhirPathRegistry> {
        let mut registry = FhirPathRegistry::new();

        // Apply configuration settings
        if self.config.enable_caching {
            // Registry caching is enabled by default, no action needed
        }

        // Register operations based on configuration
        if self.config.include_arithmetic {
            self.register_arithmetic_operations(&mut registry).await?;
        }

        if self.config.include_collection_functions {
            self.register_collection_functions(&mut registry).await?;
        }

        if self.config.include_string_functions {
            self.register_string_functions(&mut registry).await?;
        }

        if self.config.include_comparison_operators {
            self.register_comparison_operators(&mut registry).await?;
        }

        if self.config.include_logical_operators {
            self.register_logical_operators(&mut registry).await?;
        }

        if self.config.include_type_operators {
            self.register_type_operators(&mut registry).await?;
        }

        if self.config.include_math_functions {
            self.register_math_functions(&mut registry).await?;
        }

        if self.config.include_datetime_functions {
            self.register_datetime_functions(&mut registry).await?;
        }

        if self.config.include_fhir_functions {
            self.register_fhir_functions(&mut registry).await?;
        }

        if self.config.include_utility_functions {
            self.register_utility_functions(&mut registry).await?;
        }

        // Register additional operations
        for operation in self.config.additional_operations {
            if !self.config.excluded_operations.contains(operation.identifier()) {
                registry.register(operation).await
                    .map_err(|e| BuilderError::RegistryError {
                        message: format!("Failed to register additional operation: {}", e),
                    })?;
            }
        }

        // Remove excluded operations
        for operation_name in &self.config.excluded_operations {
            registry.unregister(operation_name).await;
        }

        // Validate if requested
        if self.config.validate_operations {
            registry.validate().await
                .map_err(|e| BuilderError::RegistryError {
                    message: format!("Registry validation failed: {}", e),
                })?;
        }

        Ok(registry)
    }

    /// Register arithmetic operations
    async fn register_arithmetic_operations(&self, registry: &mut FhirPathRegistry) -> Result<()> {
        let operations: Vec<Box<dyn FhirPathOperation>> = vec![
            Box::new(AdditionOperation::new()),
            Box::new(SubtractionOperation::new()),
            Box::new(MultiplicationOperation::new()),
            Box::new(DivisionOperation::new()),
            Box::new(ModuloOperation::new()),
            Box::new(IntegerDivisionOperation::new()),
            Box::new(UnaryMinusOperation::new()),
            Box::new(UnaryPlusOperation::new()),
        ];

        for operation in operations {
            if !self.config.excluded_operations.contains(operation.identifier()) {
                registry.register(operation).await
                    .map_err(|e| BuilderError::RegistryError {
                        message: format!("Failed to register arithmetic operation: {}", e),
                    })?;
            }
        }

        Ok(())
    }

    /// Register collection functions
    async fn register_collection_functions(&self, registry: &mut FhirPathRegistry) -> Result<()> {
        let operations: Vec<Box<dyn FhirPathOperation>> = vec![
            Box::new(CountFunction::new()),
            Box::new(EmptyFunction::new()),
            Box::new(ExistsFunction::new()),
            Box::new(FirstFunction::new()),
            Box::new(LastFunction::new()),
            Box::new(SingleFunction::new()),
        ];

        for operation in operations {
            if !self.config.excluded_operations.contains(operation.identifier()) {
                registry.register(operation).await
                    .map_err(|e| BuilderError::RegistryError {
                        message: format!("Failed to register collection function: {}", e),
                    })?;
            }
        }

        Ok(())
    }

    /// Register string functions
    async fn register_string_functions(&self, registry: &mut FhirPathRegistry) -> Result<()> {
        let operations: Vec<Box<dyn FhirPathOperation>> = vec![
            Box::new(LengthFunction::new()),
            Box::new(ContainsFunction::new()),
            Box::new(StartsWithFunction::new()),
            Box::new(EndsWithFunction::new()),
            Box::new(SubstringFunction::new()),
        ];

        for operation in operations {
            if !self.config.excluded_operations.contains(operation.identifier()) {
                registry.register(operation).await
                    .map_err(|e| BuilderError::RegistryError {
                        message: format!("Failed to register string function: {}", e),
                    })?;
            }
        }

        Ok(())
    }

    /// Register comparison operators (placeholder - not yet implemented)
    async fn register_comparison_operators(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when comparison operators are available
        Ok(())
    }

    /// Register logical operators (placeholder - not yet implemented)
    async fn register_logical_operators(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when logical operators are available
        Ok(())
    }

    /// Register type operators (placeholder - not yet implemented)
    async fn register_type_operators(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when type operators are available
        Ok(())
    }

    /// Register math functions (placeholder - not yet implemented)
    async fn register_math_functions(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when math functions are available
        Ok(())
    }

    /// Register datetime functions (placeholder - not yet implemented)
    async fn register_datetime_functions(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when datetime functions are available
        Ok(())
    }

    /// Register FHIR functions (placeholder - not yet implemented)
    async fn register_fhir_functions(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when FHIR functions are available
        Ok(())
    }

    /// Register utility functions (placeholder - not yet implemented)
    async fn register_utility_functions(&self, _registry: &mut FhirPathRegistry) -> Result<()> {
        // TODO: Implement when utility functions are available
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut RegistryConfig {
        &mut self.config
    }
}

impl Default for StandardRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a standard registry with default configuration
pub async fn create_standard_registry() -> Result<FhirPathRegistry> {
    StandardRegistryBuilder::new().build().await
}

/// Convenience function to create a minimal registry
pub async fn create_minimal_registry() -> Result<FhirPathRegistry> {
    StandardRegistryBuilder::minimal().build().await
}

/// Convenience function to create a performance-optimized registry
pub async fn create_performance_registry() -> Result<FhirPathRegistry> {
    StandardRegistryBuilder::performance().build().await
}

/// Convenience function to create a complete FHIR registry
pub async fn create_fhir_registry() -> Result<FhirPathRegistry> {
    StandardRegistryBuilder::fhir_complete().build().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standard_registry_creation() {
        let registry = create_standard_registry().await.unwrap();
        
        // Test that basic operations are available
        assert!(registry.contains("count").await);
        assert!(registry.contains("+").await);
        assert!(registry.contains("length").await);
    }

    #[tokio::test]
    async fn test_minimal_registry() {
        let registry = create_minimal_registry().await.unwrap();
        
        // Should have arithmetic and collection
        assert!(registry.contains("count").await);
        assert!(registry.contains("+").await);
        
        // Should not have string functions in minimal config
        assert!(!registry.contains("length").await);
    }

    #[tokio::test]
    async fn test_custom_configuration() {
        let registry = StandardRegistryBuilder::new()
            .with_arithmetic(true)
            .with_collection_functions(true)
            .with_string_functions(false)
            .exclude_operation("count")
            .build()
            .await
            .unwrap();
        
        // Should have arithmetic but not count
        assert!(registry.contains("+").await);
        assert!(!registry.contains("count").await);
        
        // Should not have string functions
        assert!(!registry.contains("length").await);
    }

    #[tokio::test]
    async fn test_performance_registry() {
        let registry = create_performance_registry().await.unwrap();
        
        // Should have core operations
        assert!(registry.contains("+").await);
        assert!(registry.contains("count").await);
        assert!(registry.contains("length").await);
    }

    #[tokio::test]
    async fn test_builder_chaining() {
        let registry = StandardRegistryBuilder::minimal()
            .with_string_functions(true)
            .with_optimizations(false)
            .exclude_operation("single")
            .build()
            .await
            .unwrap();
        
        // Should have string functions added to minimal
        assert!(registry.contains("length").await);
        
        // Should not have excluded operation
        assert!(!registry.contains("single").await);
    }

    #[tokio::test]
    async fn test_configuration_presets() {
        // Test that all presets build successfully
        let _minimal = StandardRegistryBuilder::minimal().build().await.unwrap();
        let _performance = StandardRegistryBuilder::performance().build().await.unwrap();
        let _fhir = StandardRegistryBuilder::fhir_complete().build().await.unwrap();
    }
}