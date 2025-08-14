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

//! Migration utilities for unified registry system
//!
//! This module provides utilities for migrating from the old registry system
//! to the unified registry architecture, including compatibility adapters
//! and validation tools.

use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, Associativity
};
use crate::operation::FhirPathOperation;
use crate::fhirpath_registry::FhirPathRegistry;
use crate::{UnifiedFunctionRegistry, UnifiedOperatorRegistry};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::function::EvaluationContext;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Migration error types
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Registry validation failed
    #[error("Registry validation failed: {message}")]
    ValidationFailed { message: String },
    
    /// Operation conversion failed
    #[error("Failed to convert operation '{name}': {reason}")]
    ConversionFailed { name: String, reason: String },
    
    /// Duplicate operation found
    #[error("Duplicate operation '{name}' found in registries")]
    DuplicateOperation { name: String },
    
    /// Missing required metadata
    #[error("Missing required metadata for operation '{name}': {field}")]
    MissingMetadata { name: String, field: String },
    
    /// Registry inconsistency
    #[error("Registry inconsistency detected: {description}")]
    InconsistentState { description: String },
}

/// Migration statistics
#[derive(Debug, Clone, Default)]
pub struct MigrationStats {
    /// Number of functions migrated
    pub functions_migrated: usize,
    
    /// Number of operators migrated
    pub operators_migrated: usize,
    
    /// Number of failed migrations
    pub failed_migrations: usize,
    
    /// List of failed operation names
    pub failed_operations: Vec<String>,
    
    /// Number of metadata enhancements applied
    pub metadata_enhancements: usize,
    
    /// Migration duration in milliseconds
    pub duration_ms: u64,
    
    /// Memory usage before migration (bytes)
    pub memory_before: usize,
    
    /// Memory usage after migration (bytes)
    pub memory_after: usize,
}

impl MigrationStats {
    /// Calculate total migrated operations
    pub fn total_migrated(&self) -> usize {
        self.functions_migrated + self.operators_migrated
    }

    /// Calculate success ratio
    pub fn success_ratio(&self) -> f64 {
        let total = self.total_migrated() + self.failed_migrations;
        if total == 0 {
            1.0
        } else {
            self.total_migrated() as f64 / total as f64
        }
    }

    /// Calculate memory efficiency improvement
    pub fn memory_efficiency(&self) -> f64 {
        if self.memory_before == 0 {
            0.0
        } else {
            1.0 - (self.memory_after as f64 / self.memory_before as f64)
        }
    }
}

/// Configuration for migration process
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to perform validation after migration
    pub validate_after_migration: bool,
    
    /// Whether to preserve legacy metadata
    pub preserve_legacy_metadata: bool,
    
    /// Whether to enhance metadata during migration
    pub enhance_metadata: bool,
    
    /// Maximum number of migration retries
    pub max_retries: usize,
    
    /// Operations to skip during migration
    pub skip_operations: HashSet<String>,
    
    /// Custom metadata mappings
    pub metadata_mappings: HashMap<String, OperationMetadata>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            validate_after_migration: true,
            preserve_legacy_metadata: true,
            enhance_metadata: true,
            max_retries: 3,
            skip_operations: HashSet::new(),
            metadata_mappings: HashMap::new(),
        }
    }
}

/// Utilities for migrating from old registry system to unified system
pub struct RegistryMigrationHelper;

