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

//! Unified children() function implementation

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

/// Unified children() function implementation
/// 
/// Returns all immediate child nodes of the input collection.
/// Syntax: children()
pub struct UnifiedChildrenFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedChildrenFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("children", FunctionCategory::Collections)
            .display_name("Children")
            .description("Returns all immediate child nodes of input items")
            .example("Bundle.children()")
            .example("Patient.name.children()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("children()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["children", "child", "tree", "navigation"])
            .usage_pattern(
                "Tree navigation",
                "Bundle.children()",
                "Getting immediate child nodes"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedChildrenFunction {
    fn name(&self) -> &str {
        "children"
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
        
        let mut children = Vec::new();
        
        match &context.input {
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    self.collect_children(item, &mut children);
                }
            }
            single_item => {
                self.collect_children(single_item, &mut children);
            }
        }
        
        if children.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(children))
        }
    }
}

impl UnifiedChildrenFunction {
    /// Collect immediate children of a value
    fn collect_children(&self, value: &FhirPathValue, children: &mut Vec<FhirPathValue>) {
        match value {
            FhirPathValue::Resource(resource) => {
                // Add all object properties as children
                for (_, child_value) in resource.as_json().as_object().unwrap_or(&serde_json::Map::new()).iter() {
                    children.push(self.json_to_fhirpath_value(child_value));
                }
            }
            FhirPathValue::JsonValue(json_value) => {
                // Add all object properties as children
                if let Some(obj) = json_value.as_object() {
                    for (_, child_value) in obj.iter() {
                        children.push(self.json_to_fhirpath_value(child_value));
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                // Collections themselves don't have children, but their items do
                // This follows the FHIRPath specification
                for item in items.iter() {
                    self.collect_children(item, children);
                }
            }
            _ => {
                // Primitive values have no children
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
    async fn test_children_function() {
        let func = UnifiedChildrenFunction::new();
        
        // Test with resource
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
            assert!(items.len() >= 3); // resourceType, id, name
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
        let func = UnifiedChildrenFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "children");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}