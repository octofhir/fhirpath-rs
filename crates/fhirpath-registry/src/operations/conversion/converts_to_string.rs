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

//! convertsToString() implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToString(): Returns true if the input can be converted to String
pub struct ConvertsToStringFunction;

impl Default for ConvertsToStringFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToStringFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToString", OperationType::Function)
            .description("Returns true if the input can be converted to String")
            .example("true.convertsToString()")
            .example("5.convertsToString()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_string(value: &FhirPathValue) -> Result<bool> {
        match value {
            // Direct string
            FhirPathValue::String(_) => Ok(true),
            // Primitives that have string representation
            FhirPathValue::Boolean(_)
            | FhirPathValue::Integer(_)
            | FhirPathValue::Decimal(_)
            | FhirPathValue::Date(_)
            | FhirPathValue::DateTime(_)
            | FhirPathValue::Time(_)
            | FhirPathValue::Quantity(_) => Ok(true),
            // JSON simple types convertible by to_string_value()
            FhirPathValue::JsonValue(json) => {
                let inner = json.as_inner();
                use sonic_rs::JsonValueTrait;
                Ok(inner.as_str().is_some()
                    || inner.as_bool().is_some()
                    || inner.as_f64().is_some()
                    || inner.is_null())
            }
            // Empty yields empty result; spec treats convertsTo* on empty as true
            FhirPathValue::Empty => Ok(true),
            // Collection rules
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true)
                } else if c.len() == 1 {
                    Self::can_convert_to_string(c.first().unwrap())
                } else {
                    Err(FhirPathError::evaluation_error("convertsToString() requires a single item, but collection has multiple items".to_string() ))
                }
            }
            // Other complex types cannot convert
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToStringFunction {
    fn identifier(&self) -> &str {
        "convertsToString"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToStringFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }
        match Self::can_convert_to_string(&context.input) {
            Ok(b) => Ok(FhirPathValue::Boolean(b)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(match Self::can_convert_to_string(&context.input) {
            Ok(b) => Ok(FhirPathValue::Boolean(b)),
            Err(e) => Err(e),
        })
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
