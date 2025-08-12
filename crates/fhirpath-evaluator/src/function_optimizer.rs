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

//! Function call optimizer using cached type information
//!
//! This module provides optimization capabilities for function calls by leveraging
//! cached type information from async ModelProvider operations.

use super::context::EvaluationContext;
use fhirpath_core::EvaluationResult;
use fhirpath_model::{provider::ModelProvider, FhirPathValue};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Function call optimizer that uses cached type information
pub struct FunctionOptimizer {
    /// Reference to the async ModelProvider for advanced optimizations
    provider: Arc<dyn ModelProvider>,
    /// Cache of optimized function signatures
    signature_cache: FxHashMap<String, OptimizedSignature>,
}

/// Optimized function signature with pre-resolved type information
#[derive(Debug, Clone)]
pub struct OptimizedSignature {
    /// Function name
    pub name: String,
    /// Parameter types (if known)
    pub parameter_types: Vec<Option<String>>,
    /// Return type (if known)
    pub return_type: Option<String>,
    /// Whether this function can be optimized for specific input types
    pub optimizable: bool,
    /// Fast path dispatch information
    pub dispatch_info: DispatchInfo,
}

/// Fast path dispatch information for optimized functions
#[derive(Debug, Clone)]
pub enum DispatchInfo {
    /// No optimization available
    None,
    /// Collection operation that can be optimized
    CollectionOp {
        /// Type of collection operation
        op_type: CollectionOpType,
        /// Expected element type
        element_type: Option<String>,
    },
    /// Type checking operation
    TypeCheck {
        /// Expected input type
        input_type: String,
        /// Target type for checking
        target_type: String,
    },
    /// Property navigation optimization
    PropertyNav {
        /// Source type
        source_type: String,
        /// Property name
        property: String,
        /// Result type
        result_type: Option<String>,
    },
}

/// Types of collection operations that can be optimized
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionOpType {
    /// where() function
    Where,
    /// select() function  
    Select,
    /// first() function
    First,
    /// last() function
    Last,
    /// count() function
    Count,
    /// exists() function
    Exists,
    /// all() function
    All,
    /// any() function
    Any,
}

