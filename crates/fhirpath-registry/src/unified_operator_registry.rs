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

//! Unified operator registry with enhanced capabilities

use crate::enhanced_operator_metadata::{EnhancedOperatorMetadata, OperatorCategory};
use crate::unified_operator::Associativity;
use crate::operator_lsp::{CompletionContext, OperatorLspProvider};
use crate::unified_operator::UnifiedFhirPathOperator;
use crate::function::EvaluationContext;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;

/// Result type for registry operations
pub type RegistryResult<T> = Result<T, OperatorRegistryError>;

/// Operator registry errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OperatorRegistryError {
    /// Operator not found
    #[error("Operator '{symbol}' not found")]
    OperatorNotFound { symbol: String },
    
    /// Operator already registered
    #[error("Operator '{symbol}' is already registered")]
    OperatorAlreadyExists { symbol: String },
    
    /// Invalid operator configuration
    #[error("Invalid operator configuration: {message}")]
    InvalidConfiguration { message: String },
    
    /// Evaluation error
    #[error("Error evaluating operator '{symbol}': {message}")]
    EvaluationError { symbol: String, message: String },
}

impl From<OperatorRegistryError> for octofhir_fhirpath_core::EvaluationError {
    fn from(error: OperatorRegistryError) -> Self {
        match error {
            OperatorRegistryError::OperatorNotFound { symbol } => {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation {
                    message: format!("Operator '{}' not found", symbol),
                }
            }
            OperatorRegistryError::OperatorAlreadyExists { symbol } => {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation {
                    message: format!("Operator '{}' is already registered", symbol),
                }
            }
            OperatorRegistryError::InvalidConfiguration { message } => {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation { message }
            }
            OperatorRegistryError::EvaluationError { symbol, message } => {
                octofhir_fhirpath_core::EvaluationError::Operator(format!(
                    "Error evaluating operator '{}': {}",
                    symbol, message
                ))
            }
        }
    }
}

/// Statistics about the operator registry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRegistryStats {
    /// Total number of operators
    pub total_operators: usize,
    
    /// Number of binary operators
    pub binary_operators: usize,
    
    /// Number of unary operators
    pub unary_operators: usize,
    
    /// Number of operators by category
    pub operators_by_category: FxHashMap<OperatorCategory, usize>,
    
    /// Number of commutative operators
    pub commutative_operators: usize,
    
    /// Number of optimizable operators
    pub optimizable_operators: usize,
}

/// Enhanced operator registry with unified operator support and LSP integration
#[derive(Clone)]
pub struct UnifiedOperatorRegistry {
    /// Binary operators by symbol
    binary_operators: FxHashMap<String, Arc<dyn UnifiedFhirPathOperator>>,
    
    /// Unary operators by symbol
    unary_operators: FxHashMap<String, Arc<dyn UnifiedFhirPathOperator>>,
    
    /// Operator precedence lookup
    precedence: FxHashMap<String, u8>,
    
    /// Operator associativity lookup
    associativity: FxHashMap<String, Associativity>,
    
    /// LSP support provider
    lsp_provider: OperatorLspProvider,
    
    /// Registry statistics
    stats: OperatorRegistryStats,
}

impl UnifiedOperatorRegistry {
    /// Create a new unified operator registry
    pub fn new() -> Self {
        Self {
            binary_operators: FxHashMap::default(),
            unary_operators: FxHashMap::default(),
            precedence: FxHashMap::default(),
            associativity: FxHashMap::default(),
            lsp_provider: OperatorLspProvider::new(),
            stats: OperatorRegistryStats {
                total_operators: 0,
                binary_operators: 0,
                unary_operators: 0,
                operators_by_category: FxHashMap::default(),
                commutative_operators: 0,
                optimizable_operators: 0,
            },
        }
    }

    /// Register a unified operator
    pub fn register<O: UnifiedFhirPathOperator + 'static>(&mut self, operator: O) -> RegistryResult<()> {
        let symbol = operator.symbol().to_string();
        
        // Check if already registered
        if self.binary_operators.contains_key(&symbol) || self.unary_operators.contains_key(&symbol) {
            return Err(OperatorRegistryError::OperatorAlreadyExists { symbol });
        }

        let arc_op = Arc::new(operator);
        let metadata = arc_op.metadata().clone();
        
        // Store precedence and associativity
        self.precedence.insert(symbol.clone(), arc_op.precedence());
        self.associativity.insert(symbol.clone(), arc_op.associativity());
        
