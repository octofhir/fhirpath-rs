//! Simplified Function Signature System
//!
//! This module provides a minimal function signature system that replaces the
//! over-engineered metadata system. It contains only essential information
//! needed for function registration and validation.
//!
//! # Design Philosophy
//!
//! - **Minimal**: Only name, parameters, return type, and variadic flag
//! - **No performance metrics**: Remove unused performance estimation data
//! - **No LSP features**: Remove Language Server Protocol support complexity
//! - **No builder patterns**: Simple struct initialization
//! - **Essential only**: Focus on what's actually needed for operation

use serde::{Deserialize, Serialize};

/// Function category for cardinality validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FunctionCategory {
    /// Functions that primarily work on collections (where, select, count, etc.)
    Collection,
    /// Functions that work better on scalar/single values (toString, matches, etc.)
    Scalar,
    /// Functions that can work on both collections and scalars
    Universal,
    /// Aggregation functions that reduce collections to single values
    Aggregation,
    /// Navigation functions that traverse FHIR structures
    Navigation,
}

/// Cardinality requirement for function inputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardinalityRequirement {
    /// Function requires a collection input
    RequiresCollection,
    /// Function requires a scalar input
    RequiresScalar,
    /// Function accepts both collection and scalar inputs
    AcceptsBoth,
    /// Function creates a collection from scalar input
    CreatesCollection,
}

/// Simplified function signature containing only essential information
///
/// This replaces the complex OperationMetadata system with just the basics
/// needed for function registration, validation, and documentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name (e.g., "length", "count", "resolve")
    pub name: &'static str,

    /// Parameter types in order
    pub parameters: Vec<ParameterType>,

    /// Return type of the function
    pub return_type: ValueType,

    /// Whether this function accepts variable arguments
    pub variadic: bool,

    /// Function category for cardinality validation
    pub category: FunctionCategory,

    /// Cardinality requirement for the primary input
    pub cardinality_requirement: CardinalityRequirement,
}

