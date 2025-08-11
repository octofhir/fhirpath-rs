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

//! Pre-compiled function signatures for faster dispatch
//!
//! This module provides a pre-compilation system for function signatures that eliminates
//! runtime type checking and enables faster function dispatch through generated code.

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{FunctionError, FunctionResult};
use crate::registry::signature::FunctionSignature;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Pre-compiled function signature optimized for fast dispatch
#[derive(Clone)]
pub struct CompiledSignature {
    /// Original function signature
    pub signature: FunctionSignature,
    /// Pre-compiled type validation function
    pub type_validator: Arc<dyn Fn(&[FhirPathValue]) -> bool + Send + Sync>,
    /// Pre-compiled arity validator
    pub arity_validator: Arc<dyn Fn(usize) -> bool + Send + Sync>,
    /// Fast dispatch key based on common type patterns
    pub dispatch_key: u64,
    /// Whether this signature matches any type (fallback)
    pub matches_any: bool,
}

impl std::fmt::Debug for CompiledSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledSignature")
            .field("signature", &self.signature)
            .field("dispatch_key", &self.dispatch_key)
            .field("matches_any", &self.matches_any)
            .finish()
    }
}

impl CompiledSignature {
    /// Create a new compiled signature from a function signature
    pub fn new(signature: FunctionSignature) -> Self {
        let min_arity = signature.min_arity;
        let max_arity = signature.max_arity;
        let params = signature.parameters.clone();

        // Generate fast arity validator
        let arity_validator = Arc::new(move |arg_count: usize| -> bool {
            if arg_count < min_arity {
                return false;
            }
            if let Some(max) = max_arity {
                if arg_count > max {
                    return false;
                }
            }
            true
        });

        // Generate fast type validator (strict type checking for compiled signatures)
        let type_validator = Arc::new(move |args: &[FhirPathValue]| -> bool {
            for (i, arg) in args.iter().enumerate() {
                if let Some(param) = params.get(i) {
                    if param.param_type != TypeInfo::Any {
                        let arg_type = arg.to_type_info();
                        // Use strict type matching for compiled signatures (no coercion)
                        if param.param_type != arg_type
                            && !matches!(param.param_type, TypeInfo::Any)
                        {
                            return false;
                        }
                    }
                }
            }
            true
        });

        // Generate dispatch key for common patterns
        let dispatch_key = Self::generate_dispatch_key(&signature);

        // Check if this signature accepts any types
        let matches_any = signature
            .parameters
            .iter()
            .all(|p| p.param_type == TypeInfo::Any);

        Self {
            signature,
            type_validator,
            arity_validator,
            dispatch_key,
            matches_any,
        }
    }

    /// Generate a fast dispatch key based on signature patterns
    fn generate_dispatch_key(signature: &FunctionSignature) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash based on parameter types and arity
        signature.min_arity.hash(&mut hasher);
        signature.max_arity.hash(&mut hasher);

