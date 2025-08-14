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

//! Unified select() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;
use rust_decimal::Decimal;

/// Unified select() function implementation
/// 
/// Projects each collection item through an expression.
/// Syntax: select(projection)
pub struct UnifiedSelectFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSelectFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "select",
            vec![ParameterInfo::required("expression", TypeInfo::Any)], // Lambda expression
            TypeInfo::Collection(Box::new(TypeInfo::Any)),
        );
        
        let metadata = MetadataBuilder::new("select", FunctionCategory::Collections)
            .display_name("Select")
            .description("Evaluates projection expression for each item, flattening results")
            .example("Bundle.entry.select(resource as Patient)")
            .example("Patient.name.select(given.first() + ' ' + family)")
            .signature(signature)
            .execution_mode(ExecutionMode::Async) // May need async evaluation for complex expressions
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("select(${1:projection})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["select", "project", "transform", "map"])
            .usage_pattern(
                "Collection projection",
                "Bundle.entry.select(resource as Patient)",
                "Transforming collections through expression evaluation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSelectFunction {
    fn name(&self) -> &str {
        "select"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::SyncFirst
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (projection expression)  
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Get input collection
        let collection = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single_item => vec![single_item.clone()].into(),
        };

        // For now, implement basic select() behavior for common patterns
        // The argument should be an expression, but since we don't have full AST evaluation,
        // we'll handle the arguments as they come in (already evaluated)
        
        let projection_arg = &args[0];
        let mut results = Vec::new();

        // For each item in the collection, apply projection logic
        for item in collection.iter() {
            match projection_arg {
                // Direct value projection - return the projection value itself
                FhirPathValue::Boolean(_) | FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                    results.push(projection_arg.clone());
                }
                // String values can be property names or literal values
                FhirPathValue::String(s) => {
                    match s.as_ref() {
                        "$this" => {
                            // Special case: $this returns the item itself
                            results.push(item.clone());
                        }
                        property_name => {
                            // Try to access this property on the item
                            if let Some(property_value) = self.get_property(item, property_name) {
                                if !matches!(property_value, FhirPathValue::Empty) {
                                    results.push(property_value);
                                }
                            }
                            // If property doesn't exist, skip this item (don't add anything)
                        }
                    }
                }
                // Collection projections - handle flattening
                FhirPathValue::Collection(projection_items) => {
                    // For each projection item, add it to results
                    for proj_item in projection_items.iter() {
                        if !matches!(proj_item, FhirPathValue::Empty) {
                            results.push(proj_item.clone());
                        }
                    }
                }
                FhirPathValue::Empty => {
                    // Empty projection results in empty - skip this item
                    continue;
                }
                _ => {
                    // For other value types, return the projection as-is
                    results.push(projection_arg.clone());
                }
            }
        }

        Ok(FhirPathValue::collection(results))
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // For sync execution, we can't handle complex lambda expressions,
        // but we can handle simple property projections and values
        
        // Validate arguments - exactly 1 required (projection expression)  
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Get the collection from context
        let collection = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single_item => vec![single_item.clone()].into(),
        };

        let projection_arg = &args[0];
        let mut results = Vec::new();

        // For each item in the collection, apply projection logic
        for item in collection.iter() {
            match projection_arg {
                // Direct value projection - return the projection value itself
                FhirPathValue::Boolean(_) | FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                    results.push(projection_arg.clone());
                }
                // String values can be property names or literal values
                FhirPathValue::String(s) => {
                    match s.as_ref() {
                        "$this" => {
                            // Special case: $this returns the item itself
                            results.push(item.clone());
                        }
                        property_name => {
                            // Try to access this property on the item
                            if let Some(property_value) = self.get_property(item, property_name) {
                                if !matches!(property_value, FhirPathValue::Empty) {
                                    results.push(property_value);
                                }
                            }
                            // If property doesn't exist, skip this item (don't add anything)
                        }
                    }
                }
                // Collection projections - handle flattening
                FhirPathValue::Collection(projection_items) => {
                    // For each projection item, add it to results
                    for proj_item in projection_items.iter() {
                        if !matches!(proj_item, FhirPathValue::Empty) {
                            results.push(proj_item.clone());
                        }
                    }
                }
                _ => {
                    // For other types, just add the projection value
                    results.push(projection_arg.clone());
                }
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

impl UnifiedSelectFunction {

    /// Get a property value from a FhirPathValue (basic property access)
    fn get_property(&self, value: &FhirPathValue, property: &str) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_value) => {
                // Try to access property from JSON value
                match json_value.as_json() {
                    serde_json::Value::Object(obj) => {
                        obj.get(property).map(|v| {
                            // Convert JSON value back to FhirPathValue
                            match v {
                                serde_json::Value::String(s) => FhirPathValue::String(s.as_str().into()),
                                serde_json::Value::Number(n) => {
                                    if let Some(i) = n.as_i64() {
                                        FhirPathValue::Integer(i)
                                    } else if let Some(f) = n.as_f64() {
                                        FhirPathValue::Decimal(Decimal::try_from(f).unwrap_or_default())
                                    } else {
                                        FhirPathValue::Empty
                                    }
                                }
                                serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
                                serde_json::Value::Null => FhirPathValue::Empty,
                                _ => FhirPathValue::JsonValue(v.clone().into()),
                            }
                        })
                    }
                    _ => None,
                }
            }
            FhirPathValue::Resource(resource) => {
                // Try to access property from resource
                resource.get_property(property).map(|v| {
                    // Convert JSON value back to FhirPathValue
                    match v {
                        serde_json::Value::String(s) => FhirPathValue::String(s.as_str().into()),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                FhirPathValue::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                FhirPathValue::Decimal(Decimal::try_from(f).unwrap_or_default())
                            } else {
                                FhirPathValue::Empty
                            }
                        }
                        serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
                        serde_json::Value::Null => FhirPathValue::Empty,
                        _ => FhirPathValue::JsonValue(v.clone().into()),
                    }
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_select_function_metadata() {
        let func = UnifiedSelectFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "select");
        assert_eq!(metadata.execution_mode, ExecutionMode::Async);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}