        // Register based on supported operations
        if arc_op.supports_binary() {
            self.binary_operators.insert(symbol.clone(), arc_op.clone());
            self.stats.binary_operators += 1;
        }
        
        if arc_op.supports_unary() {
            self.unary_operators.insert(symbol.clone(), arc_op);
            self.stats.unary_operators += 1;
        }
        
        // Register with LSP provider
        self.lsp_provider.register_operator(metadata.clone());
        
        // Update statistics
        self.update_stats(&metadata);
        
        Ok(())
    }

    /// Get a binary operator by symbol
    pub fn get_binary(&self, symbol: &str) -> Option<Arc<dyn UnifiedFhirPathOperator>> {
        self.binary_operators.get(symbol).cloned()
    }

    /// Get a unary operator by symbol
    pub fn get_unary(&self, symbol: &str) -> Option<Arc<dyn UnifiedFhirPathOperator>> {
        self.unary_operators.get(symbol).cloned()
    }

    /// Evaluate a binary operator
    pub async fn evaluate_binary(
        &self,
        symbol: &str,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let operator = self.get_binary(symbol)
            .ok_or_else(|| OperatorRegistryError::OperatorNotFound { 
                symbol: symbol.to_string() 
            })?;
        
        operator.evaluate_binary(left, right, context)
            .await
            .map_err(|e| e.into())
    }

    /// Evaluate a unary operator
    pub async fn evaluate_unary(
        &self,
        symbol: &str,
        operand: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let operator = self.get_unary(symbol)
            .ok_or_else(|| OperatorRegistryError::OperatorNotFound { 
                symbol: symbol.to_string() 
            })?;
        
        operator.evaluate_unary(operand, context)
            .await
            .map_err(|e| e.into())
    }

    /// Get operator precedence
    pub fn get_precedence(&self, symbol: &str) -> Option<u8> {
        self.precedence.get(symbol).copied()
    }

    /// Get operator associativity
    pub fn get_associativity(&self, symbol: &str) -> Option<Associativity> {
        self.associativity.get(symbol).copied()
    }

    /// Check if a binary operator exists
    pub fn contains_binary(&self, symbol: &str) -> bool {
        self.binary_operators.contains_key(symbol)
    }

    /// Check if a unary operator exists
    pub fn contains_unary(&self, symbol: &str) -> bool {
        self.unary_operators.contains_key(symbol)
    }

    /// Get all binary operator symbols
    pub fn binary_operator_symbols(&self) -> Vec<&str> {
        self.binary_operators.keys().map(|s| s.as_str()).collect()
    }

    /// Get all unary operator symbols
    pub fn unary_operator_symbols(&self) -> Vec<&str> {
        self.unary_operators.keys().map(|s| s.as_str()).collect()
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> &OperatorRegistryStats {
        &self.stats
    }

    /// Get LSP provider for completions and hover support
    pub fn lsp_provider(&self) -> &OperatorLspProvider {
        &self.lsp_provider
    }

    /// Get LSP provider (mutable) for dynamic updates
    pub fn lsp_provider_mut(&mut self) -> &mut OperatorLspProvider {
        &mut self.lsp_provider
    }

    /// Get completions for operators
    pub fn get_completions(&self, context: &CompletionContext) -> Vec<crate::operator_lsp::OperatorCompletionItem> {
        self.lsp_provider.get_completions(context)
    }

    /// Get hover information for an operator
    pub fn get_hover_info(&self, symbol: &str) -> Option<&crate::operator_lsp::OperatorHoverInfo> {
        self.lsp_provider.get_hover_info(symbol)
    }

    /// Search operators by query
    pub fn search_operators(&self, query: &str) -> Vec<&EnhancedOperatorMetadata> {
        self.lsp_provider.search_operators(query)
    }

    /// Get operators by category
    pub fn get_operators_by_category(&self, category: OperatorCategory) -> Vec<&EnhancedOperatorMetadata> {
        self.lsp_provider.get_operators_by_category(category)
    }

    /// Check if an operator can be optimized
    pub fn is_optimizable(&self, symbol: &str) -> bool {
        self.get_binary(symbol)
            .or_else(|| self.get_unary(symbol))
            .map(|op| op.is_optimizable())
            .unwrap_or(false)
    }

    /// Check if an operator can short-circuit
    pub fn can_short_circuit(&self, symbol: &str) -> bool {
        self.get_binary(symbol)
            .or_else(|| self.get_unary(symbol))
            .map(|op| op.can_short_circuit())
            .unwrap_or(false)
    }

    /// Check if an operator is commutative
    pub fn is_commutative(&self, symbol: &str) -> bool {
        self.get_binary(symbol)
            .or_else(|| self.get_unary(symbol))
            .map(|op| op.is_commutative())
            .unwrap_or(false)
    }

    /// Update statistics after adding an operator
    fn update_stats(&mut self, metadata: &EnhancedOperatorMetadata) {
        self.stats.total_operators += 1;
        
        // Update category count
        *self.stats.operators_by_category
            .entry(metadata.basic.category)
            .or_insert(0) += 1;
        
        // Update feature counts
        if metadata.basic.is_commutative {
            self.stats.commutative_operators += 1;
        }
        
        if metadata.performance.optimizable {
            self.stats.optimizable_operators += 1;
        }
    }

    /// Validate operator configuration
    pub fn validate_operator(&self, metadata: &EnhancedOperatorMetadata) -> RegistryResult<()> {
        // Check that binary operators have appropriate type signatures
        if metadata.basic.supports_binary {
            let has_binary_signatures = metadata.types.type_signatures.iter()
                .any(|sig| sig.left_type.is_some());
            
            if !has_binary_signatures {
                return Err(OperatorRegistryError::InvalidConfiguration {
                    message: format!(
                        "Binary operator '{}' must have at least one binary type signature",
                        metadata.basic.symbol
                    ),
                });
            }
        }
        
        // Check that unary operators have appropriate type signatures
        if metadata.basic.supports_unary {
            let has_unary_signatures = metadata.types.type_signatures.iter()
                .any(|sig| sig.left_type.is_none());
            
            if !has_unary_signatures {
                return Err(OperatorRegistryError::InvalidConfiguration {
                    message: format!(
                        "Unary operator '{}' must have at least one unary type signature",
                        metadata.basic.symbol
                    ),
                });
            }
        }
        
        // Check that operator supports at least one operation type
        if !metadata.basic.supports_binary && !metadata.basic.supports_unary {
            return Err(OperatorRegistryError::InvalidConfiguration {
                message: format!(
                    "Operator '{}' must support either binary or unary operations",
                    metadata.basic.symbol
                ),
            });
        }
        
        Ok(())
    }

    /// Clear all registered operators (useful for testing)
    pub fn clear(&mut self) {
        self.binary_operators.clear();
        self.unary_operators.clear();
        self.precedence.clear();
        self.associativity.clear();
        self.lsp_provider = OperatorLspProvider::new();
        self.stats = OperatorRegistryStats {
            total_operators: 0,
            binary_operators: 0,
            unary_operators: 0,
            operators_by_category: FxHashMap::default(),
            commutative_operators: 0,
            optimizable_operators: 0,
        };
    }
}

