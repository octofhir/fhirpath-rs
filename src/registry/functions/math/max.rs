//! max() function - finds maximum value in a collection

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// max() function - finds maximum value in a collection
pub struct MaxFunction;

impl FhirPathFunction for MaxFunction {
    fn name(&self) -> &str {
        "max"
    }
    fn human_friendly_name(&self) -> &str {
        "Maximum"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("max", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // max() is a pure mathematical function
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

        let mut max_value: Option<FhirPathValue> = None;

        for item in items {
            match item {
                FhirPathValue::Empty => continue, // Skip empty values
                _ => {
                    if let Some(ref current_max) = max_value {
                        // Compare values
                        if let Ok(ordering) = self.compare_values(item, current_max) {
                            if ordering == std::cmp::Ordering::Greater {
                                max_value = Some(item.clone());
                            }
                        }
                    } else {
                        max_value = Some(item.clone());
                    }
                }
            }
        }

        match max_value {
            Some(value) => Ok(value),
            None => Ok(FhirPathValue::Empty),
        }
    }
}

impl MaxFunction {
    fn compare_values(
        &self,
        a: &FhirPathValue,
        b: &FhirPathValue,
    ) -> Result<std::cmp::Ordering, FunctionError> {
        match (a, b) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Ok(Decimal::from(*a).cmp(b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => Ok(a.cmp(&Decimal::from(*b))),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(a.cmp(b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Ok(a.cmp(b)),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(a.cmp(b)),
            _ => Err(FunctionError::InvalidArgumentType {
                name: "max".to_string(),
                index: 0,
                expected: "Comparable types".to_string(),
                actual: format!("Cannot compare {a:?} and {b:?}"),
            }),
        }
    }
}
