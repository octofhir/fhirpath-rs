//! ToQuantity function implementation
//!
//! The toQuantity function converts a value to a quantity.
//! Syntax: value.toQuantity()

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use rust_decimal::Decimal;
use std::sync::Arc;

pub struct ToQuantityFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToQuantityFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toQuantity".to_string(),
                description: "Converts a value to a quantity".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "unit".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "UCUM code or calendar duration keyword to convert to"
                            .to_string(),
                        default_value: None,
                    }],
                    return_type: "Quantity".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToQuantityFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        let unit = unit_argument(&args, "toQuantity")?;
        let value = match input.len() {
            0 => None,
            1 => to_quantity(&input[0], unit.as_deref()),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "toQuantity requires a single input item".to_string(),
                ));
            }
        };
        Ok(EvaluationResult {
            value: value
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// The optional unit argument of `toQuantity`/`convertsToQuantity`: a single string.
pub(crate) fn unit_argument(args: &[Collection], function: &str) -> Result<Option<String>> {
    match args {
        [] => Ok(None),
        [arg] => match arg.first() {
            None => Ok(None),
            Some(FhirPathValue::String(unit, _, _)) if arg.len() == 1 => Ok(Some(unit.clone())),
            Some(_) => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("{function} unit argument must be a single string"),
            )),
        },
        _ => Err(FhirPathError::evaluation_error(
            crate::core::error_code::FP0053,
            format!("{function} takes at most one argument"),
        )),
    }
}

/// `value.toQuantity([unit])` for one item (FHIRPath §1.5.5.7.1): numbers take the unit
/// `'1'`, booleans become `1.0 '1'`/`0.0 '1'`, strings parse as quantity literals; with a
/// unit the quantity is converted to it. `None` when not convertible.
pub(crate) fn to_quantity(value: &FhirPathValue, unit: Option<&str>) -> Option<FhirPathValue> {
    let unitless = |amount: Decimal| {
        FhirPathValue::quantity_with_components(amount, Some("1".to_string()), None, None)
    };
    let quantity = match value {
        FhirPathValue::Quantity { .. } => value.clone(),
        FhirPathValue::Integer(i, _, _) => unitless(Decimal::from(*i)),
        FhirPathValue::Decimal(d, _, _) => unitless(*d),
        FhirPathValue::Boolean(b, _, _) => unitless(if *b {
            Decimal::new(10, 1)
        } else {
            Decimal::new(0, 1)
        }),
        FhirPathValue::String(s, _, _) => {
            crate::evaluator::quantity_utils::parse_string_to_quantity_value(s)?
        }
        _ => return None,
    };
    match unit {
        None => Some(quantity),
        Some(unit) => crate::evaluator::quantity_utils::quantity_to_unit(&quantity, unit),
    }
}