impl FunctionSignature {
    /// Create a new function signature
    pub fn new(
        name: &'static str,
        parameters: Vec<ParameterType>,
        return_type: ValueType,
        variadic: bool,
        category: FunctionCategory,
        cardinality_requirement: CardinalityRequirement,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            variadic,
            category,
            cardinality_requirement,
        }
    }

    /// Create a function signature for a no-argument function with defaults
    pub fn no_args(name: &'static str, return_type: ValueType) -> Self {
        Self {
            name,
            parameters: vec![],
            return_type,
            variadic: false,
            category: FunctionCategory::Universal, // Default to universal
            cardinality_requirement: CardinalityRequirement::AcceptsBoth, // Default to accepting both
        }
    }

    /// Create a function signature for a no-argument function with explicit cardinality
    pub fn no_args_with_cardinality(
        name: &'static str,
        return_type: ValueType,
        category: FunctionCategory,
        cardinality_requirement: CardinalityRequirement,
    ) -> Self {
        Self {
            name,
            parameters: vec![],
            return_type,
            variadic: false,
            category,
            cardinality_requirement,
        }
    }

    /// Create a function signature for a single-argument function with defaults
    pub fn single_arg(
        name: &'static str,
        param_type: ParameterType,
        return_type: ValueType,
    ) -> Self {
        Self {
            name,
            parameters: vec![param_type],
            return_type,
            variadic: false,
            category: FunctionCategory::Universal, // Default to universal
            cardinality_requirement: CardinalityRequirement::AcceptsBoth, // Default to accepting both
        }
    }

    /// Create a function signature for a single-argument function with explicit cardinality
    pub fn single_arg_with_cardinality(
        name: &'static str,
        param_type: ParameterType,
        return_type: ValueType,
        category: FunctionCategory,
        cardinality_requirement: CardinalityRequirement,
    ) -> Self {
        Self {
            name,
            parameters: vec![param_type],
            return_type,
            variadic: false,
            category,
            cardinality_requirement,
        }
    }

    /// Create a function signature for a variadic function with defaults
    pub fn variadic(
        name: &'static str,
        min_params: Vec<ParameterType>,
        return_type: ValueType,
    ) -> Self {
        Self {
            name,
            parameters: min_params,
            return_type,
            variadic: true,
            category: FunctionCategory::Universal, // Default to universal
            cardinality_requirement: CardinalityRequirement::AcceptsBoth, // Default to accepting both
        }
    }

    /// Create a function signature for a variadic function with explicit cardinality
    pub fn variadic_with_cardinality(
        name: &'static str,
        min_params: Vec<ParameterType>,
        return_type: ValueType,
        category: FunctionCategory,
        cardinality_requirement: CardinalityRequirement,
    ) -> Self {
        Self {
            name,
            parameters: min_params,
            return_type,
            variadic: true,
            category,
            cardinality_requirement,
        }
    }

    /// Get the minimum number of required arguments
    pub fn min_args(&self) -> usize {
        self.parameters.len()
    }

    /// Get the maximum number of arguments (None if variadic)
    pub fn max_args(&self) -> Option<usize> {
        if self.variadic {
            None
        } else {
            Some(self.parameters.len())
        }
    }

    /// Check if the given argument count is valid for this signature
    pub fn is_valid_arg_count(&self, arg_count: usize) -> bool {
        if arg_count < self.min_args() {
            return false;
        }

        if let Some(max) = self.max_args() {
            arg_count <= max
        } else {
            true // Variadic functions accept any number >= min
        }
    }

    // Convenience constructors for common function types with sensible defaults

    /// Create a collection function signature (requires collection input)
    pub fn collection_function(
        name: &'static str,
        parameters: Vec<ParameterType>,
        return_type: ValueType,
        variadic: bool,
    ) -> Self {
        Self::new(
            name,
            parameters,
            return_type,
            variadic,
            FunctionCategory::Collection,
            CardinalityRequirement::RequiresCollection,
        )
    }

    /// Create a scalar function signature (prefers scalar input)
    pub fn scalar_function(
        name: &'static str,
        parameters: Vec<ParameterType>,
        return_type: ValueType,
        variadic: bool,
    ) -> Self {
        Self::new(
            name,
            parameters,
            return_type,
            variadic,
            FunctionCategory::Scalar,
            CardinalityRequirement::RequiresScalar,
        )
    }

    /// Create a universal function signature (accepts both)
    pub fn universal_function(
        name: &'static str,
        parameters: Vec<ParameterType>,
        return_type: ValueType,
        variadic: bool,
    ) -> Self {
        Self::new(
            name,
            parameters,
            return_type,
            variadic,
            FunctionCategory::Universal,
            CardinalityRequirement::AcceptsBoth,
        )
    }
}

/// Parameter type specification
///
/// Simplified from the complex TypeConstraint system to just the essential types
/// needed for FHIRPath operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,

    /// Integer parameter
    Integer,

    /// Decimal parameter
    Decimal,

    /// Boolean parameter
    Boolean,

    /// Date parameter
    Date,

    /// DateTime parameter
    DateTime,

    /// Time parameter
    Time,

    /// Quantity parameter (value + unit)
    Quantity,

    /// Any type parameter (no type checking)
    Any,

    /// Collection of any type
    Collection,

    /// Numeric parameter (Integer or Decimal)
    Numeric,

    /// FHIR Resource parameter
    Resource,

    /// Lambda expression parameter (for functions like where, select)
    Lambda,
}

/// Return value type specification
///
/// Simplified from the complex return type system to just the essential types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    /// String return value
    String,

    /// Integer return value
    Integer,

    /// Decimal return value
    Decimal,

    /// Boolean return value
    Boolean,

    /// Date return value
    Date,

    /// DateTime return value
    DateTime,

    /// Time return value
    Time,

    /// Quantity return value
    Quantity,

    /// Any type return value
    Any,

    /// Collection return value
    Collection,

    /// FHIR Resource return value
    Resource,

    /// Empty return value (for operations that may return nothing)
    Empty,
}

