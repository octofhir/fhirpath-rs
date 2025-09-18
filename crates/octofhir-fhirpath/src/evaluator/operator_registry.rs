//! Operator registry for FHIRPath binary and unary operations
//!
//! This module implements the operator registry with metadata and signature information.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ast::{BinaryOperator, UnaryOperator};
use crate::core::{FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Metadata for an operator describing its behavior and signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorMetadata {
    /// The operator name (e.g., "+", "=", "and")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Expected parameter types for signature checking
    pub signature: OperatorSignature,
    /// Whether this operator propagates empty values
    pub empty_propagation: EmptyPropagation,
    /// Whether this operator is deterministic
    pub deterministic: bool,
    /// Operator precedence for parsing
    pub precedence: u8,
    /// Associativity (Left, Right, or None)
    pub associativity: Associativity,
}

/// Signature information for operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorSignature {
    /// Type signature with proper FHIRPath types
    pub signature: TypeSignature,
    /// Alternative signatures for overloaded operators
    pub overloads: Vec<TypeSignature>,
}

/// Empty value propagation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmptyPropagation {
    /// Propagate empty if any operand is empty
    Propagate,
    /// Don't propagate empty (e.g., 'or' operator)
    NoPropagation,
    /// Custom propagation logic (handled by the evaluator)
    Custom,
}

/// Operator associativity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Associativity {
    Left,
    Right,
    None,
}

/// Trait for evaluating operations
#[async_trait]
pub trait OperationEvaluator: Send + Sync {
    /// Evaluate the operation
    /// - input: The input collection (for context)
    /// - context: Evaluation context with variables and providers
    /// - left: Left operand values (or operand for unary operations)
    /// - right: Right operand values (empty for unary operations)
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult>;

    /// Get metadata for this operation
    fn metadata(&self) -> &OperatorMetadata;

    /// Check if the operation can handle the given types
    fn can_handle(&self, argument_types: &[FhirPathType]) -> bool {
        let metadata = self.metadata();

        // Check primary signature
        if metadata.signature.signature.matches(argument_types) {
            return true;
        }

        // Check overloaded signatures
        metadata
            .signature
            .overloads
            .iter()
            .any(|sig| sig.matches(argument_types))
    }
}

/// Registry for operation evaluators
pub struct OperatorRegistry {
    /// Binary operator evaluators
    binary_operators: HashMap<BinaryOperator, Arc<dyn OperationEvaluator>>,
    /// Unary operator evaluators
    unary_operators: HashMap<UnaryOperator, Arc<dyn OperationEvaluator>>,
    /// Metadata cache for introspection
    metadata_cache: HashMap<String, OperatorMetadata>,
}

