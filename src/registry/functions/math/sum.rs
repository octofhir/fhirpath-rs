//! sum() function - sums numeric values in a collection

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// sum() function - sums numeric values in a collection
pub struct SumFunction;

impl FhirPathFunction for SumFunction {
    fn name(&self) -> &str {
        "sum"
    }
    fn human_friendly_name(&self) -> &str {
        "Sum"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("sum", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // sum() is a pure mathematical function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => vec![single],
        };

        if items.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let mut int_sum: Option<i64> = None;
        let mut decimal_sum: Option<Decimal> = None;

        for item in items {
            match item {
                FhirPathValue::Integer(i) => {
                    if let Some(ref mut sum) = int_sum {
                        *sum = sum.saturating_add(*i);
                    } else if decimal_sum.is_none() {
                        int_sum = Some(*i);
                    } else {
                        decimal_sum = Some(decimal_sum.unwrap() + Decimal::from(*i));
                    }
                }
                FhirPathValue::Decimal(d) => {
                    if let Some(sum) = int_sum.take() {
                        decimal_sum = Some(Decimal::from(sum) + d);
                    } else if let Some(ref mut sum) = decimal_sum {
                        *sum += d;
                    } else {
                        decimal_sum = Some(*d);
                    }
                }
                FhirPathValue::Empty => {
                    // Skip empty values
                }
                _ => {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 0,
                        expected: "Number".to_string(),
                        actual: format!("{item:?}"),
                    });
                }
            }
        }

        if let Some(sum) = decimal_sum {
            Ok(FhirPathValue::Decimal(sum))
        } else if let Some(sum) = int_sum {
            Ok(FhirPathValue::Integer(sum))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}