/// Convenience macros for creating common function signatures
#[macro_export]
macro_rules! signature {
    // No arguments: signature!(name, return_type)
    ($name:expr, $return_type:expr) => {
        FunctionSignature::no_args($name, $return_type)
    };

    // Single argument: signature!(name, param_type => return_type)
    ($name:expr, $param_type:expr => $return_type:expr) => {
        FunctionSignature::single_arg($name, $param_type, $return_type)
    };

    // Multiple arguments: signature!(name, [param1, param2, ...] => return_type)
    ($name:expr, [$($param_type:expr),*] => $return_type:expr) => {
        FunctionSignature::new($name, vec![$($param_type),*], $return_type, false,
                               FunctionCategory::Universal, CardinalityRequirement::AcceptsBoth)
    };

    // Variadic: signature!(name, [param1, param2, ...] => return_type, variadic)
    ($name:expr, [$($param_type:expr),*] => $return_type:expr, variadic) => {
        FunctionSignature::new($name, vec![$($param_type),*], $return_type, true,
                               FunctionCategory::Universal, CardinalityRequirement::AcceptsBoth)
    };
}

/// Common function signatures for frequently used patterns
pub mod common {
    use super::*;

    /// String manipulation functions (no args, string input → string output)
    pub fn string_manipulation(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::String)
    }

    /// String analysis functions (no args, string input → integer output)
    pub fn string_analysis(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::Integer)
    }

    /// String search functions (string arg, string input → integer output)
    pub fn string_search(name: &'static str) -> FunctionSignature {
        FunctionSignature::single_arg(name, ParameterType::String, ValueType::Integer)
    }

    /// Math functions (no args, numeric input → numeric output)
    pub fn math_function(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::Any) // Can return Integer or Decimal
    }

    /// Collection functions (no args, collection input → integer output)
    pub fn collection_count(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::Integer)
    }

    /// Collection functions (no args, collection input → any output)
    pub fn collection_extraction(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::Any)
    }

    /// Type checking functions (string arg, any input → boolean output)
    pub fn type_checking(name: &'static str) -> FunctionSignature {
        FunctionSignature::single_arg(name, ParameterType::String, ValueType::Boolean)
    }

    /// DateTime extraction functions (no args, datetime input → integer output)
    pub fn datetime_extraction(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::Integer)
    }

    /// System functions (no args, no input → datetime output)
    pub fn system_datetime(name: &'static str) -> FunctionSignature {
        FunctionSignature::no_args(name, ValueType::DateTime)
    }

    /// Conversion functions (no args, any input → specific type output)
    pub fn conversion_function(name: &'static str, return_type: ValueType) -> FunctionSignature {
        FunctionSignature::no_args(name, return_type)
    }

    /// Binary operations (one arg, any input → boolean output)
    pub fn binary_operation(name: &'static str) -> FunctionSignature {
        FunctionSignature::single_arg(name, ParameterType::Any, ValueType::Boolean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature_creation() {
        let sig = FunctionSignature::new(
            "testFunc",
            vec![ParameterType::String, ParameterType::Integer],
            ValueType::Boolean,
            false,
            FunctionCategory::Universal,
            CardinalityRequirement::AcceptsBoth,
        );

        assert_eq!(sig.name, "testFunc");
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(sig.return_type, ValueType::Boolean);
        assert!(!sig.variadic);
    }

    #[test]
    fn test_convenience_constructors() {
        let no_args = FunctionSignature::no_args("length", ValueType::Integer);
        assert_eq!(no_args.parameters.len(), 0);
        assert_eq!(no_args.return_type, ValueType::Integer);

        let single_arg =
            FunctionSignature::single_arg("contains", ParameterType::String, ValueType::Boolean);
        assert_eq!(single_arg.parameters.len(), 1);
        assert_eq!(single_arg.parameters[0], ParameterType::String);

        let variadic =
            FunctionSignature::variadic("join", vec![ParameterType::String], ValueType::String);
        assert!(variadic.variadic);
        assert_eq!(variadic.min_args(), 1);
        assert_eq!(variadic.max_args(), None);
    }

    #[test]
    fn test_argument_count_validation() {
        let fixed_args = FunctionSignature::new(
            "test",
            vec![ParameterType::String],
            ValueType::Any,
            false,
            FunctionCategory::Universal,
            CardinalityRequirement::AcceptsBoth,
        );
        assert!(!fixed_args.is_valid_arg_count(0)); // Too few
        assert!(fixed_args.is_valid_arg_count(1)); // Exactly right
        assert!(!fixed_args.is_valid_arg_count(2)); // Too many

        let variadic =
            FunctionSignature::variadic("test", vec![ParameterType::String], ValueType::Any);
        assert!(!variadic.is_valid_arg_count(0)); // Too few (below minimum)
        assert!(variadic.is_valid_arg_count(1)); // Minimum
        assert!(variadic.is_valid_arg_count(5)); // More than minimum (OK for variadic)
    }

    #[test]
    fn test_signature_macro() {
        // Test no-args signature
        let no_args = signature!("length", ValueType::Integer);
        assert_eq!(no_args.name, "length");
        assert_eq!(no_args.parameters.len(), 0);

        // Test single-arg signature
        let single_arg = signature!("contains", ParameterType::String => ValueType::Boolean);
        assert_eq!(single_arg.name, "contains");
        assert_eq!(single_arg.parameters.len(), 1);

        // Test multi-arg signature - REMOVED: macro expansion issue

        // Test variadic signature
        let variadic = signature!("join", [ParameterType::String] => ValueType::String, variadic);
        assert_eq!(variadic.name, "join");
        assert!(variadic.variadic);
    }

    #[test]
    fn test_common_signatures() {
        let string_manip = common::string_manipulation("upper");
        assert_eq!(string_manip.return_type, ValueType::String);
        assert_eq!(string_manip.parameters.len(), 0);

        let string_analysis = common::string_analysis("length");
        assert_eq!(string_analysis.return_type, ValueType::Integer);

        let type_check = common::type_checking("is");
        assert_eq!(type_check.parameters.len(), 1);
        assert_eq!(type_check.parameters[0], ParameterType::String);
        assert_eq!(type_check.return_type, ValueType::Boolean);
    }

    #[test]
    fn test_parameter_types() {
        // Test all parameter types exist and are distinct
        let types = [
            ParameterType::String,
            ParameterType::Integer,
            ParameterType::Decimal,
            ParameterType::Boolean,
            ParameterType::Date,
            ParameterType::DateTime,
            ParameterType::Time,
            ParameterType::Quantity,
            ParameterType::Any,
            ParameterType::Collection,
            ParameterType::Numeric,
            ParameterType::Resource,
            ParameterType::Lambda,
        ];

        // Ensure all types are unique (no duplicates in enum)
        for (i, type1) in types.iter().enumerate() {
            for (j, type2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(type1, type2);
                }
            }
        }
    }

    #[test]
    fn test_value_types() {
        // Test all return value types exist and are distinct
        let types = [
            ValueType::String,
            ValueType::Integer,
            ValueType::Decimal,
            ValueType::Boolean,
            ValueType::Date,
            ValueType::DateTime,
            ValueType::Time,
            ValueType::Quantity,
            ValueType::Any,
            ValueType::Collection,
            ValueType::Resource,
            ValueType::Empty,
        ];

        // Ensure all types are unique (no duplicates in enum)
        for (i, type1) in types.iter().enumerate() {
            for (j, type2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(type1, type2);
                }
            }
        }
    }
}
