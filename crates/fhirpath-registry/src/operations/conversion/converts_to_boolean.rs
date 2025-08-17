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

//! Boolean conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ConvertsToBoolean function: returns true if the input can be converted to Boolean
pub struct ConvertsToBooleanFunction;

impl Default for ConvertsToBooleanFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToBooleanFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToBoolean", OperationType::Function)
            .description("Returns true if the input can be converted to Boolean")
            .example("'true'.convertsToBoolean()")
            .example("1.convertsToBoolean()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_boolean(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Boolean(_) => Ok(true),
            FhirPathValue::Integer(i) => Ok(*i == 0 || *i == 1),
            FhirPathValue::Decimal(d) => Ok(d.is_zero() || *d == rust_decimal::Decimal::ONE),
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                Ok(lower == "true"
                    || lower == "t"
                    || lower == "yes"
                    || lower == "y"
                    || lower == "1"
                    || lower == "1.0"
                    || lower == "false"
                    || lower == "f"
                    || lower == "no"
                    || lower == "n"
                    || lower == "0"
                    || lower == "0.0")
            }
            FhirPathValue::Empty => Ok(true), // Empty collection returns empty result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns empty result
                } else if c.len() == 1 {
                    Self::can_convert_to_boolean(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "convertsToBoolean() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToBooleanFunction {
    fn identifier(&self) -> &str {
        "convertsToBoolean"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToBooleanFunction::create_metadata);
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

        match Self::can_convert_to_boolean(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_boolean(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
