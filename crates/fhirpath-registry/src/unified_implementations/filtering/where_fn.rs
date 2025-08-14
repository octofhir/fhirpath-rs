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

//! Unified where() function implementation

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

/// Unified where() function implementation
/// 
/// Filters a collection based on a boolean criteria expression.
/// Syntax: where(criteria)
pub struct UnifiedWhereFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedWhereFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "where",
            vec![ParameterInfo::required("criteria", TypeInfo::Boolean)], // Predicate expression
            TypeInfo::Collection(Box::new(TypeInfo::Any)),
        );
        
        let metadata = MetadataBuilder::new("where", FunctionCategory::Collections)
            .display_name("Where")
            .description("Returns collection items for which the criteria expression evaluates to true")
            .example("Patient.telecom.where(use = 'official')")
            .example("Bundle.entry.where(resource.resourceType = 'Patient')")
            .signature(signature)
            .execution_mode(ExecutionMode::Async) // May need async evaluation for complex expressions
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("where(${1:criteria})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["filter", "where", "criteria", "condition"])
            .usage_pattern(
                "Collection filtering",
                "Patient.telecom.where(use = 'official')",
                "Filtering collections based on boolean criteria"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedWhereFunction {
    fn name(&self) -> &str {
        "where"
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
        // Validate arguments - exactly 1 required (criteria expression)
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

        let criteria_arg = &args[0];
        let mut results = Vec::new();

        // For each item in the collection, evaluate the criteria
        for item in collection.iter() {
            // Evaluate the criteria for this item
            if let Ok(Some(matches)) = self.evaluate_criteria(item, criteria_arg) {
                if matches {
                    results.push(item.clone());
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
        // Validate arguments - exactly 1 required (criteria expression)
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

        let criteria = &args[0];
        let mut results = Vec::new();

        // Filter items based on criteria
        for item in collection.iter() {
            match self.evaluate_criteria_sync(item, criteria) {
                Ok(Some(true)) => results.push(item.clone()),
                Ok(Some(false)) | Ok(None) => {}, // Skip item
                Err(_) => {}, // Skip on error for robustness
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

impl UnifiedWhereFunction {
    /// Evaluate criteria against an item (sync version)
    fn evaluate_criteria_sync(&self, item: &FhirPathValue, criteria: &FhirPathValue) -> FunctionResult<Option<bool>> {
        match criteria {
            // Direct boolean value
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            // Empty criteria evaluates to false
            FhirPathValue::Empty => Ok(Some(false)),
            // String criteria can be property names or literal comparisons
            FhirPathValue::String(s) => {
                match s.as_ref() {
                    // Special case: literal boolean strings
                    "true" => Ok(Some(true)),
                    "false" => Ok(Some(false)),
                    property_name => {
                        // Try property existence check
                        if let Some(property_value) = self.get_property(item, property_name) {
                            // Property exists if it's not empty
                            Ok(Some(!matches!(property_value, FhirPathValue::Empty)))
                        } else {
                            // Property doesn't exist
                            Ok(Some(false))
                        }
                    }
                }
            }
            // Collection criteria - check if any item in collection is truthy
            FhirPathValue::Collection(c) => {
                // Empty collection is falsy
                if c.is_empty() {
                    Ok(Some(false))
                } else {
                    // Collection is truthy if it has at least one non-empty item
                    let has_truthy = c.iter().any(|item| !matches!(item, FhirPathValue::Empty));
                    Ok(Some(has_truthy))
                }
            }
            // Other types - evaluate truthiness
            _ => {
                let is_truthy = match criteria {
                    FhirPathValue::Integer(i) => *i != 0,
                    FhirPathValue::Decimal(d) => !d.is_zero(),
                    _ => true, // Most non-empty values are truthy
                };
                Ok(Some(is_truthy))
            }
        }
    }

    /// Evaluate criteria against an item (enhanced implementation)
    fn evaluate_criteria(&self, item: &FhirPathValue, criteria: &FhirPathValue) -> FunctionResult<Option<bool>> {
        match criteria {
            // Direct boolean value
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            // Empty criteria evaluates to false
            FhirPathValue::Empty => Ok(Some(false)),
            // String criteria can be property names or literal comparisons
            FhirPathValue::String(s) => {
                match s.as_ref() {
                    // Special case: literal boolean strings
                    "true" => Ok(Some(true)),
                    "false" => Ok(Some(false)),
                    property_name => {
                        // Try property existence check
                        if let Some(property_value) = self.get_property(item, property_name) {
                            // Property exists if it's not empty
                            Ok(Some(!matches!(property_value, FhirPathValue::Empty)))
                        } else {
                            // Property doesn't exist
                            Ok(Some(false))
                        }
                    }
                }
            }
            // Collection criteria - check if any item in collection is truthy
            FhirPathValue::Collection(c) => {
                // Empty collection is falsy
                if c.is_empty() {
                    Ok(Some(false))
                } else {
                    // Collection is truthy if it has at least one non-empty item
                    let has_truthy = c.iter().any(|item| !matches!(item, FhirPathValue::Empty));
                    Ok(Some(has_truthy))
                }
            }
            // Other types - evaluate truthiness
            _ => {
                let is_truthy = match criteria {
                    FhirPathValue::Integer(i) => *i != 0,
                    FhirPathValue::Decimal(d) => !d.is_zero(),
                    _ => true, // Most non-empty values are truthy
                };
                Ok(Some(is_truthy))
            }
        }
    }

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
    async fn test_where_function_metadata() {
        let func = UnifiedWhereFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "where");
        assert_eq!(metadata.execution_mode, ExecutionMode::Async);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}