//! Weight function implementation
//!
//! The weight function returns the ordinal weight of a coded element.
//! It first checks for the itemWeight extension, then falls back to
//! code system weight property via the terminology provider.
//! Syntax: element.weight()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use rust_decimal::Decimal;

const ITEM_WEIGHT_EXTENSION_URL: &str = "http://hl7.org/fhir/StructureDefinition/itemWeight";
const ORDINAL_VALUE_EXTENSION_URL: &str = "http://hl7.org/fhir/StructureDefinition/ordinalValue";

/// Weight function evaluator
pub struct WeightFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl WeightFunctionEvaluator {
    /// Create a new weight function evaluator
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "weight".to_string(),
                description:
                    "Returns the ordinal weight of an element from itemWeight extension or code system"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Decimal".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: true,
                requires_model: false,
            },
        })
    }

    /// Try to extract weight from itemWeight or ordinalValue extension
    fn extract_weight_from_extensions(value: &FhirPathValue) -> Option<Decimal> {
        match value {
            FhirPathValue::Resource(json, _, _) => {
                // Check extensions on the element
                if let Some(extensions) = json.get("extension").and_then(|e| e.as_array()) {
                    for ext in extensions {
                        if let Some(url) = ext.get("url").and_then(|u| u.as_str())
                            && (url == ITEM_WEIGHT_EXTENSION_URL
                                || url == ORDINAL_VALUE_EXTENSION_URL)
                        {
                            if let Some(val) = ext.get("valueDecimal").and_then(|v| v.as_f64()) {
                                return Decimal::try_from(val).ok();
                            }
                            if let Some(val) = ext.get("valueInteger").and_then(|v| v.as_i64()) {
                                return Some(Decimal::from(val));
                            }
                        }
                    }
                }
                None
            }
            // Check wrapped primitive element extensions
            other => {
                if let Some(pe) = other.wrapped_primitive_element() {
                    for ext in &pe.extensions {
                        if ext.url == ITEM_WEIGHT_EXTENSION_URL
                            || ext.url == ORDINAL_VALUE_EXTENSION_URL
                        {
                            let json = ext.to_json();
                            if let Some(val) = json.get("valueDecimal").and_then(|v| v.as_f64()) {
                                return Decimal::try_from(val).ok();
                            }
                            if let Some(val) = json.get("valueInteger").and_then(|v| v.as_i64()) {
                                return Some(Decimal::from(val));
                            }
                        }
                    }
                }
                None
            }
        }
    }

    /// Try to extract system and code from the value for terminology lookup
    fn extract_coding(value: &FhirPathValue) -> Option<(String, String)> {
        match value {
            FhirPathValue::Resource(json, _, _) => {
                let system = json.get("system").and_then(|s| s.as_str())?;
                let code = json.get("code").and_then(|c| c.as_str())?;
                Some((system.to_string(), code.to_string()))
            }
            FhirPathValue::String(code, _, _) => Some((String::new(), code.clone())),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for WeightFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "weight function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Process first item only (singleton)
        let item = &input[0];

        // 1. Try extension-based weight
        if let Some(weight) = Self::extract_weight_from_extensions(item) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::decimal(weight)),
            });
        }

        // 2. Fall back to terminology lookup for "weight" property
        if let Some(terminology) = context.terminology_provider()
            && let Some((system, code)) = Self::extract_coding(item)
            && !system.is_empty()
            && let Ok(lookup) = terminology
                .lookup_code(&system, &code, None, Some(vec!["weight"]))
                .await
        {
            for prop in &lookup.properties {
                if prop.code == "weight"
                    && let Ok(weight) = prop.value.parse::<Decimal>()
                {
                    return Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::decimal(weight)),
                    });
                }
            }
        }

        // No weight found
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
