//! avg() function - averages numeric values in a collection

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// avg() function - averages numeric values in a collection
pub struct AvgFunction;

impl FhirPathFunction for AvgFunction {
    fn name(&self) -> &str {
        "avg"
    }
    fn human_friendly_name(&self) -> &str {
        "Average"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("avg", vec![], TypeInfo::Decimal));
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // avg() is a pure mathematical function
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

        let mut sum = Decimal::ZERO;
        let mut count = 0;

        for item in items {
            match item {
                FhirPathValue::Integer(i) => {
                    sum += Decimal::from(*i);
                    count += 1;
                }
                FhirPathValue::Decimal(d) => {
                    sum += d;
                    count += 1;
                }
                FhirPathValue::Empty => {
                    // Skip empty values
                }
                _ => {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 0,
                        expected: "Number".to_string(),
                        actual: format!("{:?}", item),
                    });
                }
            }
        }

        if count == 0 {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::Decimal(sum / Decimal::from(count)))
        }
    }
}