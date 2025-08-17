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

//! Children function implementation

use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Children function - returns a collection with all immediate child nodes
#[derive(Debug, Clone)]
pub struct ChildrenFunction;

impl Default for ChildrenFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildrenFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("children", OperationType::Function)
            .description("Returns a collection with all immediate child nodes of all items in the input collection")
            .example("Patient.children()")
            .example("Bundle.entry.children()")
            .returns(TypeConstraint::Collection(Box::new(TypeConstraint::Any)))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn get_children_from_value(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if let Some(obj) = json_val.as_object() {
                    // Get all property values as children
                    let mut children = Vec::new();
                    for (_property_name, property_value) in obj.iter() {
                        // Each property value becomes a child node
                        // Arrays ARE unrolled - each element becomes a separate child
                        if let Some(arr) = property_value.as_array() {
                            for element in arr {
                                children.push(FhirPathValue::from(element.clone()));
                            }
                        } else {
                            children.push(FhirPathValue::from(property_value.clone()));
                        }
                    }
                    children
                } else if let Some(arr) = json_val.as_array() {
                    // If the input itself is an array, each element is a child
                    arr.iter()
                        .map(|value| FhirPathValue::from(value.clone()))
                        .collect()
                } else {
                    // Primitive values have no children
                    Vec::new()
                }
            }
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, get children from their JSON representation
                self.get_children_from_value(&FhirPathValue::JsonValue(
                    resource.as_json().clone().into(),
                ))
            }
            FhirPathValue::Collection(items) => {
                // Get children of all items in collection
                let mut all_children = Vec::new();
                for item in items.iter() {
                    all_children.extend(self.get_children_from_value(item));
                }
                all_children
            }
            // Primitive values don't have children
            _ => Vec::new(),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ChildrenFunction {
    fn identifier(&self) -> &str {
        "children"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ChildrenFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        // For children(), we operate on the focus (current context)
        let focus = &context.input;
        let children = self.get_children_from_value(focus);

        Ok(FhirPathValue::Collection(Collection::from(children)))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        // For children(), we operate on the focus (current context)
        let focus = &context.input;
        let children = self.get_children_from_value(focus);

        Some(Ok(FhirPathValue::Collection(Collection::from(children))))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
