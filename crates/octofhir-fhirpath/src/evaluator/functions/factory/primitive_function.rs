//! `%factory.{primitive}(value, extensions)`: a FHIR primitive with an optional value and
//! extensions, e.g. `%factory.code('final')` (FHIR API Incubator, `%factory`).

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{
    Collection, FhirPathError, FhirPathValue, Result, WrappedExtension, WrappedPrimitiveElement,
};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// The FHIR primitive types `%factory` can create.
pub const FHIR_PRIMITIVES: &[&str] = &[
    "base64Binary",
    "boolean",
    "canonical",
    "code",
    "date",
    "dateTime",
    "decimal",
    "id",
    "instant",
    "integer",
    "integer64",
    "markdown",
    "oid",
    "positiveInt",
    "string",
    "time",
    "unsignedInt",
    "uri",
    "url",
    "uuid",
    "xhtml",
];

/// `%factory.{primitive}(value, extensions)` function evaluator, one per primitive type.
pub struct FactoryPrimitiveFunctionEvaluator {
    type_name: &'static str,
    metadata: FunctionMetadata,
}

impl FactoryPrimitiveFunctionEvaluator {
    pub fn create(type_name: &'static str) -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            type_name,
            metadata: FunctionMetadata {
                name: type_name.to_string(),
                description: format!("Creates a FHIR {type_name} with a value and extensions"),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "The value".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "extensions".to_string(),
                            parameter_type: vec!["Extension".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Extensions on the primitive".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: type_name.to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(2),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::None,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// The value as this primitive type: strings convert to booleans, numbers and
    /// dates/times as needed.
    fn convert(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        let text = || match value {
            FhirPathValue::String(s, _, _) => Some(s.as_str()),
            _ => None,
        };
        match self.type_name {
            "boolean" => match value {
                FhirPathValue::Boolean(..) => Some(value.clone()),
                _ => match text()? {
                    "true" => Some(FhirPathValue::boolean(true)),
                    "false" => Some(FhirPathValue::boolean(false)),
                    _ => None,
                },
            },
            "integer" | "positiveInt" | "unsignedInt" | "integer64" => match value {
                FhirPathValue::Integer(..) => Some(value.clone()),
                _ => text()?.parse::<i64>().ok().map(FhirPathValue::integer),
            },
            "decimal" => match value {
                FhirPathValue::Decimal(..) => Some(value.clone()),
                FhirPathValue::Integer(i, _, _) => Some(FhirPathValue::decimal(*i)),
                _ => crate::core::value::utils::decimal_from_text(text()?)
                    .map(FhirPathValue::decimal),
            },
            "date" => match value {
                FhirPathValue::Date(..) => Some(value.clone()),
                _ => PrecisionDate::parse(text()?).map(FhirPathValue::date),
            },
            "dateTime" | "instant" => match value {
                FhirPathValue::DateTime(..) => Some(value.clone()),
                _ => PrecisionDateTime::parse(text()?).map(FhirPathValue::datetime),
            },
            "time" => match value {
                FhirPathValue::Time(..) => Some(value.clone()),
                _ => PrecisionTime::parse(text()?).map(FhirPathValue::time),
            },
            _ => text().map(|s| FhirPathValue::string(s.to_string())),
        }
    }
}

/// An Extension resource as a primitive's extension.
fn wrapped_extension(value: &FhirPathValue) -> Option<WrappedExtension> {
    let FhirPathValue::Resource(node, _, _) = value else {
        return None;
    };
    let json = node.to_json();
    let url = json.get("url")?.as_str()?.to_string();
    let value_field = json
        .as_object()?
        .iter()
        .find(|(key, _)| key.starts_with("value"))
        .map(|(key, value)| (key.clone(), value.clone()));
    Some(match value_field {
        Some((key, value)) => WrappedExtension::with_value(url, key, value),
        None => WrappedExtension::new(url),
    })
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryPrimitiveFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                format!(
                    "{} function can only be called on %factory variable",
                    self.type_name
                ),
            ));
        }
        let value = match args.first() {
            Some(arg) if arg.len() > 1 => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    format!("{} takes a single value", self.type_name),
                ));
            }
            Some(arg) => match arg.first() {
                Some(value) => Some(self.convert(value).ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        format!(
                            "Cannot make a {} from {}",
                            self.type_name,
                            value.type_name()
                        ),
                    )
                })?),
                None => None,
            },
            None => None,
        };
        let extensions: Vec<WrappedExtension> = args
            .get(1)
            .map(|arg| arg.iter().filter_map(wrapped_extension).collect())
            .unwrap_or_default();

        let type_info = Arc::new(TypeInfo {
            type_name: self.type_name.to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some(self.type_name.to_string()),
        });
        let element = (!extensions.is_empty()).then(|| {
            let mut element = WrappedPrimitiveElement::new();
            element.extensions = extensions;
            element
        });
        // Without a value there is nothing to carry the extensions: the result is empty.
        let Some(value) = value else {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        };
        let result = FhirPathValue::wrap_value(value, type_info, element);
        Ok(EvaluationResult {
            value: Collection::single(result),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