        for param in &signature.parameters {
            param.param_type.hash(&mut hasher);
            param.optional.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Fast validation that combines arity and type checking
    pub fn validates_fast(&self, args: &[FhirPathValue]) -> bool {
        (self.arity_validator)(args.len()) && (self.type_validator)(args)
    }

    /// Validate arguments with detailed error reporting
    pub fn validate_with_errors(
        &self,
        args: &[FhirPathValue],
        function_name: &str,
    ) -> FunctionResult<()> {
        let arg_count = args.len();

        // Check arity first
        if !self.arity_validator.as_ref()(arg_count) {
            return Err(FunctionError::InvalidArity {
                name: function_name.to_string(),
                min: self.signature.min_arity,
                max: self.signature.max_arity,
                actual: arg_count,
            });
        }

        // Check types with detailed error messages
        for (i, arg) in args.iter().enumerate() {
            if let Some(param) = self.signature.parameters.get(i) {
                if param.param_type != TypeInfo::Any {
                    let arg_type = arg.to_type_info();
                    if !param.param_type.is_compatible_with(&arg_type) {
                        return Err(FunctionError::InvalidArgumentType {
                            name: function_name.to_string(),
                            index: i,
                            expected: param.param_type.to_string(),
                            actual: arg_type.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

/// Specialized compiled signatures for common function patterns
#[derive(Debug, Clone)]
pub enum SpecializedSignature {
    /// No arguments (e.g., count(), empty(), exists())
    Nullary,
    /// Single argument of any type (e.g., toString(), length())
    UnaryAny,
    /// Single argument of specific type
    UnaryTyped(TypeInfo),
    /// Two arguments of any type (e.g., contains(), startsWith())
    BinaryAny,
    /// Two arguments of specific types
    BinaryTyped(TypeInfo, TypeInfo),
    /// Variable arguments, all same type
    VariadicSameType(TypeInfo),
    /// Variable arguments of any type
    VariadicAny,
    /// Custom pattern (fallback to compiled signature)
    Custom(CompiledSignature),
}

impl SpecializedSignature {
    /// Create from a function signature, choosing the most efficient specialization
    pub fn from_signature(signature: FunctionSignature) -> Self {
        match signature.parameters.len() {
            0 => Self::Nullary,
            1 => {
                let param = &signature.parameters[0];
                if param.param_type == TypeInfo::Any {
                    Self::UnaryAny
                } else {
                    Self::UnaryTyped(param.param_type.clone())
                }
            }
            2 if signature.max_arity == Some(2) => {
                let param1 = &signature.parameters[0];
                let param2 = &signature.parameters[1];
                if param1.param_type == TypeInfo::Any && param2.param_type == TypeInfo::Any {
                    Self::BinaryAny
                } else {
                    Self::BinaryTyped(param1.param_type.clone(), param2.param_type.clone())
                }
            }
            _ => {
                // Check for variadic patterns
                if signature.max_arity.is_none() {
                    if signature.parameters.is_empty() {
                        Self::VariadicAny
                    } else {
                        // Check if all parameters have the same type
                        let first_type = &signature.parameters[0].param_type;
                        if signature
                            .parameters
                            .iter()
                            .all(|p| &p.param_type == first_type)
                        {
                            if *first_type == TypeInfo::Any {
                                Self::VariadicAny
                            } else {
                                Self::VariadicSameType(first_type.clone())
                            }
                        } else {
                            Self::Custom(CompiledSignature::new(signature))
                        }
                    }
                } else {
                    Self::Custom(CompiledSignature::new(signature))
                }
            }
        }
    }

    /// Fast validation for specialized signatures
    pub fn validates_fast(&self, args: &[FhirPathValue]) -> bool {
        match self {
            Self::Nullary => args.is_empty(),
            Self::UnaryAny => args.len() == 1,
            Self::UnaryTyped(expected_type) => {
                args.len() == 1
                    && (expected_type == &args[0].to_type_info() || *expected_type == TypeInfo::Any)
            }
            Self::BinaryAny => args.len() == 2,
            Self::BinaryTyped(type1, type2) => {
                args.len() == 2
                    && (type1 == &args[0].to_type_info() || *type1 == TypeInfo::Any)
                    && (type2 == &args[1].to_type_info() || *type2 == TypeInfo::Any)
            }
            Self::VariadicAny => true, // Any number of any-type arguments
            Self::VariadicSameType(expected_type) => args
                .iter()
                .all(|arg| expected_type == &arg.to_type_info() || *expected_type == TypeInfo::Any),
            Self::Custom(compiled) => compiled.validates_fast(args),
        }
    }

    /// Get the underlying signature for error reporting
    pub fn get_signature(&self) -> Option<&FunctionSignature> {
        match self {
            Self::Custom(compiled) => Some(&compiled.signature),
            _ => None, // Specialized signatures generate errors differently
        }
    }
}

/// Registry for pre-compiled function signatures with ultra-fast dispatch
#[derive(Debug, Clone)]
pub struct CompiledSignatureRegistry {
    /// Compiled signatures by function name
    compiled_signatures: FxHashMap<String, Vec<CompiledSignature>>,
    /// Specialized signatures for common patterns
    specialized_signatures: FxHashMap<String, SpecializedSignature>,
    /// Fast dispatch table for common type combinations
    dispatch_table: FxHashMap<(String, u64), usize>, // (function_name, type_hash) -> signature_index
}

impl CompiledSignatureRegistry {
    /// Create a new compiled signature registry
    pub fn new() -> Self {
        Self {
            compiled_signatures: FxHashMap::default(),
            specialized_signatures: FxHashMap::default(),
            dispatch_table: FxHashMap::default(),
        }
    }

    /// Register a function signature for compilation
    pub fn register_signature(&mut self, function_name: String, signature: FunctionSignature) {
        // Create specialized signature if possible
        let specialized = SpecializedSignature::from_signature(signature.clone());
        self.specialized_signatures
            .insert(function_name.clone(), specialized);

        // Create compiled signature
        let compiled = CompiledSignature::new(signature);
        let dispatch_key = compiled.dispatch_key;

        let signatures = self
            .compiled_signatures
            .entry(function_name.clone())
            .or_default();
        let index = signatures.len();
        signatures.push(compiled);

        // Register in dispatch table
        self.dispatch_table
            .insert((function_name, dispatch_key), index);
    }

    /// Get the best matching compiled signature for given arguments
    pub fn get_best_signature(
        &self,
        function_name: &str,
        args: &[FhirPathValue],
    ) -> Option<&CompiledSignature> {
        // Try specialized signature first (fastest path)
        if let Some(specialized) = self.specialized_signatures.get(function_name) {
            if specialized.validates_fast(args) {
                // For specialized signatures, return the first compiled signature as fallback
                // In practice, we'd have specialized validators that don't need CompiledSignature
                return self.compiled_signatures.get(function_name)?.first();
            }
        }

        // Try dispatch table lookup
        let arg_types: Vec<TypeInfo> = args.iter().map(|v| v.to_type_info()).collect();
        let type_hash = Self::hash_types(&arg_types);

        if let Some(&index) = self
            .dispatch_table
            .get(&(function_name.to_string(), type_hash))
        {
            if let Some(signatures) = self.compiled_signatures.get(function_name) {
                if let Some(signature) = signatures.get(index) {
                    if signature.validates_fast(args) {
                        return Some(signature);
                    }
                }
            }
        }

        // Fallback: linear search through compiled signatures
        if let Some(signatures) = self.compiled_signatures.get(function_name) {
            signatures.iter().find(|sig| sig.validates_fast(args))
        } else {
            None
        }
    }

    /// Fast validation using specialized or compiled signatures
    pub fn validates_fast(&self, function_name: &str, args: &[FhirPathValue]) -> bool {
        // Try specialized signature first
        if let Some(specialized) = self.specialized_signatures.get(function_name) {
            return specialized.validates_fast(args);
        }

        // Fallback to compiled signatures
        self.get_best_signature(function_name, args).is_some()
    }

    /// Validate with detailed error reporting
    pub fn validate_with_errors(
        &self,
        function_name: &str,
        args: &[FhirPathValue],
    ) -> FunctionResult<()> {
        if let Some(signature) = self.get_best_signature(function_name, args) {
            signature.validate_with_errors(args, function_name)
        } else {
            Err(FunctionError::EvaluationError {
                name: function_name.to_string(),
                message: "No matching signature found".to_string(),
            })
        }
    }

    /// Get compilation statistics
    pub fn compilation_stats(&self) -> CompilationStats {
        let total_functions = self.compiled_signatures.len();
        let specialized_count = self.specialized_signatures.len();
        let dispatch_entries = self.dispatch_table.len();

        let signature_counts: Vec<usize> = self
            .compiled_signatures
            .values()
            .map(|sigs| sigs.len())
            .collect();

        let total_signatures = signature_counts.iter().sum();
        let avg_signatures_per_function = if total_functions > 0 {
            total_signatures as f64 / total_functions as f64
        } else {
            0.0
        };

        CompilationStats {
            total_functions,
            total_signatures,
            specialized_count,
            dispatch_entries,
            avg_signatures_per_function,
        }
    }

    /// Pre-warm the dispatch table with common type combinations
    pub fn warm_dispatch_table(&mut self, common_combinations: Vec<(String, Vec<TypeInfo>)>) {
        for (function_name, arg_types) in common_combinations {
            let type_hash = Self::hash_types(&arg_types);

            // Find best matching signature index
            if let Some(signatures) = self.compiled_signatures.get(&function_name) {
                for (index, signature) in signatures.iter().enumerate() {
                    // Create dummy args for validation (this is for pre-warming only)
                    let dummy_args: Vec<FhirPathValue> = arg_types
                        .iter()
                        .map(Self::create_dummy_value_for_type)
                        .collect();

                    if signature.validates_fast(&dummy_args) {
                        self.dispatch_table
                            .insert((function_name.clone(), type_hash), index);
                        break;
                    }
                }
            }
        }
    }

    /// Hash a list of types for dispatch table lookup
    fn hash_types(types: &[TypeInfo]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for t in types {
            t.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Create a dummy value for a given type (for pre-warming)
    fn create_dummy_value_for_type(type_info: &TypeInfo) -> FhirPathValue {
        match type_info {
            TypeInfo::Integer => FhirPathValue::Integer(0),
            TypeInfo::Decimal => FhirPathValue::Decimal("0.0".parse().unwrap()),
            TypeInfo::String => FhirPathValue::String("".into()),
            TypeInfo::Boolean => FhirPathValue::Boolean(false),
            TypeInfo::Date => FhirPathValue::Date("2000-01-01".parse().unwrap()),
            TypeInfo::DateTime => FhirPathValue::DateTime("2000-01-01T00:00:00Z".parse().unwrap()),
            TypeInfo::Time => FhirPathValue::Time("00:00:00".parse().unwrap()),
            TypeInfo::Quantity => {
                use crate::model::quantity::Quantity;
                use rust_decimal::Decimal;
                FhirPathValue::Quantity(Quantity::new(Decimal::ZERO, Some("1".to_string())).into())
            }
            TypeInfo::Collection(_) => FhirPathValue::collection(vec![]),
            TypeInfo::Resource(_) => FhirPathValue::Empty, // Can't create meaningful dummy resource
            TypeInfo::Union(_) => FhirPathValue::Empty,    // Use empty for union types
            TypeInfo::Optional(_) => FhirPathValue::Empty, // Use empty for optional types
            TypeInfo::SimpleType => FhirPathValue::Empty,
            TypeInfo::ClassType => FhirPathValue::Empty,
            TypeInfo::TypeInfo => FhirPathValue::Empty,
            TypeInfo::Function { .. } => FhirPathValue::Empty, // Can't create dummy functions
            TypeInfo::Tuple(_) => FhirPathValue::Empty,        // Use empty for tuple types
            TypeInfo::Named { .. } => FhirPathValue::Empty,    // Use empty for named types
            TypeInfo::Any => FhirPathValue::Empty,
        }
    }
}

impl Default for CompiledSignatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about signature compilation
#[derive(Debug, Clone)]
pub struct CompilationStats {
    /// Total number of functions with compiled signatures
    pub total_functions: usize,
    /// Total number of compiled signatures
    pub total_signatures: usize,
    /// Number of functions with specialized signatures
    pub specialized_count: usize,
    /// Number of entries in dispatch table
    pub dispatch_entries: usize,
    /// Average signatures per function
    pub avg_signatures_per_function: f64,
}

impl std::fmt::Display for CompilationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Compilation Stats: {} functions, {} signatures ({:.1} avg), {} specialized, {} dispatch entries",
            self.total_functions,
            self.total_signatures,
            self.avg_signatures_per_function,
            self.specialized_count,
            self.dispatch_entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FhirPathValue, TypeInfo};
    use crate::registry::signature::{FunctionSignature, ParameterInfo};

    #[test]
    fn test_compiled_signature_creation() {
        let signature = FunctionSignature::new(
            "test",
            vec![
                ParameterInfo::required("x", TypeInfo::Integer),
                ParameterInfo::optional("y", TypeInfo::String),
            ],
            TypeInfo::Boolean,
        );

        let compiled = CompiledSignature::new(signature);

        // Test valid arguments
        let valid_args = vec![FhirPathValue::Integer(42)];
        assert!(compiled.validates_fast(&valid_args));

        let valid_args2 = vec![
            FhirPathValue::Integer(42),
            FhirPathValue::String("test".into()),
        ];
        assert!(compiled.validates_fast(&valid_args2));

        // Test invalid arguments
        let invalid_args = vec![]; // Too few
        assert!(!compiled.validates_fast(&invalid_args));

        let invalid_args2 = vec![FhirPathValue::String("wrong".into())]; // Wrong type
        assert!(!compiled.validates_fast(&invalid_args2));
    }

    #[test]
    fn test_specialized_signatures() {
        // Nullary function
        let nullary_sig = FunctionSignature::new("count", vec![], TypeInfo::Integer);
        let specialized = SpecializedSignature::from_signature(nullary_sig);
        assert!(matches!(specialized, SpecializedSignature::Nullary));
        assert!(specialized.validates_fast(&[]));
        assert!(!specialized.validates_fast(&[FhirPathValue::Integer(1)]));

        // Unary any function
        let unary_sig = FunctionSignature::new(
            "toString",
            vec![ParameterInfo::required("input", TypeInfo::Any)],
            TypeInfo::String,
        );
        let specialized = SpecializedSignature::from_signature(unary_sig);
        assert!(matches!(specialized, SpecializedSignature::UnaryAny));
        assert!(specialized.validates_fast(&[FhirPathValue::Integer(42)]));
        assert!(specialized.validates_fast(&[FhirPathValue::String("test".into())]));
        assert!(!specialized.validates_fast(&[]));
        assert!(
            !specialized.validates_fast(&[FhirPathValue::Integer(1), FhirPathValue::Integer(2)])
        );

        // Binary typed function
        let binary_sig = FunctionSignature::new(
            "add",
            vec![
                ParameterInfo::required("x", TypeInfo::Integer),
                ParameterInfo::required("y", TypeInfo::Integer),
            ],
            TypeInfo::Integer,
        );
        let specialized = SpecializedSignature::from_signature(binary_sig);
        assert!(matches!(
            specialized,
            SpecializedSignature::BinaryTyped(_, _)
        ));
        assert!(
            specialized.validates_fast(&[FhirPathValue::Integer(1), FhirPathValue::Integer(2)])
        );
        assert!(
            !specialized
                .validates_fast(&[FhirPathValue::String("1".into()), FhirPathValue::Integer(2)])
        );
    }

    #[test]
    fn test_compiled_signature_registry() {
        let mut registry = CompiledSignatureRegistry::new();

        // Register a simple function
        let signature = FunctionSignature::new(
            "double",
            vec![ParameterInfo::required("x", TypeInfo::Integer)],
            TypeInfo::Integer,
        );
        registry.register_signature("double".to_string(), signature);

        // Test validation
        let valid_args = vec![FhirPathValue::Integer(42)];
        assert!(registry.validates_fast("double", &valid_args));
        assert!(registry.validate_with_errors("double", &valid_args).is_ok());

        let invalid_args = vec![FhirPathValue::String("not a number".into())];
        assert!(!registry.validates_fast("double", &invalid_args));
        assert!(
            registry
                .validate_with_errors("double", &invalid_args)
                .is_err()
        );

        // Test signature retrieval
        let best_sig = registry.get_best_signature("double", &valid_args);
        assert!(best_sig.is_some());

        let no_sig = registry.get_best_signature("nonexistent", &valid_args);
        assert!(no_sig.is_none());
    }

    #[test]
    fn test_dispatch_table_warming() {
        let mut registry = CompiledSignatureRegistry::new();

        // Register some functions
        let add_sig = FunctionSignature::new(
            "add",
            vec![
                ParameterInfo::required("x", TypeInfo::Integer),
                ParameterInfo::required("y", TypeInfo::Integer),
            ],
            TypeInfo::Integer,
        );
        registry.register_signature("add".to_string(), add_sig);

        let concat_sig = FunctionSignature::new(
            "concat",
            vec![
                ParameterInfo::required("x", TypeInfo::String),
                ParameterInfo::required("y", TypeInfo::String),
            ],
            TypeInfo::String,
        );
        registry.register_signature("concat".to_string(), concat_sig);

        // Warm dispatch table
        let common_combinations = vec![
            (
                "add".to_string(),
                vec![TypeInfo::Integer, TypeInfo::Integer],
            ),
            (
                "concat".to_string(),
                vec![TypeInfo::String, TypeInfo::String],
            ),
        ];
        registry.warm_dispatch_table(common_combinations);

        // Verify dispatch table entries were created
        let stats = registry.compilation_stats();
        assert!(stats.dispatch_entries >= 2);
    }

    #[test]
    fn test_compilation_stats() {
        let mut registry = CompiledSignatureRegistry::new();

        // Register multiple functions with different numbers of signatures
        let sig1 = FunctionSignature::new("func1", vec![], TypeInfo::Integer);
        registry.register_signature("func1".to_string(), sig1);

        let sig2a = FunctionSignature::new(
            "func2",
            vec![ParameterInfo::required("x", TypeInfo::Integer)],
            TypeInfo::Integer,
        );
        registry.register_signature("func2".to_string(), sig2a);

        let sig2b = FunctionSignature::new(
            "func2",
            vec![ParameterInfo::required("x", TypeInfo::String)],
            TypeInfo::String,
        );
        registry.register_signature("func2".to_string(), sig2b);

        let stats = registry.compilation_stats();
        assert_eq!(stats.total_functions, 2);
        assert_eq!(stats.total_signatures, 3);
        assert_eq!(stats.specialized_count, 2); // Both functions have specialized forms
        assert_eq!(stats.avg_signatures_per_function, 1.5);
    }
}
