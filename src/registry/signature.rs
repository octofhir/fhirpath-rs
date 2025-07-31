//! Function and operator signatures for type checking

use crate::model::TypeInfo;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Function signature for overload resolution and type checking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Parameter types
    pub parameters: Vec<ParameterInfo>,
    /// Return type
    pub return_type: TypeInfo,
    /// Minimum number of arguments
    pub min_arity: usize,
    /// Maximum number of arguments (None for variadic)
    pub max_arity: Option<usize>,
}

/// Parameter information for functions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: TypeInfo,
    /// Whether this parameter is optional
    pub optional: bool,
}

/// Operator signature for type checking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperatorSignature {
    /// Operator symbol
    pub symbol: String,
    /// Left operand type
    pub left_type: TypeInfo,
    /// Right operand type (None for unary operators)
    pub right_type: Option<TypeInfo>,
    /// Result type
    pub result_type: TypeInfo,
}

impl FunctionSignature {
    /// Create a new function signature
    pub fn new(
        name: impl Into<String>,
        parameters: Vec<ParameterInfo>,
        return_type: TypeInfo,
    ) -> Self {
        let required_params = parameters.iter().filter(|p| !p.optional).count();
        let max_arity = if parameters.is_empty() {
            Some(0)
        } else {
            Some(parameters.len())
        };

        Self {
            name: name.into(),
            parameters,
            return_type,
            min_arity: required_params,
            max_arity,
        }
    }

    /// Create a variadic function signature
    pub fn variadic(
        name: impl Into<String>,
        parameters: Vec<ParameterInfo>,
        return_type: TypeInfo,
    ) -> Self {
        let required_params = parameters.iter().filter(|p| !p.optional).count();

        Self {
            name: name.into(),
            parameters,
            return_type,
            min_arity: required_params,
            max_arity: None,
        }
    }

    /// Check if this signature matches the given argument types
    pub fn matches(&self, arg_types: &[TypeInfo]) -> bool {
        if arg_types.len() < self.min_arity {
            return false;
        }

        if let Some(max) = self.max_arity {
            if arg_types.len() > max {
                return false;
            }
        }

        // Check parameter types
        for (i, arg_type) in arg_types.iter().enumerate() {
            if let Some(param) = self.parameters.get(i) {
                if param.param_type != TypeInfo::Any && arg_type != &param.param_type {
                    return false;
                }
            } else if self.max_arity.is_some() {
                // Too many arguments for non-variadic function
                return false;
            }
        }

        true
    }
}

impl ParameterInfo {
    /// Create a required parameter
    pub fn required(name: impl Into<String>, param_type: TypeInfo) -> Self {
        Self {
            name: name.into(),
            param_type,
            optional: false,
        }
    }

    /// Create an optional parameter
    pub fn optional(name: impl Into<String>, param_type: TypeInfo) -> Self {
        Self {
            name: name.into(),
            param_type,
            optional: true,
        }
    }
}

impl OperatorSignature {
    /// Create a binary operator signature
    pub fn binary(
        symbol: impl Into<String>,
        left_type: TypeInfo,
        right_type: TypeInfo,
        result_type: TypeInfo,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            left_type,
            right_type: Some(right_type),
            result_type,
        }
    }

    /// Create a unary operator signature
    pub fn unary(symbol: impl Into<String>, operand_type: TypeInfo, result_type: TypeInfo) -> Self {
        Self {
            symbol: symbol.into(),
            left_type: operand_type,
            right_type: None,
            result_type,
        }
    }

    /// Check if this signature matches the given operand types
    pub fn matches(&self, left_type: &TypeInfo, right_type: Option<&TypeInfo>) -> bool {
        if self.left_type != TypeInfo::Any && &self.left_type != left_type {
            return false;
        }

        match (&self.right_type, right_type) {
            (Some(expected), Some(actual)) => *expected == TypeInfo::Any || expected == actual,
            (None, None) => true, // Unary operator
            _ => false,           // Mismatch between binary/unary
        }
    }
}

impl fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", param.name, param.param_type)?;
            if param.optional {
                write!(f, "?")?;
            }
        }
        write!(f, ") -> {}", self.return_type)
    }
}

impl fmt::Display for OperatorSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.right_type {
            Some(right) => write!(
                f,
                "{} {} {} -> {}",
                self.left_type, self.symbol, right, self.result_type
            ),
            None => write!(
                f,
                "{} {} -> {}",
                self.symbol, self.left_type, self.result_type
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature_matching() {
        let sig = FunctionSignature::new(
            "test",
            vec![
                ParameterInfo::required("x", TypeInfo::Integer),
                ParameterInfo::optional("y", TypeInfo::String),
            ],
            TypeInfo::Boolean,
        );

        assert!(sig.matches(&[TypeInfo::Integer]));
        assert!(sig.matches(&[TypeInfo::Integer, TypeInfo::String]));
        assert!(!sig.matches(&[])); // Too few arguments
        assert!(!sig.matches(&[TypeInfo::String])); // Wrong type
    }

    #[test]
    fn test_operator_signature_matching() {
        let sig =
            OperatorSignature::binary("+", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer);

        assert!(sig.matches(&TypeInfo::Integer, Some(&TypeInfo::Integer)));
        assert!(!sig.matches(&TypeInfo::String, Some(&TypeInfo::Integer)));
        assert!(!sig.matches(&TypeInfo::Integer, None)); // Unary when binary expected
    }
}
