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

//! Unified exists() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult, FunctionError};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::{FunctionSignature, ParameterInfo};
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;

/// Unified exists() function implementation
pub struct UnifiedExistsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedExistsFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("exists", vec![
            ParameterInfo::optional("criteria", TypeInfo::Boolean)
        ], TypeInfo::Boolean);
        
        let metadata = MetadataBuilder::collection_function("exists")
            .display_name("Exists")
            .description("Returns true if the collection is not empty, or if any item matches the criteria")
            .signature(signature)
            .example("Patient.name.exists()")
            .example("Patient.name.exists(use = 'official')")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .output_is_collection(true)
            .lsp_snippet("exists(${1:criteria})")
            .keywords(vec!["exists", "present", "available", "collection", "criteria"])
            .usage_pattern_with_frequency(
                "Check if value exists",
                "Patient.name.exists()",
                "Conditional logic and validation",
                UsageFrequency::VeryCommon
            )
            .usage_pattern_with_frequency(
                "Check if matching item exists",
                "Patient.name.exists(use = 'official')",
                "Conditional matching and filtering",
                UsageFrequency::Common
            )
            .related_function("empty")
            .related_function("count")
            .related_function("where")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedExistsFunction {
    fn name(&self) -> &str {
        "exists"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate 0-1 arguments
        if args.len() > 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let exists = if args.is_empty() {
            // exists() with no criteria - check if collection is not empty
            match &context.input {
                FhirPathValue::Empty => false,
                FhirPathValue::Collection(items) => !items.is_empty(),
                _ => true,
            }
        } else {
            // exists(criteria) - check if any item matches the criteria
            let criteria = &args[0];
            match &context.input {
                FhirPathValue::Empty => false,
                FhirPathValue::Collection(items) => {
                    // For each item, check if it matches the criteria
                    items.iter().any(|item| {
                        self.evaluate_criteria(item, criteria).unwrap_or(false)
                    })
                }
                single_item => {
                    // Single item - check if it matches the criteria
                    self.evaluate_criteria(single_item, criteria).unwrap_or(false)
                }
            }
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(exists)]))
    }
}

impl UnifiedExistsFunction {
    /// Evaluate criteria against an item (basic implementation)
    fn evaluate_criteria(&self, item: &FhirPathValue, criteria: &FhirPathValue) -> Result<bool, ()> {
        match criteria {
            // Direct boolean value
            FhirPathValue::Boolean(b) => Ok(*b),
            // Empty criteria evaluates to false
            FhirPathValue::Empty => Ok(false),
            // String criteria can be property names or literal comparisons
            FhirPathValue::String(s) => {
                match s.as_ref() {
                    // Special case: literal boolean strings
                    "true" => Ok(true),
                    "false" => Ok(false),
                    property_name => {
                        // Try property existence check
                        if let Some(property_value) = self.get_property(item, property_name) {
                            // Property exists if it's not empty
                            Ok(!matches!(property_value, FhirPathValue::Empty))
                        } else {
                            // Property doesn't exist
                            Ok(false)
                        }
                    }
                }
            }
            // Collection criteria - check if any item in collection is truthy
            FhirPathValue::Collection(c) => {
                // Empty collection is falsy
                if c.is_empty() {
                    Ok(false)
                } else {
                    // Collection is truthy if it has at least one non-empty item
                    let has_truthy = c.iter().any(|item| !matches!(item, FhirPathValue::Empty));
                    Ok(has_truthy)
                }
            }
            // Other types - evaluate truthiness
            _ => {
                let is_truthy = match criteria {
                    FhirPathValue::Integer(i) => *i != 0,
                    FhirPathValue::Decimal(d) => !d.is_zero(),
                    _ => true, // Most non-empty values are truthy
                };
                Ok(is_truthy)
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
    
    #[tokio::test]
    async fn test_unified_exists_function() {
        let exists_func = UnifiedExistsFunction::new();
        
        // Test metadata
        assert_eq!(exists_func.name(), "exists");
        assert_eq!(exists_func.execution_mode(), ExecutionMode::Sync);
        assert!(exists_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = exists_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test non-empty single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = exists_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test non-empty collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
        ]));
        let result = exists_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
    }
}