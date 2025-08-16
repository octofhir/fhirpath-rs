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

//! HasValue function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// HasValue function - returns true if the input collection contains exactly one item that has a value
#[derive(Debug, Clone)]
pub struct HasValueFunction;

impl Default for HasValueFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl HasValueFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("hasValue", OperationType::Function)
            .description("Returns true if the input collection contains exactly one item that has a value (i.e., is not empty)")
            .example("Patient.name.hasValue()")
            .example("'hello'.hasValue()")
            .example("{}.hasValue()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn item_has_value(&self, item: &FhirPathValue) -> bool {
        match item {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => !items.is_empty(),
            FhirPathValue::String(s) => !s.is_empty(),
            FhirPathValue::JsonValue(json) => match json.as_json() {
                serde_json::Value::Object(obj) => !obj.is_empty(),
                serde_json::Value::Array(arr) => !arr.is_empty(),
                serde_json::Value::String(s) => !s.is_empty(),
                serde_json::Value::Null => false,
                _ => true,
            },
            // All other value types are considered to have value if they exist
            _ => true,
        }
    }
}

#[async_trait]
impl FhirPathOperation for HasValueFunction {
    fn identifier(&self) -> &str {
        "hasValue"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(HasValueFunction::create_metadata);
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

        let input = &context.input;

        let has_value = match input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item that is not empty/null
                items.len() == 1 && self.item_has_value(items.get(0).unwrap())
            }
            _ => {
                // Single item - check if it has a value
                self.item_has_value(input)
            }
        };

        Ok(FhirPathValue::Boolean(has_value))
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

        let input = &context.input;

        let has_value = match input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item that is not empty/null
                items.len() == 1 && self.item_has_value(items.get(0).unwrap())
            }
            _ => {
                // Single item - check if it has a value
                self.item_has_value(input)
            }
        };

        Some(Ok(FhirPathValue::Boolean(has_value)))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