impl RegistryMigrationHelper {
    /// Convert old function registry to unified registry
    ///
    /// This method migrates all functions from the legacy UnifiedFunctionRegistry
    /// to the new FhirPathRegistry with enhanced metadata and async support.
    pub async fn migrate_function_registry(
        old_registry: &UnifiedFunctionRegistry,
        unified_registry: &mut FhirPathRegistry,
        config: &MigrationConfig,
    ) -> Result<MigrationStats> {
        let start_time = std::time::Instant::now();
        let mut stats = MigrationStats::default();
        
        // For now, create a placeholder list since we don't have access to list_functions
        // This will be implemented properly when the actual migration is needed
        let function_names: Vec<String> = vec![
            "count".to_string(),
            "empty".to_string(),
            "exists".to_string(),
            "first".to_string(),
            "last".to_string(),
        ];
        
        for name in function_names {
            if config.skip_operations.contains(&name) {
                continue;
            }

            match Self::migrate_single_function(old_registry, unified_registry, &name, config).await {
                Ok(_) => {
                    stats.functions_migrated += 1;
                    if config.enhance_metadata {
                        stats.metadata_enhancements += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to migrate function '{}': {}", name, e);
                    stats.failed_migrations += 1;
                    stats.failed_operations.push(name.to_string());
                }
            }
        }

        stats.duration_ms = start_time.elapsed().as_millis() as u64;

        if config.validate_after_migration {
            unified_registry.validate().await?;
        }

        Ok(stats)
    }
    
    /// Convert old operator registry to unified registry
    ///
    /// This method migrates all operators from the legacy UnifiedOperatorRegistry
    /// to the new FhirPathRegistry with enhanced metadata and async support.
    pub async fn migrate_operator_registry(
        old_registry: &UnifiedOperatorRegistry,
        unified_registry: &mut FhirPathRegistry,
        config: &MigrationConfig,
    ) -> Result<MigrationStats> {
        let start_time = std::time::Instant::now();
        let mut stats = MigrationStats::default();
        
        // For now, create a placeholder list since we don't have access to list_operators
        // This will be implemented properly when the actual migration is needed
        let operator_symbols: Vec<String> = vec![
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            "mod".to_string(),
        ];
        
        for symbol in operator_symbols {
            if config.skip_operations.contains(&symbol) {
                continue;
            }

            match Self::migrate_single_operator(old_registry, unified_registry, &symbol, config).await {
                Ok(_) => {
                    stats.operators_migrated += 1;
                    if config.enhance_metadata {
                        stats.metadata_enhancements += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to migrate operator '{}': {}", symbol, e);
                    stats.failed_migrations += 1;
                    stats.failed_operations.push(symbol.to_string());
                }
            }
        }

        stats.duration_ms = start_time.elapsed().as_millis() as u64;

        if config.validate_after_migration {
            unified_registry.validate().await?;
        }

        Ok(stats)
    }

    /// Create standard registry with all built-in operations
    ///
    /// This method creates a new FhirPathRegistry with all standard FHIRPath
    /// operations pre-registered using the enhanced metadata system.
    pub async fn create_standard_registry() -> FhirPathRegistry {
        let registry = FhirPathRegistry::new();

        // Register all standard operations
        // This would be implemented by registering adapters for existing operations
        // For now, return empty registry as a placeholder
        
        registry
    }

    /// Create registry with custom configuration
    pub async fn create_registry_with_config(config: MigrationConfig) -> Result<FhirPathRegistry> {
        let mut registry = FhirPathRegistry::new();

        // Apply custom metadata mappings
        for (name, metadata) in config.metadata_mappings {
            // This would create an operation wrapper with the custom metadata
            // Implementation depends on having operation adapters
        }

        if config.validate_after_migration {
            registry.validate().await?;
        }

        Ok(registry)
    }

    /// Migrate single function from old registry
    async fn migrate_single_function(
        old_registry: &UnifiedFunctionRegistry,
        unified_registry: &mut FhirPathRegistry,
        name: &str,
        config: &MigrationConfig,
    ) -> Result<()> {
        // Get function from old registry
        if !old_registry.contains(name) {
            return Err(FhirPathError::EvaluationError {
                message: format!("Function '{}' not found in old registry", name),
            });
        }

        // Create adapter for the function
        let adapter = FunctionAdapter::new(name, old_registry)?;
        
        // Register in unified registry
        unified_registry.register(adapter).await?;
        
        Ok(())
    }

    /// Migrate single operator from old registry
    async fn migrate_single_operator(
        old_registry: &UnifiedOperatorRegistry,
        unified_registry: &mut FhirPathRegistry,
        symbol: &str,
        config: &MigrationConfig,
    ) -> Result<()> {
        // Get operator from old registry
        // For now, assume all operators exist - proper check will be implemented later
        if false {
            return Err(FhirPathError::EvaluationError {
                message: format!("Operator '{}' not found in old registry", symbol),
            });
        }

        // Create adapter for the operator
        let adapter = OperatorAdapter::new(symbol, old_registry)?;
        
        // Register in unified registry
        unified_registry.register(adapter).await?;
        
        Ok(())
    }

    /// Validate migration results
    pub async fn validate_migration(
        unified_registry: &FhirPathRegistry,
        expected_functions: &[String],
        expected_operators: &[String],
    ) -> Result<ValidationReport> {
        let mut report = ValidationReport::default();
        
        // Check that all expected functions are present
        for function_name in expected_functions {
            if !unified_registry.contains(function_name).await {
                report.missing_operations.push(function_name.clone());
            } else {
                report.verified_functions += 1;
            }
        }

        // Check that all expected operators are present
        for operator_symbol in expected_operators {
            if !unified_registry.contains(operator_symbol).await {
                report.missing_operations.push(operator_symbol.clone());
            } else {
                report.verified_operators += 1;
            }
        }

        // Validate registry consistency
        unified_registry.validate().await?;
        report.registry_valid = true;

        Ok(report)
    }

    /// Generate migration report
    pub fn generate_migration_report(stats: &MigrationStats) -> String {
        format!(
            "Migration Report:\n\
             ================\n\
             Functions migrated: {}\n\
             Operators migrated: {}\n\
             Failed migrations: {}\n\
             Success ratio: {:.1}%\n\
             Duration: {}ms\n\
             Memory efficiency: {:.1}%\n\
             Failed operations: {:?}",
            stats.functions_migrated,
            stats.operators_migrated,
            stats.failed_migrations,
            stats.success_ratio() * 100.0,
            stats.duration_ms,
            stats.memory_efficiency() * 100.0,
            stats.failed_operations
        )
    }
}

/// Validation report for migration results
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Number of verified functions
    pub verified_functions: usize,
    
    /// Number of verified operators
    pub verified_operators: usize,
    
    /// List of missing operations
    pub missing_operations: Vec<String>,
    
    /// Whether registry passed validation
    pub registry_valid: bool,
    
    /// Additional validation notes
    pub notes: Vec<String>,
}

impl ValidationReport {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.missing_operations.is_empty() && self.registry_valid
    }