impl FunctionOptimizer {
    /// Create a new function optimizer
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            provider,
            signature_cache: FxHashMap::default(),
        }
    }

    /// Optimize a function call using cached type information
    pub async fn optimize_function_call(
        &mut self,
        context: &EvaluationContext,
        function_name: &str,
        input: &FhirPathValue,
        args: &[FhirPathValue],
    ) -> EvaluationResult<Option<FhirPathValue>> {
        // Check if we have cached optimization info for this function
        let cache_key = format!("{}:{}", function_name, args.len());

        let signature = if let Some(cached) = self.signature_cache.get(&cache_key) {
            cached.clone()
        } else {
            // Analyze and cache the function signature
            let sig = self
                .analyze_function_signature(context, function_name, input, args)
                .await?;
            self.signature_cache.insert(cache_key, sig.clone());
            sig
        };

        // Apply optimizations based on the signature
        if signature.optimizable {
            self.apply_fast_path_optimization(context, input, args, &signature)
                .await
        } else {
            // No optimization available
            Ok(None)
        }
    }

    /// Analyze a function signature for optimization opportunities
    async fn analyze_function_signature(
        &self,
        _context: &EvaluationContext,
        function_name: &str,
        input: &FhirPathValue,
        args: &[FhirPathValue],
    ) -> EvaluationResult<OptimizedSignature> {
        let input_type = self.infer_value_type(input);

        let dispatch_info = match function_name {
            // Collection operations
            "where" if args.len() == 1 => DispatchInfo::CollectionOp {
                op_type: CollectionOpType::Where,
                element_type: input_type.clone(),
            },
            "select" if args.len() == 1 => DispatchInfo::CollectionOp {
                op_type: CollectionOpType::Select,
                element_type: input_type.clone(),
            },
            "first" | "last" | "count" | "exists" if args.is_empty() => {
                let op_type = match function_name {
                    "first" => CollectionOpType::First,
                    "last" => CollectionOpType::Last,
                    "count" => CollectionOpType::Count,
                    "exists" => CollectionOpType::Exists,
                    _ => unreachable!(),
                };
                DispatchInfo::CollectionOp {
                    op_type,
                    element_type: input_type.clone(),
                }
            }
            "all" | "any" if args.len() == 1 => {
                let op_type = if function_name == "all" {
                    CollectionOpType::All
                } else {
                    CollectionOpType::Any
                };
                DispatchInfo::CollectionOp {
                    op_type,
                    element_type: input_type.clone(),
                }
            }
            _ => DispatchInfo::None,
        };

        let optimizable = !matches!(dispatch_info, DispatchInfo::None);

        Ok(OptimizedSignature {
            name: function_name.to_string(),
            parameter_types: args.iter().map(|arg| self.infer_value_type(arg)).collect(),
            return_type: self
                .predict_return_type(function_name, &input_type, args)
                .await,
            optimizable,
            dispatch_info,
        })
    }

    /// Apply fast path optimizations
    async fn apply_fast_path_optimization(
        &self,
        _context: &EvaluationContext,
        input: &FhirPathValue,
        _args: &[FhirPathValue],
        signature: &OptimizedSignature,
    ) -> EvaluationResult<Option<FhirPathValue>> {
        match &signature.dispatch_info {
            DispatchInfo::CollectionOp { op_type, .. } => {
                self.optimize_collection_operation(input, op_type).await
            }
            DispatchInfo::TypeCheck { .. } => {
                // Type checking optimizations would be implemented here
                Ok(None)
            }
            DispatchInfo::PropertyNav { .. } => {
                // Property navigation optimizations would be implemented here
                Ok(None)
            }
            DispatchInfo::None => Ok(None),
        }
    }

    /// Optimize collection operations using type information
    async fn optimize_collection_operation(
        &self,
        input: &FhirPathValue,
        op_type: &CollectionOpType,
    ) -> EvaluationResult<Option<FhirPathValue>> {
        match input {
            FhirPathValue::Collection(collection) => {
                match op_type {
                    CollectionOpType::Count => {
                        // Fast path for count() - just return collection size
                        Ok(Some(FhirPathValue::Integer(collection.len() as i64)))
                    }
                    CollectionOpType::Exists => {
                        // Fast path for exists() - check if collection is non-empty
                        Ok(Some(FhirPathValue::Boolean(!collection.is_empty())))
                    }
                    CollectionOpType::First => {
                        // Fast path for first() - return first element or empty
                        if let Some(first) = collection.iter().next() {
                            Ok(Some(first.clone()))
                        } else {
                            Ok(Some(FhirPathValue::Empty))
                        }
                    }
                    CollectionOpType::Last => {
                        // Fast path for last() - return last element or empty
                        if let Some(last) = collection.iter().last() {
                            Ok(Some(last.clone()))
                        } else {
                            Ok(Some(FhirPathValue::Empty))
                        }
                    }
                    _ => {
                        // Other operations need more complex optimization
                        Ok(None)
                    }
                }
            }
            _ => {
                // For non-collections, some operations can still be optimized
                match op_type {
                    CollectionOpType::Count => {
                        // Single value has count of 1 (unless it's empty)
                        let count = if matches!(input, FhirPathValue::Empty) {
                            0
                        } else {
                            1
                        };
                        Ok(Some(FhirPathValue::Integer(count)))
                    }
                    CollectionOpType::Exists => {
                        // Single value exists if it's not empty
                        Ok(Some(FhirPathValue::Boolean(!matches!(
                            input,
                            FhirPathValue::Empty
                        ))))
                    }
                    CollectionOpType::First | CollectionOpType::Last => {
                        // For single values, first and last are the same
                        if matches!(input, FhirPathValue::Empty) {
                            Ok(Some(FhirPathValue::Empty))
                        } else {
                            Ok(Some(input.clone()))
                        }
                    }
                    _ => Ok(None),
                }
            }
        }
    }

    /// Predict the return type of a function call
    async fn predict_return_type(
        &self,
        function_name: &str,
        input_type: &Option<String>,
        _args: &[FhirPathValue],
    ) -> Option<String> {
        match function_name {
            "count" => Some("integer".to_string()),
            "exists" | "all" | "any" => Some("boolean".to_string()),
            "first" | "last" => input_type.clone(), // Same as input element type
            "where" | "select" => input_type.clone(), // Same collection type
            _ => None,
        }
    }

    /// Infer FHIR type from FhirPathValue (similar to TypeChecker but simpler)
    fn infer_value_type(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::Resource(resource) => resource.resource_type().map(|rt| rt.to_string()),
            FhirPathValue::JsonValue(json) => json
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .map(|s| s.to_string()),
            FhirPathValue::String(_) => Some("string".to_string()),
            FhirPathValue::Integer(_) => Some("integer".to_string()),
            FhirPathValue::Decimal(_) => Some("decimal".to_string()),
            FhirPathValue::Boolean(_) => Some("boolean".to_string()),
            FhirPathValue::Date(_) => Some("date".to_string()),
            FhirPathValue::DateTime(_) => Some("dateTime".to_string()),
            FhirPathValue::Time(_) => Some("time".to_string()),
            FhirPathValue::Quantity(_) => Some("Quantity".to_string()),
            FhirPathValue::Collection(coll) => {
                // For collections, try to infer from first element
                if let Some(first) = coll.iter().next() {
                    self.infer_value_type(first)
                        .map(|t| format!("Collection<{t}>"))
                } else {
                    Some("Collection".to_string())
                }
            }
            _ => None,
        }
    }

    /// Clear the optimization cache
    pub fn clear_cache(&mut self) {
        self.signature_cache.clear();
    }

    /// Get cache statistics for monitoring
    pub fn cache_stats(&self) -> CacheStats {
        let total_entries = self.signature_cache.len();
        let optimizable_entries = self
            .signature_cache
            .values()
            .filter(|sig| sig.optimizable)
            .count();

        CacheStats {
            total_entries,
            optimizable_entries,
            hit_rate: 0.0, // Would need to track hits/misses for this
        }
    }
}

