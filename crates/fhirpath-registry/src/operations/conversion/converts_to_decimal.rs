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

//! Decimal conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// ConvertsToDecimal function: returns true if the input can be converted to Decimal
pub struct ConvertsToDecimalFunction;

impl Default for ConvertsToDecimalFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToDecimalFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToDecimal", OperationType::Function)
            .description("Returns true if the input can be converted to Decimal")
            .example("'1.5'.convertsToDecimal()")
            .example("true.convertsToDecimal()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_decimal(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Decimal(_) => Ok(true),
            FhirPathValue::Integer(_) => Ok(true),
            FhirPathValue::Boolean(_) => Ok(true),
            FhirPathValue::String(s) => {
                // Try to parse as decimal
                Ok(s.trim().parse::<Decimal>().is_ok())
            }
            FhirPathValue::Empty => Ok(true), // Empty collection returns true result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns true result
                } else if c.len() == 1 {
                    Self::can_convert_to_decimal(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "convertsToDecimal() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToDecimalFunction {
    fn identifier(&self) -> &str {
        "convertsToDecimal"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToDecimalFunction::create_metadata);
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

        match Self::can_convert_to_decimal(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_decimal(&context.input) {
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