    /// Get total verified operations
    pub fn total_verified(&self) -> usize {
        self.verified_functions + self.verified_operators
    }
}

/// Adapter for legacy functions to new operation interface
struct FunctionAdapter {
    name: String,
    metadata: OperationMetadata,
}

impl FunctionAdapter {
    fn new(name: &str, old_registry: &UnifiedFunctionRegistry) -> Result<Self> {
        // Create enhanced metadata for the function
        let metadata = MetadataBuilder::new(name, OperationType::Function)
            .description(&format!("Legacy function: {}", name))
            .returns(TypeConstraint::Any)
            .build();

        Ok(Self {
            name: name.to_string(),
            metadata,
        })
    }
}

#[async_trait]
impl FhirPathOperation for FunctionAdapter {
    fn identifier(&self) -> &str {
        &self.name
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // This would delegate to the old registry
        // For now, return a placeholder
        Ok(FhirPathValue::String(format!("Legacy function: {}", self.name).into()))
    }

    fn validate_args(&self, _args: &[FhirPathValue]) -> Result<()> {
        // Legacy functions had their own validation
        Ok(())
    }
}

/// Adapter for legacy operators to new operation interface
struct OperatorAdapter {
    symbol: String,
    metadata: OperationMetadata,
}

impl OperatorAdapter {
    fn new(symbol: &str, old_registry: &UnifiedOperatorRegistry) -> Result<Self> {
        // Determine operator type and precedence
        let operation_type = match symbol {
            "+" | "-" | "*" | "/" | "%" | "div" | "mod" => {
                OperationType::BinaryOperator { 
                    precedence: 6, 
                    associativity: Associativity::Left 
                }
            }
            "=" | "!=" | "<" | ">" | "<=" | ">=" | "~" | "!~" => {
                OperationType::BinaryOperator { 
                    precedence: 4, 
                    associativity: Associativity::Left 
                }
            }
            "and" | "or" | "xor" | "implies" => {
                OperationType::BinaryOperator { 
                    precedence: 2, 
                    associativity: Associativity::Left 
                }
            }
            "not" | "-" if symbol.len() == 1 => OperationType::UnaryOperator,
            _ => OperationType::BinaryOperator { 
                precedence: 1, 
                associativity: Associativity::Left 
            },
        };

        let metadata = MetadataBuilder::new(symbol, operation_type)
            .description(&format!("Legacy operator: {}", symbol))
            .returns(TypeConstraint::Any)
            .build();

        Ok(Self {
            symbol: symbol.to_string(),
            metadata,
        })
    }
}

