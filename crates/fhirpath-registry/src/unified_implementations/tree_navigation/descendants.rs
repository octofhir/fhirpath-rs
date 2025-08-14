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

//! Unified descendants() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified descendants() function implementation
/// 
/// Returns all descendant nodes of the input collection.
/// Syntax: descendants()
pub struct UnifiedDescendantsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedDescendantsFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("descendants", FunctionCategory::Collections)
            .display_name("Descendants")
            .description("Returns all descendant nodes of input items (shorthand for repeat(children()))")
            .example("Bundle.descendants()")
            .example("Patient.descendants().ofType(string)")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // Can be expensive for deep trees
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("descendants()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["descendants", "tree", "navigation", "recursive"])
            .usage_pattern(
                "Deep tree navigation",
                "Bundle.descendants()",
                "Getting all descendant nodes recursively"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedDescendantsFunction {
    fn name(&self) -> &str {
        "descendants"
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
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        let mut descendants = Vec::new();
        
        match &context.input {
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    self.collect_descendants(item, &mut descendants);
                }
            }
            single_item => {
                self.collect_descendants(single_item, &mut descendants);
            }
        }
        
        if descendants.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(descendants))
        }
    }
}

impl UnifiedDescendantsFunction {
    /// Collect all descendants of a value (recursive)
    fn collect_descendants(&self, value: &FhirPathValue, descendants: &mut Vec<FhirPathValue>) {
        match value {
            FhirPathValue::Resource(resource) => {
                // Add all object properties as descendants and recurse
                for (_, child_value) in resource.as_json().as_object().unwrap_or(&serde_json::Map::new()).iter() {
                    let child_fhir_value = self.json_to_fhirpath_value(child_value);
                    descendants.push(child_fhir_value.clone());
                    // Recursively collect descendants of this child
                    self.collect_descendants(&child_fhir_value, descendants);
                }
            }
            FhirPathValue::JsonValue(json_value) => {
                // Add all object properties as descendants and recurse
                if let Some(obj) = json_value.as_object() {
                    for (_, child_value) in obj.iter() {
                        let child_fhir_value = self.json_to_fhirpath_value(child_value);
                        descendants.push(child_fhir_value.clone());
                        // Recursively collect descendants of this child
                        self.collect_descendants(&child_fhir_value, descendants);
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                // Collections themselves don't have children, but their items do
                for item in items.iter() {
                    self.collect_descendants(item, descendants);
                }
            }
            _ => {
                // Primitive values have no descendants
            }
        }
    }
    
    /// Convert JSON value to FhirPathValue
    fn json_to_fhirpath_value(&self, value: &serde_json::Value) -> FhirPathValue {
        match value {
            serde_json::Value::Null => FhirPathValue::Empty,
            serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    use rust_decimal::prelude::FromPrimitive;
                    if let Some(decimal) = rust_decimal::Decimal::from_f64(f) {
                        FhirPathValue::Decimal(decimal)
                    } else {
                        FhirPathValue::Empty
                    }
                } else {
                    FhirPathValue::Empty
                }
            }
            serde_json::Value::String(s) => FhirPathValue::String(s.clone().into()),
            serde_json::Value::Array(arr) => {
                let items: Vec<FhirPathValue> = arr.iter()
                    .map(|item| self.json_to_fhirpath_value(item))
                    .collect();
                FhirPathValue::collection(items)
            }
            serde_json::Value::Object(_) => {
                // Convert JSON object to FhirPathValue JsonValue
                FhirPathValue::JsonValue(value.clone().into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use serde_json::json;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_descendants_function() {
        let func = UnifiedDescendantsFunction::new();
        
        // Test with simple resource
        let resource_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "name": [
                {
                    "given": ["John"],
                    "family": "Doe"
                }
            ]
        });
        let resource = FhirPathValue::JsonValue(resource_json.into());
        
        let context = create_test_context(resource);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            // Should have many descendants (resourceType, id, name array, name object, given array, given string, family string, etc.)
            assert!(items.len() > 5);
        } else {
            panic!("Expected collection result");
        }
        
        // Test with empty input
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedDescendantsFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "descendants");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}