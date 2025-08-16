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

//! Equals operator (=) implementation

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// Equals operator (=)
#[derive(Debug, Clone)]
pub struct EqualsOperation;

impl Default for EqualsOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl EqualsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "=",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("Equality comparison operator")
        .example("1 = 1")
        .example("'hello' = 'hello'")
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn compare_equal(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        Self::compare_equal_with_collections(left, right).map(|opt| opt.unwrap_or(false))
    }

    pub fn compare_equal_with_collections(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<bool>> {
        match (left, right) {
            // Both empty collections - return empty (not true)
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r))
                if l.is_empty() && r.is_empty() =>
            {
                Ok(None)
            }
            // Either is empty collection - return empty (not false)
            (FhirPathValue::Collection(l), _) if l.is_empty() => Ok(None),
            (_, FhirPathValue::Collection(r)) if r.is_empty() => Ok(None),
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(None),

            // Collection comparison - both must have same number of items and be equal element-wise
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() != r.len() {
                    return Ok(Some(false));
                }

                // Compare element by element
                for (left_item, right_item) in l.iter().zip(r.iter()) {
                    match Self::compare_equal_with_collections(left_item, right_item)? {
                        Some(false) => return Ok(Some(false)), // Any element not equal = whole not equal
                        None => return Ok(None), // Any element comparison is empty = whole is empty
                        Some(true) => continue,  // This element is equal, check next
                    }
                }
                Ok(Some(true)) // All elements equal
            }

            // Single value vs collection - unwrap if singleton
            (FhirPathValue::Collection(l), right_val) => {
                if l.len() == 1 {
                    Self::compare_equal_with_collections(l.get(0).unwrap(), right_val)
                } else {
                    Ok(Some(false)) // Multi-element collection can't equal single value
                }
            }
            (left_val, FhirPathValue::Collection(r)) => {
                if r.len() == 1 {
                    Self::compare_equal_with_collections(left_val, r.get(0).unwrap())
                } else {
                    Ok(Some(false)) // Single value can't equal multi-element collection
                }
            }

            // Scalar value comparisons
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(Some(a == b)),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(Some(a == b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(Some((a - b).abs() < Decimal::new(1, 10)))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(Some((Decimal::from(*a) - b).abs() < Decimal::new(1, 10)))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(Some((a - Decimal::from(*b)).abs() < Decimal::new(1, 10)))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(Some(a == b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(Some(a == b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                // Check if one has explicit timezone and other doesn't
                // We use +00:01 for implicit timezone and +00:00 for explicit Z
                let a_offset_seconds = a.offset().local_minus_utc();
                let b_offset_seconds = b.offset().local_minus_utc();

                // If one is +00:00 (explicit Z) and other is +00:01 (implicit), indeterminate
                if (a_offset_seconds == 0 && b_offset_seconds == 60)
                    || (a_offset_seconds == 60 && b_offset_seconds == 0)
                {
                    Ok(None) // Indeterminate when timezone precision differs
                } else {
                    Ok(Some(a == b))
                }
            }
            // Date vs DateTime comparisons - return empty (indeterminate) per FHIRPath spec
            (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => {
                // Date vs DateTime comparison is indeterminate due to precision differences
                Ok(None)
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => {
                // DateTime vs Date comparison is indeterminate due to precision differences
                Ok(None)
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(Some(a == b)),
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Use UCUM-aware quantity comparison with unit conversion
                match a.equals_with_conversion(b) {
                    Ok(result) => Ok(Some(result)),
                    Err(_) => Ok(Some(false)), // If conversion fails, quantities are not equal
                }
            }
            // Resource object comparison - compare JSON representations
            (FhirPathValue::Resource(a), FhirPathValue::Resource(b)) => Ok(Some(a == b)),
            // Handle other FhirPathValue types that might not be explicitly covered
            (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => {
                // Same type - use default equality comparison
                Ok(Some(a == b))
            }
            _ => {
                // Different types - not equal
                Ok(Some(false))
            }
        }
    }
}

#[async_trait]
impl FhirPathOperation for EqualsOperation {
    fn identifier(&self) -> &str {
        "="
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(EqualsOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_collection_aware_operator(
            &args[0],
            &args[1],
            Self::compare_equal_with_collections,
        )
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_collection_aware_operator(
            &args[0],
            &args[1],
            Self::compare_equal_with_collections,
        ))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
