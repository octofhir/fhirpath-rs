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

//! Descendants function implementation

use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use crate::operations::fhir::ChildrenFunction;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use std::collections::HashSet;

/// Descendants function - returns a collection with all descendant nodes
/// This is a shorthand for repeat(children())
#[derive(Debug, Clone)]
pub struct DescendantsFunction {
    children_fn: ChildrenFunction,
}

impl Default for DescendantsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DescendantsFunction {
    pub fn new() -> Self {
        Self {
            children_fn: ChildrenFunction::new(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("descendants", OperationType::Function)
            .description("Returns a collection with all descendant nodes of all items in the input collection. The result does not include the nodes in the input collection themselves. This function is a shorthand for repeat(children()).")
            .example("Patient.descendants()")
            .example("Bundle.entry.descendants()")
            .returns(TypeConstraint::Collection(Box::new(TypeConstraint::Any)))
            .performance(PerformanceComplexity::Exponential, true)
            .build()
    }

    fn get_all_descendants(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        let mut to_process = vec![value.clone()];

        // Safety limits to prevent infinite loops and memory explosion
        const MAX_ITERATIONS: usize = 1000;
        const MAX_RESULT_SIZE: usize = 10000;
        let mut iteration_count = 0;

        while !to_process.is_empty()
            && iteration_count < MAX_ITERATIONS
            && result.len() < MAX_RESULT_SIZE
        {
            iteration_count += 1;
            let mut next_level = Vec::new();

            for item in to_process {
                let children = self.get_children_from_value(&item);

                for child in children {
                    let child_key = self.value_to_key(&child);
                    if seen.insert(child_key) {
                        result.push(child.clone());
                        next_level.push(child);
                    }
                }
            }

            to_process = next_level;
        }

        result
    }

    fn get_children_from_value(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if json_val.is_object() {
                    // Get all property values as children - using object iterator
                    let mut children = Vec::new();
                    if let Some(iter) = json_val.object_iter() {
                        for (_property_name, property_value) in iter {
                            // Each property value becomes a child node
                            // Arrays ARE unrolled - each element becomes a separate child
                            if property_value.is_array() {
                                if let Some(array_iter) = property_value.array_iter() {
                                    for element in array_iter {
                                        children.push(FhirPathValue::JsonValue(element));
                                    }
                                }
                            } else {
                                children.push(FhirPathValue::JsonValue(property_value));
                            }
                        }
                    }
                    children
                } else if json_val.is_array() {
                    // If the input itself is an array, each element is a child
                    if let Some(iter) = json_val.array_iter() {
                        iter.map(FhirPathValue::JsonValue).collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    // Primitive values have no children
                    Vec::new()
                }
            }
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, get children from their JSON representation - no conversions!
                let json_value = resource.as_json_value().clone();
                self.get_children_from_value(&FhirPathValue::JsonValue(json_value))
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

    fn value_to_key(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => format!("string:{}", s.as_ref()),
            FhirPathValue::Integer(i) => format!("integer:{i}"),
            FhirPathValue::Decimal(d) => format!("decimal:{d}"),
            FhirPathValue::Boolean(b) => format!("boolean:{b}"),
            FhirPathValue::Date(d) => format!("date:{d}"),
            FhirPathValue::DateTime(dt) => format!("datetime:{dt}"),
            FhirPathValue::Time(t) => format!("time:{t}"),
            FhirPathValue::JsonValue(json) => {
                // For JSON objects, try to use id if available
                if json.is_object() {
                    if let Some(id_val) = json.get_property("id") {
                        if let Some(id_str) = id_val.as_str() {
                            return format!("json:id:{id_str}");
                        }
                    }
                }
                // Fallback to string representation
                format!("json:{}", json.to_string().unwrap_or_default())
            }
            FhirPathValue::Collection(items) => {
                format!("collection:len:{}", items.len())
            }
            FhirPathValue::Empty => "empty".to_string(),
            FhirPathValue::Quantity(q) => format!("quantity:{q}"),
            FhirPathValue::Resource(r) => {
                let id = r
                    .as_json_value()
                    .get_property("id")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                format!("resource:{id}")
            }
            FhirPathValue::TypeInfoObject { name, .. } => format!("typeinfo:{name}"),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DescendantsFunction {
    fn identifier(&self) -> &str {
        "descendants"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DescendantsFunction::create_metadata);
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

        // For descendants(), we operate on the focus (current context)
        let focus = &context.input;
        let descendants = self.get_all_descendants(focus);

        Ok(FhirPathValue::Collection(Collection::from(descendants)))
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

        // For descendants(), we operate on the focus (current context)
        let focus = &context.input;
        let descendants = self.get_all_descendants(focus);

        Some(Ok(FhirPathValue::Collection(Collection::from(descendants))))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
