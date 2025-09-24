//! lowBoundary function implementation
//!
//! Computes the lowest possible value for the supplied input at the requested precision.

use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::core::error_code::{FP0053, FP0054, FP0055, FP0056, FP0057};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::functions::boundary_utils::{
    BoundaryKind, NumericBoundaryError, compute_date_boundary, compute_datetime_boundary,
    compute_numeric_boundaries, compute_time_boundary, resolve_date_precision,
    resolve_datetime_precision, resolve_time_precision,
};

pub struct LowBoundaryFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LowBoundaryFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "lowBoundary".to_string(),
                description: "Returns the lowest possible value that could be represented by the input at the supplied precision.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "precision".to_string(),
                        parameter_type: vec!["Integer".to_string(), "Decimal".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "Optional precision override".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for LowBoundaryFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "lowBoundary accepts at most one precision argument".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0054,
                "lowBoundary expects a singleton input".to_string(),
            ));
        }

        let precision_value = if let Some(arg) = args.first() {
            if arg.is_empty() {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
            if arg.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    FP0056,
                    "precision argument must evaluate to a single value".to_string(),
                ));
            }

            Some(match &arg[0] {
                FhirPathValue::Integer(i, _, _) => *i as i32,
                FhirPathValue::Decimal(d, _, _) => d.to_i32().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        FP0057,
                        "precision argument must be an integer value".to_string(),
                    )
                })?,
                other => {
                    return Err(FhirPathError::evaluation_error(
                        FP0055,
                        format!(
                            "precision argument must be Integer or Decimal, got {}",
                            other.type_name()
                        ),
                    ));
                }
            })
        } else {
            None
        };

        let result = match &input[0] {
            FhirPathValue::Decimal(value, type_info, primitive) => {
                match compute_numeric_boundaries(*value, precision_value) {
                    Ok(boundary) => {
                        let low = apply_requested_scale(boundary.low, boundary.requested_scale);
                        FhirPathValue::Decimal(low, type_info.clone(), primitive.clone())
                    }
                    Err(NumericBoundaryError::PrecisionOutOfRange) => {
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                }
            }
            FhirPathValue::Integer(value, _, _) => {
                match compute_numeric_boundaries(Decimal::from(*value), precision_value) {
                    Ok(boundary) => {
                        let low = apply_requested_scale(boundary.low, boundary.requested_scale);
                        FhirPathValue::decimal(low)
                    }
                    Err(NumericBoundaryError::PrecisionOutOfRange) => {
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                }
            }
            FhirPathValue::Quantity {
                value,
                unit,
                code,
                system,
                ucum_unit,
                calendar_unit,
                type_info,
                primitive_element,
            } => match compute_numeric_boundaries(*value, precision_value) {
                Ok(boundary) => {
                    let adjusted = apply_requested_scale(boundary.low, boundary.requested_scale);

                    // Ensure system field has default value for UCUM units when not already set
                    let resolved_system = if system.is_none() && ucum_unit.is_some() {
                        Some("http://unitsofmeasure.org".to_string())
                    } else {
                        system.clone()
                    };

                    FhirPathValue::Quantity {
                        value: adjusted,
                        unit: unit.clone(),
                        code: code.clone(),
                        system: resolved_system,
                        ucum_unit: ucum_unit.clone(),
                        calendar_unit: *calendar_unit,
                        type_info: type_info.clone(),
                        primitive_element: primitive_element.clone(),
                    }
                }
                Err(NumericBoundaryError::PrecisionOutOfRange) => {
                    return Ok(EvaluationResult {
                        value: Collection::empty(),
                    });
                }
            },
            FhirPathValue::DateTime(datetime, type_info, primitive) => {
                match resolve_datetime_precision(datetime.precision, precision_value) {
                    Ok(target_precision) => {
                        let adjusted = compute_datetime_boundary(
                            datetime,
                            target_precision,
                            BoundaryKind::Low,
                        );
                        FhirPathValue::DateTime(adjusted, type_info.clone(), primitive.clone())
                    }
                    Err(NumericBoundaryError::PrecisionOutOfRange) => {
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                }
            }
            FhirPathValue::Date(date, type_info, primitive) => {
                match resolve_date_precision(date.precision, precision_value) {
                    Ok(target_precision) => {
                        let adjusted =
                            compute_date_boundary(date, target_precision, BoundaryKind::Low);
                        FhirPathValue::Date(adjusted, type_info.clone(), primitive.clone())
                    }
                    Err(NumericBoundaryError::PrecisionOutOfRange) => {
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                }
            }
            FhirPathValue::Time(time, type_info, primitive) => {
                match resolve_time_precision(time.precision, precision_value) {
                    Ok(target_precision) => {
                        let adjusted =
                            compute_time_boundary(time, target_precision, BoundaryKind::Low);
                        FhirPathValue::Time(adjusted, type_info.clone(), primitive.clone())
                    }
                    Err(NumericBoundaryError::PrecisionOutOfRange) => {
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                }
            }
            FhirPathValue::String(s, type_info, primitive) => {
                // Try to parse as temporal value
                if let Ok(parsed_date) = crate::core::parsing::parse_date_string(s) {
                    match resolve_date_precision(parsed_date.precision, precision_value) {
                        Ok(target_precision) => {
                            let adjusted = compute_date_boundary(
                                &parsed_date,
                                target_precision,
                                BoundaryKind::Low,
                            );
                            FhirPathValue::Date(adjusted, type_info.clone(), primitive.clone())
                        }
                        Err(NumericBoundaryError::PrecisionOutOfRange) => {
                            return Ok(EvaluationResult {
                                value: Collection::empty(),
                            });
                        }
                    }
                } else if let Ok(parsed_datetime) = crate::core::parsing::parse_datetime_string(s) {
                    match resolve_datetime_precision(parsed_datetime.precision, precision_value) {
                        Ok(target_precision) => {
                            let adjusted = compute_datetime_boundary(
                                &parsed_datetime,
                                target_precision,
                                BoundaryKind::Low,
                            );
                            FhirPathValue::DateTime(adjusted, type_info.clone(), primitive.clone())
                        }
                        Err(NumericBoundaryError::PrecisionOutOfRange) => {
                            return Ok(EvaluationResult {
                                value: Collection::empty(),
                            });
                        }
                    }
                } else {
                    return Err(FhirPathError::evaluation_error(
                        FP0055,
                        format!(
                            "lowBoundary cannot be applied to String '{s}' (not a valid temporal value)"
                        ),
                    ));
                }
            }
            other => {
                return Err(FhirPathError::evaluation_error(
                    FP0055,
                    format!("lowBoundary cannot be applied to {}", other.type_name()),
                ));
            }
        };

        Ok(EvaluationResult {
            value: Collection::from(vec![result]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

fn apply_requested_scale(mut value: Decimal, requested: Option<u32>) -> Decimal {
    if let Some(scale) = requested {
        if value.scale() < scale {
            // Increase the scale without changing the numeric value
            value.rescale(scale);
        }
    }
    value
}
