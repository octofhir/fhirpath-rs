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

//! String concatenation operation (&) implementation for FHIRPath

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// String concatenation operation (&) - concatenates string representations with special empty handling
pub struct ConcatenationOperation;

impl Default for ConcatenationOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcatenationOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "&",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("String concatenation operation with special empty handling")
        .example("'hello' & ' world'")
        .example("Patient.name.given & ' ' & Patient.name.family")
        .example("'Hello' & {}")
        .returns(TypeConstraint::Specific(FhirPathType::String))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    fn evaluate_concatenation_sync(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<Result<FhirPathValue>> {
        // Handle special empty cases according to FHIRPath spec
        match (left, right) {
            // If left operand is empty, return right operand
            (FhirPathValue::Empty, right_val) => {
                if let Some(right_str) = Self::value_to_string(right_val) {
                    Some(Ok(FhirPathValue::String(right_str.into())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            }
            // If right operand is empty, return left operand
            (left_val, FhirPathValue::Empty) => {
                if let Some(left_str) = Self::value_to_string(left_val) {
                    Some(Ok(FhirPathValue::String(left_str.into())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            }
            // Handle collections - if either contains multiple items, error
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                match (l.len(), r.len()) {
                    (1, 1) => {
                        self.evaluate_concatenation_sync(l.get(0).unwrap(), r.get(0).unwrap())
                    }
                    (0, 0) => Some(Ok(FhirPathValue::Empty)),
                    (0, 1) => {
                        // Left is empty, return right as string
                        if let Some(right_str) = Self::value_to_string(r.get(0).unwrap()) {
                            Some(Ok(FhirPathValue::String(right_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    (1, 0) => {
                        // Right is empty, return left as string
                        if let Some(left_str) = Self::value_to_string(l.get(0).unwrap()) {
                            Some(Ok(FhirPathValue::String(left_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments {
                        message: "String concatenation requires single items, not collections"
                            .to_string(),
                    })),
                }
            }
            (FhirPathValue::Collection(l), right_val) => {
                match l.len() {
                    1 => self.evaluate_concatenation_sync(l.get(0).unwrap(), right_val),
                    0 => {
                        // Left is empty, return right as string
                        if let Some(right_str) = Self::value_to_string(right_val) {
                            Some(Ok(FhirPathValue::String(right_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments {
                        message: "String concatenation requires single items, not collections"
                            .to_string(),
                    })),
                }
            }
            (left_val, FhirPathValue::Collection(r)) => {
                match r.len() {
                    1 => self.evaluate_concatenation_sync(left_val, r.get(0).unwrap()),
                    0 => {
                        // Right is empty, return left as string
                        if let Some(left_str) = Self::value_to_string(left_val) {
                            Some(Ok(FhirPathValue::String(left_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments {
                        message: "String concatenation requires single items, not collections"
                            .to_string(),
                    })),
                }
            }
            // Convert both operands to string and concatenate
            (left_val, right_val) => {
                let left_str = Self::value_to_string(left_val);
                let right_str = Self::value_to_string(right_val);
                match (left_str, right_str) {
                    (Some(l), Some(r)) => Some(Ok(FhirPathValue::String(format!("{l}{r}").into()))),
                    _ => Some(Err(FhirPathError::TypeError {
                        message: format!(
                            "Cannot concatenate {} and {}",
                            left_val.type_name(),
                            right_val.type_name()
                        ),
                    })),
                }
            }
        }
    }

    /// Convert a FhirPathValue to its string representation for concatenation
    fn value_to_string(value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::String(s) => Some(s.to_string()),
            FhirPathValue::Integer(i) => Some(i.to_string()),
            FhirPathValue::Decimal(d) => Some(d.to_string()),
            FhirPathValue::Boolean(b) => Some(b.to_string()),
            FhirPathValue::Date(d) => Some(d.to_string()),
            FhirPathValue::DateTime(dt) => Some(dt.to_string()),
            FhirPathValue::Time(t) => Some(t.to_string()),
            FhirPathValue::Empty => None, // Empty values cannot be converted to string
            FhirPathValue::Collection(_) => None, // Collections handled separately
            FhirPathValue::Resource(_) => None, // Resources cannot be directly converted
            FhirPathValue::Quantity(_) => None, // Quantities need special handling
            FhirPathValue::JsonValue(_) => None, // JSON values need special handling
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // TypeInfo objects can be converted to string representation
                if namespace.as_ref().is_empty() {
                    Some(name.to_string())
                } else {
                    Some(format!("{namespace}.{name}"))
                }
            }
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConcatenationOperation {
    fn identifier(&self) -> &str {
        "&"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConcatenationOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "&".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Use sync evaluation
        if let Some(result) = self.evaluate_concatenation_sync(&args[0], &args[1]) {
            result
        } else {
            Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot concatenate {} and {}",
                    args[0].type_name(),
                    args[1].type_name()
                ),
            })
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "&".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        self.evaluate_concatenation_sync(&args[0], &args[1])
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