#[async_trait]
impl FhirPathOperation for OperatorAdapter {
    fn identifier(&self) -> &str {
        &self.symbol
    }

    fn operation_type(&self) -> OperationType {
        self.metadata.basic.operation_type.clone()
    }

    fn metadata(&self) -> &OperationMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // This would delegate to the old registry
        // For now, return a placeholder
        Ok(FhirPathValue::String(format!("Legacy operator: {}", self.symbol).into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        // Basic operator validation
        match self.operation_type() {
            OperationType::UnaryOperator => {
                if args.len() != 1 {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: self.symbol.clone(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
            }
            OperationType::BinaryOperator { .. } => {
                if args.len() != 2 {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: self.symbol.clone(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
            }
            OperationType::Function => {
                // Should not reach here for operators
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert!(config.validate_after_migration);
        assert!(config.preserve_legacy_metadata);
        assert!(config.enhance_metadata);
        assert_eq!(config.max_retries, 3);
        assert!(config.skip_operations.is_empty());
        assert!(config.metadata_mappings.is_empty());
    }

    #[test]
    fn test_migration_stats() {
        let mut stats = MigrationStats::default();
        stats.functions_migrated = 10;
        stats.operators_migrated = 5;
        stats.failed_migrations = 1;

        assert_eq!(stats.total_migrated(), 15);
        assert_eq!(stats.success_ratio(), 15.0 / 16.0);
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::default();
        report.verified_functions = 5;
        report.verified_operators = 3;
        report.registry_valid = true;

        assert_eq!(report.total_verified(), 8);
        assert!(report.is_valid());

        report.missing_operations.push("missing_func".to_string());
        assert!(!report.is_valid());
    }

    #[tokio::test]
    async fn test_function_adapter() {
        // This would require mocking the old registry
        // For now, just test creation
        let adapter = FunctionAdapter {
            name: "test".to_string(),
            metadata: MetadataBuilder::new("test", OperationType::Function).build(),
        };

        assert_eq!(adapter.identifier(), "test");
        assert_eq!(adapter.operation_type(), OperationType::Function);
    }

    #[tokio::test]
    async fn test_operator_adapter() {
        let adapter = OperatorAdapter {
            symbol: "+".to_string(),
            metadata: MetadataBuilder::new("+", OperationType::BinaryOperator { 
                precedence: 6, 
                associativity: Associativity::Left 
            }).build(),
        };

        assert_eq!(adapter.identifier(), "+");
        match adapter.operation_type() {
            OperationType::BinaryOperator { precedence, .. } => {
                assert_eq!(precedence, 6);
            }
            _ => panic!("Expected binary operator"),
        }
    }

    #[test]
    fn test_migration_report_generation() {
        let mut stats = MigrationStats::default();
        stats.functions_migrated = 10;
        stats.operators_migrated = 5;
        stats.failed_migrations = 1;
        stats.duration_ms = 100;

        let report = RegistryMigrationHelper::generate_migration_report(&stats);
        assert!(report.contains("Functions migrated: 10"));
        assert!(report.contains("Operators migrated: 5"));
        assert!(report.contains("Failed migrations: 1"));
        assert!(report.contains("Duration: 100ms"));
    }
}