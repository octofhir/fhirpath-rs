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

//! Unified metadata system for FHIRPath operations
//!
//! This module provides a comprehensive metadata system that describes
//! both functions and operators with unified type information and performance
//! characteristics.

use octofhir_fhirpath_model::FhirPathValue;
use serde::{Deserialize, Serialize};

/// Unified metadata for operations (functions and operators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Basic operation information
    pub basic: BasicOperationInfo,

    /// Type constraints and signatures
    pub types: TypeConstraints,

    /// Performance characteristics
    pub performance: PerformanceMetadata,

    /// Operation-specific metadata
    pub specific: OperationSpecificMetadata,
}

/// Basic information about an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicOperationInfo {
    /// Operation name or symbol
    pub name: String,

    /// Type of operation
    pub operation_type: OperationType,

    /// Human-readable description
    pub description: String,

    /// Usage examples
    pub examples: Vec<String>,
}

/// Type of FHIRPath operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationType {
    /// Function call (e.g., "count()", "length()")
    Function,

    /// Binary operator (e.g., "+", "=", "and")
    BinaryOperator {
        /// Operator precedence (higher = evaluated first)
        precedence: u8,
        /// Associativity for same-precedence operators
        associativity: Associativity,
    },

    /// Unary operator (e.g., "-", "not")
    UnaryOperator,
}

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Associativity {
    /// Left-to-right evaluation (a + b + c = (a + b) + c)
    Left,
    /// Right-to-left evaluation (a = b = c = a = (b = c))
    Right,
}

/// Type constraints for operation parameters and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConstraints {
    /// Input parameter constraints
    pub parameters: Vec<ParameterConstraint>,

    /// Return type constraint
    pub return_type: TypeConstraint,

    /// Whether this operation supports variadic arguments
    pub variadic: bool,
}

impl Default for TypeConstraints {
    fn default() -> Self {
        Self {
            parameters: Vec::new(),
            return_type: TypeConstraint::Any,
            variadic: false,
        }
    }
}

/// Constraint for a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraint {
    /// Parameter name (for documentation)
    pub name: String,

    /// Type constraint
    pub constraint: TypeConstraint,

    /// Whether this parameter is optional
    pub optional: bool,

    /// Default value if optional
    pub default_value: Option<FhirPathValue>,
}

/// Type constraint specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeConstraint {
    /// Any type is acceptable
    Any,

    /// Specific FHIRPath type
    Specific(FhirPathType),

    /// One of several possible types
    OneOf(Vec<FhirPathType>),

    /// Collection of specific types
    Collection(Box<TypeConstraint>),

    /// Numeric types (Integer or Decimal)
    Numeric,

    /// Comparable types
    Comparable,
}

/// FHIRPath type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FhirPathType {
    /// Empty collection
    Empty,

    /// Boolean value
    Boolean,

    /// Integer number
    Integer,

    /// Decimal number
    Decimal,

    /// String value
    String,

    /// Date value
    Date,

    /// DateTime value
    DateTime,

    /// Time value
    Time,

    /// Quantity with unit
    Quantity,

    /// FHIR resource
    Resource,

    /// Generic collection
    Collection,

    /// Any type
    Any,
}

/// Performance metadata for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetadata {
    /// Computational complexity
    pub complexity: PerformanceComplexity,

    /// Whether operation supports synchronous evaluation
    pub supports_sync: bool,

    /// Estimated average execution time in nanoseconds
    pub avg_time_ns: u64,

    /// Estimated memory usage in bytes
    pub memory_usage: u64,
}

/// Performance complexity classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceComplexity {
    /// O(1) - constant time
    Constant,

    /// O(log n) - logarithmic time
    Logarithmic,

    /// O(n) - linear time
    Linear,

    /// O(n log n) - linearithmic time
    Linearithmic,

    /// O(n²) - quadratic time
    Quadratic,

    /// O(2^n) - exponential time
    Exponential,
}

/// Operation-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationSpecificMetadata {
    /// Function-specific metadata
    Function(FunctionMetadata),

    /// Operator-specific metadata
    Operator(OperatorMetadata),
}

/// Metadata specific to functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Whether this function supports lambda expressions
    pub supports_lambda: bool,

    /// Lambda parameter indices (which parameters are lambda expressions)
    pub lambda_parameters: Vec<usize>,

    /// Whether this function is deterministic (same input = same output)
    pub deterministic: bool,

    /// Whether this function has side effects
    pub side_effects: bool,
}

impl Default for FunctionMetadata {
    fn default() -> Self {
        Self {
            supports_lambda: false,
            lambda_parameters: Vec::new(),
            deterministic: true,
            side_effects: false,
        }
    }
}