impl Default for UnifiedOperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a unified operator registry with all built-in operators
pub fn create_unified_operator_registry() -> UnifiedOperatorRegistry {
    let mut registry = UnifiedOperatorRegistry::new();
    
    // Register arithmetic operators
    register_arithmetic_operators(&mut registry);
    
    // Register comparison operators  
    register_comparison_operators(&mut registry);
    
    // Register logical operators
    register_logical_operators(&mut registry);
    
    // Register type checking operators
    register_type_checking_operators(&mut registry);
    
    // Register collection operators
    register_collection_operators(&mut registry);
    
    // Register string operators
    register_string_operators(&mut registry);
    
    registry
}

/// Register unified arithmetic operators
fn register_arithmetic_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::arithmetic::*;
    
    let _ = registry.register(UnifiedAdditionOperator::new());
    let _ = registry.register(UnifiedSubtractionOperator::new());
    let _ = registry.register(UnifiedMultiplicationOperator::new());
    let _ = registry.register(UnifiedDivisionOperator::new());
    let _ = registry.register(UnifiedDivOperator::new());
    let _ = registry.register(UnifiedModOperator::new());
}

/// Register unified comparison operators
fn register_comparison_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::comparison::*;
    
    let _ = registry.register(UnifiedEqualsOperator::new());
    let _ = registry.register(UnifiedNotEqualsOperator::new());
    let _ = registry.register(UnifiedLessThanOperator::new());
    let _ = registry.register(UnifiedGreaterThanOperator::new());
    let _ = registry.register(UnifiedLessThanOrEqualOperator::new());
    let _ = registry.register(UnifiedGreaterThanOrEqualOperator::new());
    let _ = registry.register(UnifiedEquivalentOperator::new());
    let _ = registry.register(UnifiedNotEquivalentOperator::new());
}