impl OperatorRegistry {
    /// Create a new empty operator registry
    pub fn new() -> Self {
        Self {
            binary_operators: HashMap::new(),
            unary_operators: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Register a binary operator evaluator
    pub fn register_binary_operator(
        &mut self,
        operator: BinaryOperator,
        evaluator: Arc<dyn OperationEvaluator>,
    ) {
        let metadata = evaluator.metadata().clone();
        self.metadata_cache.insert(metadata.name.clone(), metadata);
        self.binary_operators.insert(operator, evaluator);
    }

    /// Register a unary operator evaluator
    pub fn register_unary_operator(
        &mut self,
        operator: UnaryOperator,
        evaluator: Arc<dyn OperationEvaluator>,
    ) {
        let metadata = evaluator.metadata().clone();
        self.metadata_cache.insert(metadata.name.clone(), metadata);
        self.unary_operators.insert(operator, evaluator);
    }

    /// Get binary operator evaluator
    pub fn get_binary_operator(
        &self,
        operator: &BinaryOperator,
    ) -> Option<&Arc<dyn OperationEvaluator>> {
        self.binary_operators.get(operator)
    }

    /// Get unary operator evaluator
    pub fn get_unary_operator(
        &self,
        operator: &UnaryOperator,
    ) -> Option<&Arc<dyn OperationEvaluator>> {
        self.unary_operators.get(operator)
    }

    /// Get operator metadata by name
    pub fn get_metadata(&self, operator_name: &str) -> Option<&OperatorMetadata> {
        self.metadata_cache.get(operator_name)
    }

    /// Get all registered binary operators
    pub fn list_binary_operators(&self) -> Vec<&BinaryOperator> {
        self.binary_operators.keys().collect()
    }

    /// Get all registered unary operators
    pub fn list_unary_operators(&self) -> Vec<&UnaryOperator> {
        self.unary_operators.keys().collect()
    }

    /// Get all operator metadata for introspection
    pub fn all_metadata(&self) -> &HashMap<String, OperatorMetadata> {
        &self.metadata_cache
    }

    /// Find operators that can handle the given types
    pub fn find_compatible_binary_operators(
        &self,
        argument_types: &[FhirPathType],
    ) -> Vec<&BinaryOperator> {
        self.binary_operators
            .iter()
            .filter(|(_, evaluator)| evaluator.can_handle(argument_types))
            .map(|(op, _)| op)
            .collect()
    }

    /// Find unary operators that can handle the given type
    pub fn find_compatible_unary_operators(
        &self,
        argument_types: &[FhirPathType],
    ) -> Vec<&UnaryOperator> {
        self.unary_operators
            .iter()
            .filter(|(_, evaluator)| evaluator.can_handle(argument_types))
            .map(|(op, _)| op)
            .collect()
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating operator registries with default operators
pub struct OperatorRegistryBuilder {
    registry: OperatorRegistry,
}

impl OperatorRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: OperatorRegistry::new(),
        }
    }

    /// Add default arithmetic operators (+, -, *, /, div, mod)
    pub fn with_arithmetic_operators(mut self) -> Self {
        use crate::ast::BinaryOperator;
        use crate::evaluator::operations::*;

        // Register arithmetic operators
        self.registry
            .register_binary_operator(BinaryOperator::Add, AddOperatorEvaluator::create());
        self.registry.register_binary_operator(
            BinaryOperator::Subtract,
            SubtractOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::Multiply,
            MultiplyOperatorEvaluator::create(),
        );
        self.registry
            .register_binary_operator(BinaryOperator::Divide, DivideOperatorEvaluator::create());
        self.registry
            .register_binary_operator(BinaryOperator::Modulo, ModuloOperatorEvaluator::create());
        self.registry.register_binary_operator(
            BinaryOperator::IntegerDivide,
            IntegerDivideOperatorEvaluator::create(),
        );

        // Register string concatenation operator
        self.registry.register_binary_operator(
            BinaryOperator::Concatenate,
            ConcatenateOperatorEvaluator::create(),
        );

        self
    }

    /// Add default comparison operators (=, !=, <, >, <=, >=, ~, !~)
    pub fn with_comparison_operators(mut self) -> Self {
        use crate::ast::BinaryOperator;
        use crate::evaluator::operations::*;

        // Register comparison operators
        self.registry
            .register_binary_operator(BinaryOperator::Equal, EqualsOperatorEvaluator::create());
        self.registry.register_binary_operator(
            BinaryOperator::NotEqual,
            NotEqualsOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::LessThan,
            LessThanOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::GreaterThan,
            GreaterThanOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::LessThanOrEqual,
            LessEqualOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::GreaterThanOrEqual,
            GreaterEqualOperatorEvaluator::create(),
        );

        // Register equivalence operators
        self.registry.register_binary_operator(
            BinaryOperator::Equivalent,
            EquivalentOperatorEvaluator::create(),
        );
        self.registry.register_binary_operator(
            BinaryOperator::NotEquivalent,
            NotEquivalentOperatorEvaluator::create(),
        );

        self
    }

    /// Add default logical operators (and, or, xor, implies)
    pub fn with_logical_operators(mut self) -> Self {
        use crate::ast::BinaryOperator;
        use crate::evaluator::operations::*;

        // Register logical operators
        self.registry
            .register_binary_operator(BinaryOperator::And, AndOperatorEvaluator::create());
        self.registry
            .register_binary_operator(BinaryOperator::Or, OrOperatorEvaluator::create());
        self.registry
            .register_binary_operator(BinaryOperator::Implies, ImpliesOperatorEvaluator::create());
        self.registry
            .register_binary_operator(BinaryOperator::Xor, XorOperatorEvaluator::create());

        self
    }

    /// Add default collection operators (in, contains, union)
    pub fn with_collection_operators(mut self) -> Self {
        use crate::ast::BinaryOperator;
        use crate::evaluator::operations::*;

        // Register collection operators
        self.registry
            .register_binary_operator(BinaryOperator::Union, UnionOperatorEvaluator::create());

        // Register type operators
        self.registry
            .register_binary_operator(BinaryOperator::Is, IsOperatorEvaluator::create());
        self.registry
            .register_binary_operator(BinaryOperator::As, AsOperatorEvaluator::create());

        // Register membership operators
        self.registry
            .register_binary_operator(BinaryOperator::In, InOperatorEvaluator::create());
        self.registry.register_binary_operator(
            BinaryOperator::Contains,
            ContainsOperatorEvaluator::create(),
        );

        self
    }

    /// Add default unary operators (not, -)
    pub fn with_unary_operators(mut self) -> Self {
        use crate::ast::UnaryOperator;
        use crate::evaluator::operations::*;

        // Register unary operators
        self.registry
            .register_unary_operator(UnaryOperator::Negate, NegateOperatorEvaluator::create());
        // TODO: Add NotOperatorEvaluator once implemented

        self
    }

    /// Register a custom binary operator
    pub fn register_binary_operator(
        mut self,
        operator: BinaryOperator,
        evaluator: Arc<dyn OperationEvaluator>,
    ) -> Self {
        self.registry.register_binary_operator(operator, evaluator);
        self
    }

    /// Register a custom unary operator
    pub fn register_unary_operator(
        mut self,
        operator: UnaryOperator,
        evaluator: Arc<dyn OperationEvaluator>,
    ) -> Self {
        self.registry.register_unary_operator(operator, evaluator);
        self
    }

    /// Build the operator registry
    pub fn build(self) -> OperatorRegistry {
        self.registry
    }
}

impl Default for OperatorRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a standard operator registry with all FHIRPath operators
pub fn create_standard_operator_registry() -> OperatorRegistry {
    OperatorRegistryBuilder::new()
        .with_arithmetic_operators()
        .with_comparison_operators()
        .with_logical_operators()
        .with_collection_operators()
        .with_unary_operators()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_registry_creation() {
        let registry = OperatorRegistry::new();
        assert!(registry.binary_operators.is_empty());
        assert!(registry.unary_operators.is_empty());
        assert!(registry.metadata_cache.is_empty());
    }

    #[test]
    fn test_operator_registry_builder() {
        let registry = OperatorRegistryBuilder::new()
            .with_arithmetic_operators()
            .with_comparison_operators()
            .build();

        // Test that registry was created
        // TODO: Add specific tests when operators are implemented
    }
}