/// Metadata specific to operators
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperatorMetadata {
    /// Whether this operator is commutative (a op b = b op a)
    pub commutative: bool,

    /// Whether this operator is associative ((a op b) op c = a op (b op c))
    pub associative: bool,

    /// Identity element for this operator (if any)
    pub identity: Option<FhirPathValue>,

    /// Whether this operator short-circuits evaluation
    pub short_circuit: bool,
}

/// Builder for operation metadata
pub struct MetadataBuilder {
    metadata: OperationMetadata,
}

impl MetadataBuilder {
    /// Create a new metadata builder
    pub fn new(name: &str, operation_type: OperationType) -> Self {
        Self {
            metadata: OperationMetadata {
                basic: BasicOperationInfo {
                    name: name.to_string(),
                    operation_type: operation_type.clone(),
                    description: String::new(),
                    examples: Vec::new(),
                },
                types: TypeConstraints::default(),
                performance: PerformanceMetadata {
                    complexity: PerformanceComplexity::Linear,
                    supports_sync: true,
                    avg_time_ns: 1000,
                    memory_usage: 64,
                },
                specific: match operation_type {
                    OperationType::Function => {
                        OperationSpecificMetadata::Function(FunctionMetadata::default())
                    }
                    _ => OperationSpecificMetadata::Operator(OperatorMetadata::default()),
                },
            },
        }
    }

    /// Set description
    pub fn description(mut self, description: &str) -> Self {
        self.metadata.basic.description = description.to_string();
        self
    }

    /// Add example
    pub fn example(mut self, example: &str) -> Self {
        self.metadata.basic.examples.push(example.to_string());
        self
    }

    /// Set return type constraint
    pub fn returns(mut self, constraint: TypeConstraint) -> Self {
        self.metadata.types.return_type = constraint;
        self
    }

    /// Add parameter constraint
    pub fn parameter(mut self, name: &str, constraint: TypeConstraint, optional: bool) -> Self {
        self.metadata.types.parameters.push(ParameterConstraint {
            name: name.to_string(),
            constraint,
            optional,
            default_value: None,
        });
        self
    }

    /// Set performance characteristics
    pub fn performance(mut self, complexity: PerformanceComplexity, supports_sync: bool) -> Self {
        self.metadata.performance.complexity = complexity;
        self.metadata.performance.supports_sync = supports_sync;
        self
    }

    /// Mark function as supporting lambda expressions
    pub fn supports_lambda(mut self, lambda_parameters: Vec<usize>) -> Self {
        if let OperationSpecificMetadata::Function(ref mut func_meta) = self.metadata.specific {
            func_meta.supports_lambda = true;
            func_meta.lambda_parameters = lambda_parameters;
        }
        self
    }

    /// Set operator properties
    pub fn operator_properties(mut self, commutative: bool, associative: bool) -> Self {
        if let OperationSpecificMetadata::Operator(ref mut op_meta) = self.metadata.specific {
            op_meta.commutative = commutative;
            op_meta.associative = associative;
        }
        self
    }

    /// Build the metadata
    pub fn build(self) -> OperationMetadata {
        self.metadata
    }
}

impl Default for PerformanceMetadata {
    fn default() -> Self {
        Self {
            complexity: PerformanceComplexity::Linear,
            supports_sync: true,
            avg_time_ns: 1000,
            memory_usage: 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_builder() {
        let metadata = MetadataBuilder::new("count", OperationType::Function)
            .description("Returns the number of items in a collection")
            .example("Patient.name.count()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build();

        assert_eq!(metadata.basic.name, "count");
        assert_eq!(
            metadata.basic.description,
            "Returns the number of items in a collection"
        );
        assert_eq!(metadata.basic.examples.len(), 1);
        assert!(metadata.performance.supports_sync);
    }

    #[test]
    fn test_type_constraints() {
        let constraint = TypeConstraint::OneOf(vec![FhirPathType::Integer, FhirPathType::Decimal]);

        match constraint {
            TypeConstraint::OneOf(types) => {
                assert_eq!(types.len(), 2);
                assert!(types.contains(&FhirPathType::Integer));
                assert!(types.contains(&FhirPathType::Decimal));
            }
            _ => panic!("Expected OneOf constraint"),
        }
    }

    #[test]
    fn test_operation_types() {
        let function = OperationType::Function;
        let binary_op = OperationType::BinaryOperator {
            precedence: 5,
            associativity: Associativity::Left,
        };

        assert_eq!(function, OperationType::Function);

        if let OperationType::BinaryOperator {
            precedence,
            associativity,
        } = binary_op
        {
            assert_eq!(precedence, 5);
            assert_eq!(associativity, Associativity::Left);
        }
    }
}
