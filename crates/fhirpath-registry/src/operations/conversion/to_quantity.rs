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

//! toQuantity([unit : String]) implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{quantity::Quantity, FhirPathValue};

/// toQuantity function: converts input to Quantity optionally using provided unit
pub struct ToQuantityFunction;

impl ToQuantityFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toQuantity", OperationType::Function)
            .description("Converts input to Quantity. If a unit is provided, returns the quantity in that unit when possible.")
            .example("5.toQuantity('mg')")
            .example("'5 \"kg\"'.toQuantity()")
            .returns(TypeConstraint::Specific(FhirPathType::Quantity))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn apply_unit(q: &Quantity, unit: &str) -> Option<Quantity> {
        match &q.unit {
            Some(_) => q.convert_to_compatible_unit(unit).ok(),
            None => Some(Quantity::new(q.value, Some(unit.to_string()))),
        }
    }

    fn convert_to_quantity(input: &FhirPathValue, unit_arg: Option<&FhirPathValue>) -> Result<FhirPathValue> {
        // Validate argument (if present)
        let unit_str: Option<String> = match unit_arg {
            None => None,
            Some(FhirPathValue::String(s)) => Some(s.to_string()),
            Some(FhirPathValue::Empty) => None,
            Some(_) => {
                return Err(FhirPathError::EvaluationError {
                    message: "toQuantity(unit) expects unit to be a String".to_string(),
                })
            }
        };

        // Resolve single value from collection or pass through
        let value = match input {
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else if c.len() == 1 {
                    c.first().unwrap()
                } else {
                    return Err(FhirPathError::EvaluationError { message: "toQuantity() requires a single item, but collection has multiple items".to_string() });
                }
            }
            other => other,
        };

        // Use helper to parse/convert to quantity
        let q_opt = match value {
            FhirPathValue::Quantity(q) => Some((**q).clone()),
            // Integer/Decimal/String are supported by to_quantity_value()
            _ => value.to_quantity_value().map(|arc_q| (*arc_q).clone()),
        };

        let quantity = match q_opt {
            None => return Ok(FhirPathValue::Empty),
            Some(q) => match unit_str {
                None => q,
                Some(ref unit) => match Self::apply_unit(&q, unit) {
                    Some(q2) => q2,
                    None => return Ok(FhirPathValue::Empty),
                },
            },
        };

        Ok(FhirPathValue::from(quantity))
    }
}

#[async_trait]
impl FhirPathOperation for ToQuantityFunction {
    fn identifier(&self) -> &str {
        "toQuantity"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ToQuantityFunction::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        match args.len() {
            0 => Self::convert_to_quantity(&context.input, None),
            1 => Self::convert_to_quantity(&context.input, Some(&args[0])),
            _ => Err(FhirPathError::EvaluationError {
                message: "toQuantity() expects zero or one argument".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let res = match args.len() {
            0 => Self::convert_to_quantity(&context.input, None),
            1 => Self::convert_to_quantity(&context.input, Some(&args[0])),
            _ => Err(FhirPathError::EvaluationError {
                message: "toQuantity() expects zero or one argument".to_string(),
            }),
        };
        Some(res)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn ctx(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_to_quantity_basic() {
        let f = ToQuantityFunction::new();

        // quantity passthrough
        let q = Quantity::new(Decimal::from(5), Some("mg".into()));
        let r = f.evaluate(&[], &ctx(FhirPathValue::from(q))).await.unwrap();
        match r { FhirPathValue::Quantity(_) => {}, _ => panic!("expected Quantity") }

        // integer -> unitless
        let r = f.evaluate(&[], &ctx(FhirPathValue::Integer(10))).await.unwrap();
        match r { FhirPathValue::Quantity(q) => { assert_eq!(q.value, Decimal::from(10)); assert!(q.unit.is_none()); }, _ => panic!("expected Quantity") }

        // string quantity
        let r = f.evaluate(&[], &ctx(FhirPathValue::String("5 'kg'".into()))).await.unwrap();
        match r { FhirPathValue::Quantity(q) => { assert_eq!(q.unit.as_deref(), Some("kg")); }, _ => panic!("expected Quantity") }

        // invalid string -> Empty
        let r = f.evaluate(&[], &ctx(FhirPathValue::String("abc".into()))).await.unwrap();
        assert!(matches!(r, FhirPathValue::Empty));
    }

    #[tokio::test]
    async fn test_to_quantity_with_unit() {
        let f = ToQuantityFunction::new();

        // number + unit -> assign unit
        let r = f.evaluate(&[FhirPathValue::String("mg".into())], &ctx(FhirPathValue::Integer(5))).await.unwrap();
        match r { FhirPathValue::Quantity(q) => { assert_eq!(q.value, Decimal::from(5)); assert_eq!(q.unit.as_deref(), Some("mg")); }, _ => panic!("expected Quantity") }

        // quantity + unit -> convert
        let q = Quantity::new(Decimal::from(4), Some("g".into()));
        let r = f.evaluate(&[FhirPathValue::String("mg".into())], &ctx(FhirPathValue::from(q))).await.unwrap();
        match r { FhirPathValue::Quantity(q2) => { assert_eq!(q2.unit.as_deref(), Some("mg")); }, _ => panic!("expected Quantity") }

        // incompatible conversion -> Empty
        let q = Quantity::new(Decimal::from(1), Some("m".into()));
        let r = f.evaluate(&[FhirPathValue::String("s".into())], &ctx(FhirPathValue::from(q))).await.unwrap();
        assert!(matches!(r, FhirPathValue::Empty));
    }

    #[tokio::test]
    async fn test_to_quantity_errors_and_sync() {
        let f = ToQuantityFunction::new();

        // multi-item collection -> error
        let col = FhirPathValue::collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let err = f.evaluate(&[], &ctx(col)).await.unwrap_err();
        match err { FhirPathError::EvaluationError { .. } => {}, _ => panic!("expected eval error") }

        // wrong arg type -> error
        let err = f.evaluate(&[FhirPathValue::Integer(1)], &ctx(FhirPathValue::Integer(1))).await.unwrap_err();
        match err { FhirPathError::EvaluationError { .. } => {}, _ => panic!("expected eval error") }

        // sync path
        let r = f.try_evaluate_sync(&[], &ctx(FhirPathValue::Integer(3))).unwrap().unwrap();
        assert!(matches!(r, FhirPathValue::Quantity(_)));
        assert!(f.supports_sync());
    }
}