/// Cache statistics for monitoring optimizer performance
#[derive(Debug)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Number of entries with optimizations available
    pub optimizable_entries: usize,
    /// Cache hit rate (would need tracking to implement)
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use fhirpath_model::{mock_provider::MockModelProvider, Collection};
    use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
    use tokio;

    #[tokio::test]
    async fn test_collection_count_optimization() {
        let provider = Arc::new(MockModelProvider::empty());
        let mut optimizer = FunctionOptimizer::new(provider);

        // Create a test collection
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let collection = FhirPathValue::Collection(Collection::from_vec(values));

        // Test count optimization
        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        let result = optimizer
            .optimize_function_call(&context, "count", &collection, &[])
            .await
            .unwrap();

        match result {
            Some(FhirPathValue::Integer(count)) => assert_eq!(count, 3),
            _ => panic!("Expected optimized count result"),
        }
    }

    #[tokio::test]
    async fn test_exists_optimization() {
        let provider = Arc::new(MockModelProvider::empty());
        let mut optimizer = FunctionOptimizer::new(provider);

        // Test with non-empty collection
        let values = vec![FhirPathValue::Integer(1)];
        let collection = FhirPathValue::Collection(Collection::from_vec(values));

        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        let result = optimizer
            .optimize_function_call(&context, "exists", &collection, &[])
            .await
            .unwrap();

        match result {
            Some(FhirPathValue::Boolean(exists)) => assert!(exists),
            _ => panic!("Expected optimized exists result"),
        }

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(Collection::from_vec(vec![]));
        let result = optimizer
            .optimize_function_call(&context, "exists", &empty_collection, &[])
            .await
            .unwrap();

        match result {
            Some(FhirPathValue::Boolean(exists)) => assert!(!exists),
            _ => panic!("Expected optimized exists result for empty collection"),
        }
    }

    #[tokio::test]
    async fn test_first_last_optimization() {
        let provider = Arc::new(MockModelProvider::empty());
        let mut optimizer = FunctionOptimizer::new(provider);

        let values = vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("middle".into()),
            FhirPathValue::String("last".into()),
        ];
        let collection = FhirPathValue::Collection(Collection::from_vec(values));

        let mock_provider = Arc::new(MockModelProvider::empty());
        let context = EvaluationContext::new(
            FhirPathValue::Empty,
            Arc::new(FunctionRegistry::new()),
            Arc::new(OperatorRegistry::new()),
            mock_provider,
        );

        // Test first()
        let result = optimizer
            .optimize_function_call(&context, "first", &collection, &[])
            .await
            .unwrap();
        match result {
            Some(FhirPathValue::String(s)) => assert_eq!(s.as_ref(), "first"),
            _ => panic!("Expected optimized first result"),
        }

        // Test last()
        let result = optimizer
            .optimize_function_call(&context, "last", &collection, &[])
            .await
            .unwrap();
        match result {
            Some(FhirPathValue::String(s)) => assert_eq!(s.as_ref(), "last"),
            _ => panic!("Expected optimized last result"),
        }
    }

    #[test]
    fn test_type_inference() {
        let provider = Arc::new(MockModelProvider::empty());
        let optimizer = FunctionOptimizer::new(provider);

        assert_eq!(
            optimizer.infer_value_type(&FhirPathValue::String("test".into())),
            Some("string".to_string())
        );
        assert_eq!(
            optimizer.infer_value_type(&FhirPathValue::Integer(42)),
            Some("integer".to_string())
        );
        assert_eq!(
            optimizer.infer_value_type(&FhirPathValue::Boolean(true)),
            Some("boolean".to_string())
        );

        // Test collection type inference
        let values = vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)];
        let collection = FhirPathValue::Collection(Collection::from_vec(values));
        assert_eq!(
            optimizer.infer_value_type(&collection),
            Some("Collection<integer>".to_string())
        );
    }
}