/// Register unified logical operators
fn register_logical_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::logical::*;
    
    let _ = registry.register(UnifiedAndOperator::new());
    let _ = registry.register(UnifiedOrOperator::new());
    let _ = registry.register(UnifiedNotOperator::new());
    let _ = registry.register(UnifiedXorOperator::new());
    let _ = registry.register(UnifiedImpliesOperator::new());
}

/// Register unified type checking operators
fn register_type_checking_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::type_checking::*;
    
    let _ = registry.register(UnifiedIsOperator::new());
    let _ = registry.register(UnifiedAsOperator::new());
}

/// Register unified collection operators
fn register_collection_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::collection::*;
    
    let _ = registry.register(UnifiedUnionOperator::new());
    let _ = registry.register(UnifiedInOperator::new());
    let _ = registry.register(UnifiedContainsOperator::new());
}

/// Register unified string operators
fn register_string_operators(registry: &mut UnifiedOperatorRegistry) {
    use crate::unified_operators::string::*;
    
    let _ = registry.register(UnifiedConcatenationOperator::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_operators::arithmetic::UnifiedAdditionOperator;
    use crate::unified_operators::comparison::UnifiedEqualsOperator;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_registry_registration() {
        let mut registry = UnifiedOperatorRegistry::new();
        
        let addition = UnifiedAdditionOperator::new();
        let result = registry.register(addition);
        
        assert!(result.is_ok());
        assert_eq!(registry.get_stats().total_operators, 1);
        assert_eq!(registry.get_stats().binary_operators, 1);
        assert!(registry.contains_binary("+"));
    }

    #[tokio::test]
    async fn test_binary_evaluation() {
        let mut registry = UnifiedOperatorRegistry::new();
        let _ = registry.register(UnifiedAdditionOperator::new());
        
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = registry.evaluate_binary(
            "+",
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
            &context,
        ).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(5));
    }

    #[tokio::test]
    async fn test_unary_evaluation() {
        let mut registry = UnifiedOperatorRegistry::new();
        let _ = registry.register(crate::unified_operators::arithmetic::UnifiedSubtractionOperator::new());
        
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = registry.evaluate_unary(
            "-",
            FhirPathValue::Integer(5),
            &context,
        ).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(-5));
    }

    #[test]
    fn test_lsp_integration() {
        let mut registry = UnifiedOperatorRegistry::new();
        let _ = registry.register(UnifiedAdditionOperator::new());
        
        let context = CompletionContext {
            query: Some("add".to_string()),
            ..Default::default()
        };
        
        let completions = registry.get_completions(&context);
        assert!(!completions.is_empty());
        assert_eq!(completions[0].symbol, "+");
        
        let hover = registry.get_hover_info("+");
        assert!(hover.is_some());
        assert_eq!(hover.unwrap().symbol, "+");
    }

    #[test]
    fn test_operator_properties() {
        let mut registry = UnifiedOperatorRegistry::new();
        let _ = registry.register(UnifiedAdditionOperator::new());
        
        assert_eq!(registry.get_precedence("+"), Some(12));
        assert_eq!(registry.get_associativity("+"), Some(Associativity::Left));
        assert!(registry.is_commutative("+"));
        assert!(registry.is_optimizable("+"));
    }

    #[test]
    fn test_statistics() {
        let registry = create_unified_operator_registry();
        let stats = registry.get_stats();
        
        // Should have arithmetic and comparison operators
        assert!(stats.total_operators >= 6); // 4 arithmetic + 4 comparison
        assert!(stats.binary_operators >= 6);
        assert!(stats.operators_by_category.contains_key(&OperatorCategory::Arithmetic));
        assert!(stats.operators_by_category.contains_key(&OperatorCategory::Comparison));
    }

    #[test]
    fn test_search_functionality() {
        let registry = create_unified_operator_registry();
        
        let arithmetic_results = registry.search_operators("arithmetic");
        assert!(!arithmetic_results.is_empty());
        
        let addition_results = registry.search_operators("addition");
        assert!(!addition_results.is_empty());
        
        let category_results = registry.get_operators_by_category(OperatorCategory::Arithmetic);
        assert!(!category_results.is_empty());
    }
}