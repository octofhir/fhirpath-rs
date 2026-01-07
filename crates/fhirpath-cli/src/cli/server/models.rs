//! Data structures and helpers for the FHIRPath Lab server API.

use octofhir_fhir_model::ModelProvider;
use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

// -----------------------------------------------------------------------------
// Parameter resource structures
// -----------------------------------------------------------------------------

/// Representation of a FHIR Parameters resource used by the server API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametersResource {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub parameter: Vec<Parameter>,
}

/// Individual parameter entry within a Parameters resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub part: Vec<Parameter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension: Vec<Extension>,
    #[serde(flatten)]
    pub value: ParameterValue,
}

impl Parameter {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            part: Vec::new(),
            extension: Vec::new(),
            value: ParameterValue::default(),
        }
    }
}

/// Generic extension container used for JSON fallback and additional metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Extension {
    pub url: String,
    #[serde(flatten)]
    pub value: ExtensionValue,
}

/// Extension value payload (subset required by the server API).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionValue {
    #[serde(rename = "valueString", skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
    #[serde(rename = "valueBoolean", skip_serializing_if = "Option::is_none")]
    pub value_boolean: Option<bool>,
    #[serde(rename = "valueInteger", skip_serializing_if = "Option::is_none")]
    pub value_integer: Option<i64>,
    #[serde(rename = "valueDecimal", skip_serializing_if = "Option::is_none")]
    pub value_decimal: Option<DecimalInput>,
    #[serde(rename = "valueCode", skip_serializing_if = "Option::is_none")]
    pub value_code: Option<String>,
    #[serde(rename = "valueCanonical", skip_serializing_if = "Option::is_none")]
    pub value_canonical: Option<String>,
}

/// Supported value slots for Parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParameterValue {
    #[serde(rename = "valueString", skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
    #[serde(rename = "valueBoolean", skip_serializing_if = "Option::is_none")]
    pub value_boolean: Option<bool>,
    #[serde(rename = "valueInteger", skip_serializing_if = "Option::is_none")]
    pub value_integer: Option<i64>,
    #[serde(rename = "valuePositiveInt", skip_serializing_if = "Option::is_none")]
    pub value_positive_int: Option<i64>,
    #[serde(rename = "valueUnsignedInt", skip_serializing_if = "Option::is_none")]
    pub value_unsigned_int: Option<i64>,
    #[serde(rename = "valueBase64Binary", skip_serializing_if = "Option::is_none")]
    pub value_base64_binary: Option<String>,
    #[serde(rename = "valueDecimal", skip_serializing_if = "Option::is_none")]
    pub value_decimal: Option<DecimalInput>,
    #[serde(rename = "valueUri", skip_serializing_if = "Option::is_none")]
    pub value_uri: Option<String>,
    #[serde(rename = "valueUrl", skip_serializing_if = "Option::is_none")]
    pub value_url: Option<String>,
    #[serde(rename = "valueCanonical", skip_serializing_if = "Option::is_none")]
    pub value_canonical: Option<String>,
    #[serde(rename = "valueCode", skip_serializing_if = "Option::is_none")]
    pub value_code: Option<String>,
    #[serde(rename = "valueId", skip_serializing_if = "Option::is_none")]
    pub value_id: Option<String>,
    #[serde(rename = "valueOid", skip_serializing_if = "Option::is_none")]
    pub value_oid: Option<String>,
    #[serde(rename = "valueUuid", skip_serializing_if = "Option::is_none")]
    pub value_uuid: Option<String>,
    #[serde(rename = "valueMarkdown", skip_serializing_if = "Option::is_none")]
    pub value_markdown: Option<String>,
    #[serde(rename = "valueDate", skip_serializing_if = "Option::is_none")]
    pub value_date: Option<String>,
    #[serde(rename = "valueDateTime", skip_serializing_if = "Option::is_none")]
    pub value_date_time: Option<String>,
    #[serde(rename = "valueInstant", skip_serializing_if = "Option::is_none")]
    pub value_instant: Option<String>,
    #[serde(rename = "valueTime", skip_serializing_if = "Option::is_none")]
    pub value_time: Option<String>,
    #[serde(rename = "valueQuantity", skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<JsonValue>,
    #[serde(rename = "valueCoding", skip_serializing_if = "Option::is_none")]
    pub value_coding: Option<JsonValue>,
    #[serde(
        rename = "valueCodeableConcept",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_codeable_concept: Option<JsonValue>,
    #[serde(rename = "valueHumanName", skip_serializing_if = "Option::is_none")]
    pub value_human_name: Option<JsonValue>,
    #[serde(rename = "valueIdentifier", skip_serializing_if = "Option::is_none")]
    pub value_identifier: Option<JsonValue>,
    #[serde(rename = "valueReference", skip_serializing_if = "Option::is_none")]
    pub value_reference: Option<JsonValue>,
    #[serde(rename = "valueAddress", skip_serializing_if = "Option::is_none")]
    pub value_address: Option<JsonValue>,
    #[serde(rename = "valueContactPoint", skip_serializing_if = "Option::is_none")]
    pub value_contact_point: Option<JsonValue>,
    #[serde(rename = "valuePeriod", skip_serializing_if = "Option::is_none")]
    pub value_period: Option<JsonValue>,
    #[serde(rename = "valueAttachment", skip_serializing_if = "Option::is_none")]
    pub value_attachment: Option<JsonValue>,
    #[serde(rename = "valueSampledData", skip_serializing_if = "Option::is_none")]
    pub value_sampled_data: Option<JsonValue>,
    #[serde(rename = "valueSignature", skip_serializing_if = "Option::is_none")]
    pub value_signature: Option<JsonValue>,
    #[serde(rename = "valueAnnotation", skip_serializing_if = "Option::is_none")]
    pub value_annotation: Option<JsonValue>,
    #[serde(rename = "valueDosage", skip_serializing_if = "Option::is_none")]
    pub value_dosage: Option<JsonValue>,
    #[serde(rename = "valueContactDetail", skip_serializing_if = "Option::is_none")]
    pub value_contact_detail: Option<JsonValue>,
    #[serde(rename = "valueExpression", skip_serializing_if = "Option::is_none")]
    pub value_expression: Option<JsonValue>,
    #[serde(rename = "valueContributor", skip_serializing_if = "Option::is_none")]
    pub value_contributor: Option<JsonValue>,
    #[serde(
        rename = "valueParameterDefinition",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_parameter_definition: Option<JsonValue>,
    #[serde(
        rename = "valueTriggerDefinition",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_trigger_definition: Option<JsonValue>,
    #[serde(rename = "valueAge", skip_serializing_if = "Option::is_none")]
    pub value_age: Option<JsonValue>,
    #[serde(rename = "valueCount", skip_serializing_if = "Option::is_none")]
    pub value_count: Option<JsonValue>,
    #[serde(rename = "valueDistance", skip_serializing_if = "Option::is_none")]
    pub value_distance: Option<JsonValue>,
    #[serde(rename = "valueDuration", skip_serializing_if = "Option::is_none")]
    pub value_duration: Option<JsonValue>,
    #[serde(rename = "valueMoney", skip_serializing_if = "Option::is_none")]
    pub value_money: Option<JsonValue>,
    #[serde(rename = "valueRatio", skip_serializing_if = "Option::is_none")]
    pub value_ratio: Option<JsonValue>,
    #[serde(rename = "valueRange", skip_serializing_if = "Option::is_none")]
    pub value_range: Option<JsonValue>,
    #[serde(
        rename = "valueRelatedArtifact",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_related_artifact: Option<JsonValue>,
    #[serde(
        rename = "valueDataRequirement",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_data_requirement: Option<JsonValue>,
    #[serde(rename = "valueUsageContext", skip_serializing_if = "Option::is_none")]
    pub value_usage_context: Option<JsonValue>,
    #[serde(rename = "valueTiming", skip_serializing_if = "Option::is_none")]
    pub value_timing: Option<JsonValue>,
    #[serde(rename = "valueMeta", skip_serializing_if = "Option::is_none")]
    pub value_meta: Option<JsonValue>,
    #[serde(rename = "resource", skip_serializing_if = "Option::is_none")]
    pub resource: Option<JsonValue>,
}

impl ParameterValue {
    fn multiplicity(&self) -> ValueMultiplicity {
        let mut selected: Option<ValueKind> = None;

        macro_rules! visit {
            ($field:expr, $kind:expr) => {
                if $field.is_some() {
                    if selected.is_some() {
                        return ValueMultiplicity::Multiple;
                    }
                    selected = Some($kind);
                }
            };
        }

        visit!(self.value_string, ValueKind::String);
        visit!(self.value_boolean, ValueKind::Boolean);
        visit!(self.value_integer, ValueKind::Integer);
        visit!(self.value_positive_int, ValueKind::PositiveInt);
        visit!(self.value_unsigned_int, ValueKind::UnsignedInt);
        visit!(self.value_base64_binary, ValueKind::Base64Binary);
        visit!(self.value_decimal, ValueKind::Decimal);
        visit!(self.value_uri, ValueKind::Uri);
        visit!(self.value_url, ValueKind::Url);
        visit!(self.value_canonical, ValueKind::Canonical);
        visit!(self.value_code, ValueKind::Code);
        visit!(self.value_id, ValueKind::Id);
        visit!(self.value_oid, ValueKind::Oid);
        visit!(self.value_uuid, ValueKind::Uuid);
        visit!(self.value_markdown, ValueKind::Markdown);
        visit!(self.value_date, ValueKind::Date);
        visit!(self.value_date_time, ValueKind::DateTime);
        visit!(self.value_instant, ValueKind::Instant);
        visit!(self.value_time, ValueKind::Time);
        visit!(self.value_quantity, ValueKind::Quantity);
        visit!(self.value_coding, ValueKind::Coding);
        visit!(self.value_codeable_concept, ValueKind::CodeableConcept);
        visit!(self.value_human_name, ValueKind::HumanName);
        visit!(self.value_identifier, ValueKind::Identifier);
        visit!(self.value_reference, ValueKind::Reference);
        visit!(self.value_address, ValueKind::Address);
        visit!(self.value_contact_point, ValueKind::ContactPoint);
        visit!(self.value_period, ValueKind::Period);
        visit!(self.value_attachment, ValueKind::Attachment);
        visit!(self.value_sampled_data, ValueKind::SampledData);
        visit!(self.value_signature, ValueKind::Signature);
        visit!(self.value_annotation, ValueKind::Annotation);
        visit!(self.value_dosage, ValueKind::Dosage);
        visit!(self.value_contact_detail, ValueKind::ContactDetail);
        visit!(self.value_contributor, ValueKind::Contributor);
        visit!(self.value_expression, ValueKind::Expression);
        visit!(
            self.value_parameter_definition,
            ValueKind::ParameterDefinition
        );
        visit!(self.value_trigger_definition, ValueKind::TriggerDefinition);
        visit!(self.value_age, ValueKind::Age);
        visit!(self.value_count, ValueKind::Count);
        visit!(self.value_distance, ValueKind::Distance);
        visit!(self.value_duration, ValueKind::Duration);
        visit!(self.value_money, ValueKind::Money);
        visit!(self.value_ratio, ValueKind::Ratio);
        visit!(self.value_range, ValueKind::Range);
        visit!(self.value_related_artifact, ValueKind::RelatedArtifact);
        visit!(self.value_data_requirement, ValueKind::DataRequirement);
        visit!(self.value_usage_context, ValueKind::UsageContext);
        visit!(self.value_timing, ValueKind::Timing);
        visit!(self.value_meta, ValueKind::Meta);
        visit!(self.resource, ValueKind::Resource);

        match selected {
            Some(kind) => ValueMultiplicity::Single(kind),
            None => ValueMultiplicity::None,
        }
    }
}

// -----------------------------------------------------------------------------
// Request parsing
// -----------------------------------------------------------------------------

/// Normalised representation of an incoming server request.
#[derive(Debug, Clone)]
pub struct ParsedServerRequest {
    pub expression: String,
    pub context: Option<String>,
    pub validate: bool,
    pub resource: JsonValue,
    pub variables: Vec<Parameter>,
    pub terminology_server: Option<String>,
}

impl ParametersResource {
    pub fn parse_request(self) -> Result<ParsedServerRequest, RequestError> {
        if self.resource_type != "Parameters" {
            return Err(RequestError::InvalidResourceType(self.resource_type));
        }

        let mut expression: Option<String> = None;
        let mut context: Option<String> = None;
        let mut validate = false;
        let mut resource: Option<JsonValue> = None;
        let mut variables: Vec<Parameter> = Vec::new();
        let mut terminology_server: Option<String> = None;

        for mut param in self.parameter {
            match param.name.as_str() {
                "expression" => match param.value.multiplicity() {
                    ValueMultiplicity::Single(ValueKind::String) => {
                        expression = param.value.value_string.take();
                    }
                    ValueMultiplicity::None => {
                        return Err(RequestError::InvalidParameter {
                            name: "expression".to_string(),
                            message: "valueString is required".to_string(),
                        });
                    }
                    _ => {
                        return Err(RequestError::InvalidParameter {
                            name: "expression".to_string(),
                            message: "expression must be a single string value".to_string(),
                        });
                    }
                },
                "context" => match param.value.multiplicity() {
                    ValueMultiplicity::Single(ValueKind::String) => {
                        context = param.value.value_string.take();
                    }
                    ValueMultiplicity::None => {
                        context = None;
                    }
                    _ => {
                        return Err(RequestError::InvalidParameter {
                            name: "context".to_string(),
                            message: "context must be a single string value".to_string(),
                        });
                    }
                },
                "validate" => match param.value.multiplicity() {
                    ValueMultiplicity::Single(ValueKind::Boolean) => {
                        validate = param.value.value_boolean.unwrap_or(false);
                    }
                    ValueMultiplicity::None => {}
                    _ => {
                        return Err(RequestError::InvalidParameter {
                            name: "validate".to_string(),
                            message: "validate must be a boolean".to_string(),
                        });
                    }
                },
                "resource" => match param.value.multiplicity() {
                    ValueMultiplicity::Single(ValueKind::Resource) => {
                        resource = param.value.resource.take();
                    }
                    ValueMultiplicity::None => {
                        return Err(RequestError::InvalidParameter {
                            name: "resource".to_string(),
                            message: "resource payload is required".to_string(),
                        });
                    }
                    _ => {
                        return Err(RequestError::InvalidParameter {
                            name: "resource".to_string(),
                            message: "resource must contain a FHIR resource".to_string(),
                        });
                    }
                },
                "terminologyServer" | "terminologyserver" => match param.value.multiplicity() {
                    ValueMultiplicity::Single(ValueKind::String) => {
                        terminology_server = param.value.value_string.take();
                    }
                    ValueMultiplicity::None => {}
                    _ => {
                        return Err(RequestError::InvalidParameter {
                            name: "terminologyServer".to_string(),
                            message: "terminologyServer must be a string".to_string(),
                        });
                    }
                },
                "variables" => {
                    variables.extend(param.part.into_iter());
                }
                _ => {}
            }
        }

        let expression = expression.ok_or(RequestError::MissingParameter("expression"))?;
        let resource = resource.ok_or(RequestError::MissingParameter("resource"))?;

        Ok(ParsedServerRequest {
            expression,
            context,
            validate,
            resource,
            variables,
            terminology_server,
        })
    }
}

impl ParsedServerRequest {
    pub async fn variables_as_map(
        &self,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> Result<HashMap<String, FhirPathValue>, RequestError> {
        let mut map = HashMap::new();
        for parameter in &self.variables {
            let value = parameter.to_fhirpath_value(model_provider.clone()).await?;
            map.insert(parameter.name.clone(), value);
        }
        Ok(map)
    }
}

// -----------------------------------------------------------------------------
// Value conversion helpers
// -----------------------------------------------------------------------------

impl Parameter {
    pub async fn to_fhirpath_value(
        &self,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> Result<FhirPathValue, RequestError> {
        match self.value.multiplicity() {
            ValueMultiplicity::None => Ok(FhirPathValue::Empty),
            ValueMultiplicity::Single(kind) => match kind {
                ValueKind::String => Ok(FhirPathValue::string(
                    self.value.value_string.clone().unwrap_or_default(),
                )),
                ValueKind::Boolean => Ok(FhirPathValue::boolean(
                    self.value.value_boolean.unwrap_or(false),
                )),
                ValueKind::Integer => Ok(FhirPathValue::integer(
                    self.value.value_integer.unwrap_or_default(),
                )),
                ValueKind::PositiveInt => Ok(FhirPathValue::integer(
                    self.value.value_positive_int.unwrap_or_default(),
                )),
                ValueKind::UnsignedInt => Ok(FhirPathValue::integer(
                    self.value.value_unsigned_int.unwrap_or_default(),
                )),
                ValueKind::Base64Binary => Ok(FhirPathValue::string(
                    self.value.value_base64_binary.clone().unwrap_or_default(),
                )),
                ValueKind::Decimal => {
                    let decimal = self
                        .value
                        .value_decimal
                        .as_ref()
                        .ok_or_else(|| RequestError::invalid_value(self.name.clone()))?
                        .to_decimal()
                        .map_err(|err| RequestError::InvalidParameter {
                            name: self.name.clone(),
                            message: err,
                        })?;
                    Ok(FhirPathValue::decimal(decimal))
                }
                ValueKind::Date => {
                    let raw = self
                        .value
                        .value_date
                        .as_ref()
                        .ok_or_else(|| RequestError::invalid_value(self.name.clone()))?;
                    let parsed = PrecisionDate::parse(raw).ok_or_else(|| {
                        RequestError::InvalidParameter {
                            name: self.name.clone(),
                            message: format!("invalid date: {raw}"),
                        }
                    })?;
                    Ok(FhirPathValue::date(parsed))
                }
                ValueKind::DateTime | ValueKind::Instant => {
                    let raw = self
                        .value
                        .value_date_time
                        .as_ref()
                        .or(self.value.value_instant.as_ref())
                        .ok_or_else(|| RequestError::invalid_value(self.name.clone()))?;
                    let parsed = PrecisionDateTime::parse(raw).ok_or_else(|| {
                        RequestError::InvalidParameter {
                            name: self.name.clone(),
                            message: format!("invalid dateTime: {raw}"),
                        }
                    })?;
                    Ok(FhirPathValue::datetime(parsed))
                }
                ValueKind::Time => {
                    let raw = self
                        .value
                        .value_time
                        .as_ref()
                        .ok_or_else(|| RequestError::invalid_value(self.name.clone()))?;
                    let parsed = PrecisionTime::parse(raw).ok_or_else(|| {
                        RequestError::InvalidParameter {
                            name: self.name.clone(),
                            message: format!("invalid time: {raw}"),
                        }
                    })?;
                    Ok(FhirPathValue::time(parsed))
                }
                ValueKind::Resource => {
                    let json = self
                        .value
                        .resource
                        .clone()
                        .ok_or_else(|| RequestError::invalid_value(self.name.clone()))?;
                    if let Some(provider) = model_provider.clone()
                        && let Ok(value) = FhirPathValue::resource_with_model_provider(
                            json.clone(),
                            Some(provider),
                        )
                        .await
                    {
                        return Ok(value);
                    }
                    Ok(FhirPathValue::resource(json))
                }
                _ => {
                    // Treat complex datatypes as resource-like structures for now.
                    let json = self.to_json_value()?;
                    if let Some(provider) = model_provider
                        && let Ok(value) = FhirPathValue::resource_with_model_provider(
                            json.clone(),
                            Some(provider),
                        )
                        .await
                    {
                        return Ok(value);
                    }
                    Ok(FhirPathValue::resource(json))
                }
            },
            ValueMultiplicity::Multiple => Err(RequestError::InvalidParameter {
                name: self.name.clone(),
                message: "parameter contains multiple value fields".to_string(),
            }),
        }
    }

    pub fn to_json_value(&self) -> Result<JsonValue, RequestError> {
        // Serialize the parameter value back to JSON for complex datatypes.
        let mut map = JsonMap::new();
        match self.value.multiplicity() {
            ValueMultiplicity::Single(kind) => match kind {
                ValueKind::String => {
                    map.insert(
                        "valueString".to_string(),
                        json_string(self.value.value_string.clone()),
                    );
                }
                ValueKind::Boolean => {
                    map.insert(
                        "valueBoolean".to_string(),
                        JsonValue::Bool(self.value.value_boolean.unwrap_or(false)),
                    );
                }
                ValueKind::Integer => {
                    map.insert(
                        "valueInteger".to_string(),
                        JsonValue::Number(JsonNumber::from(self.value.value_integer.unwrap_or(0))),
                    );
                }
                ValueKind::PositiveInt => {
                    map.insert(
                        "valuePositiveInt".to_string(),
                        JsonValue::Number(JsonNumber::from(
                            self.value.value_positive_int.unwrap_or(0),
                        )),
                    );
                }
                ValueKind::UnsignedInt => {
                    map.insert(
                        "valueUnsignedInt".to_string(),
                        JsonValue::Number(JsonNumber::from(
                            self.value.value_unsigned_int.unwrap_or(0),
                        )),
                    );
                }
                ValueKind::Base64Binary => {
                    map.insert(
                        "valueBase64Binary".to_string(),
                        json_string(self.value.value_base64_binary.clone()),
                    );
                }
                ValueKind::Decimal => {
                    let repr = self
                        .value
                        .value_decimal
                        .as_ref()
                        .map(|d| d.to_canonical())
                        .unwrap_or_else(|| "0".to_string());
                    map.insert("valueDecimal".to_string(), JsonValue::String(repr));
                }
                ValueKind::Uri => {
                    map.insert(
                        "valueUri".to_string(),
                        json_string(self.value.value_uri.clone()),
                    );
                }
                ValueKind::Url => {
                    map.insert(
                        "valueUrl".to_string(),
                        json_string(self.value.value_url.clone()),
                    );
                }
                ValueKind::Canonical => {
                    map.insert(
                        "valueCanonical".to_string(),
                        json_string(self.value.value_canonical.clone()),
                    );
                }
                ValueKind::Code => {
                    map.insert(
                        "valueCode".to_string(),
                        json_string(self.value.value_code.clone()),
                    );
                }
                ValueKind::Id => {
                    map.insert(
                        "valueId".to_string(),
                        json_string(self.value.value_id.clone()),
                    );
                }
                ValueKind::Oid => {
                    map.insert(
                        "valueOid".to_string(),
                        json_string(self.value.value_oid.clone()),
                    );
                }
                ValueKind::Uuid => {
                    map.insert(
                        "valueUuid".to_string(),
                        json_string(self.value.value_uuid.clone()),
                    );
                }
                ValueKind::Markdown => {
                    map.insert(
                        "valueMarkdown".to_string(),
                        json_string(self.value.value_markdown.clone()),
                    );
                }
                ValueKind::Date => {
                    map.insert(
                        "valueDate".to_string(),
                        json_string(self.value.value_date.clone()),
                    );
                }
                ValueKind::DateTime => {
                    map.insert(
                        "valueDateTime".to_string(),
                        json_string(self.value.value_date_time.clone()),
                    );
                }
                ValueKind::Instant => {
                    map.insert(
                        "valueInstant".to_string(),
                        json_string(self.value.value_instant.clone()),
                    );
                }
                ValueKind::Time => {
                    map.insert(
                        "valueTime".to_string(),
                        json_string(self.value.value_time.clone()),
                    );
                }
                ValueKind::Quantity => {
                    map.insert(
                        "valueQuantity".to_string(),
                        self.value.value_quantity.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Coding => {
                    map.insert(
                        "valueCoding".to_string(),
                        self.value.value_coding.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::CodeableConcept => {
                    map.insert(
                        "valueCodeableConcept".to_string(),
                        self.value
                            .value_codeable_concept
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::HumanName => {
                    map.insert(
                        "valueHumanName".to_string(),
                        self.value
                            .value_human_name
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Identifier => {
                    map.insert(
                        "valueIdentifier".to_string(),
                        self.value
                            .value_identifier
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Reference => {
                    map.insert(
                        "valueReference".to_string(),
                        self.value
                            .value_reference
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Address => {
                    map.insert(
                        "valueAddress".to_string(),
                        self.value.value_address.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::ContactPoint => {
                    map.insert(
                        "valueContactPoint".to_string(),
                        self.value
                            .value_contact_point
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Period => {
                    map.insert(
                        "valuePeriod".to_string(),
                        self.value.value_period.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Attachment => {
                    map.insert(
                        "valueAttachment".to_string(),
                        self.value
                            .value_attachment
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::SampledData => {
                    map.insert(
                        "valueSampledData".to_string(),
                        self.value
                            .value_sampled_data
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Signature => {
                    map.insert(
                        "valueSignature".to_string(),
                        self.value
                            .value_signature
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Annotation => {
                    map.insert(
                        "valueAnnotation".to_string(),
                        self.value
                            .value_annotation
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Dosage => {
                    map.insert(
                        "valueDosage".to_string(),
                        self.value.value_dosage.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::ContactDetail => {
                    map.insert(
                        "valueContactDetail".to_string(),
                        self.value
                            .value_contact_detail
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Contributor => {
                    map.insert(
                        "valueContributor".to_string(),
                        self.value
                            .value_contributor
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Expression => {
                    map.insert(
                        "valueExpression".to_string(),
                        self.value
                            .value_expression
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::ParameterDefinition => {
                    map.insert(
                        "valueParameterDefinition".to_string(),
                        self.value
                            .value_parameter_definition
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::TriggerDefinition => {
                    map.insert(
                        "valueTriggerDefinition".to_string(),
                        self.value
                            .value_trigger_definition
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Age => {
                    map.insert(
                        "valueAge".to_string(),
                        self.value.value_age.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Count => {
                    map.insert(
                        "valueCount".to_string(),
                        self.value.value_count.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Distance => {
                    map.insert(
                        "valueDistance".to_string(),
                        self.value.value_distance.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Duration => {
                    map.insert(
                        "valueDuration".to_string(),
                        self.value.value_duration.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Money => {
                    map.insert(
                        "valueMoney".to_string(),
                        self.value.value_money.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Ratio => {
                    map.insert(
                        "valueRatio".to_string(),
                        self.value.value_ratio.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Range => {
                    map.insert(
                        "valueRange".to_string(),
                        self.value.value_range.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::RelatedArtifact => {
                    map.insert(
                        "valueRelatedArtifact".to_string(),
                        self.value
                            .value_related_artifact
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::DataRequirement => {
                    map.insert(
                        "valueDataRequirement".to_string(),
                        self.value
                            .value_data_requirement
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::UsageContext => {
                    map.insert(
                        "valueUsageContext".to_string(),
                        self.value
                            .value_usage_context
                            .clone()
                            .unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Timing => {
                    map.insert(
                        "valueTiming".to_string(),
                        self.value.value_timing.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Meta => {
                    map.insert(
                        "valueMeta".to_string(),
                        self.value.value_meta.clone().unwrap_or(JsonValue::Null),
                    );
                }
                ValueKind::Resource => {
                    map.insert(
                        "resource".to_string(),
                        self.value.resource.clone().unwrap_or(JsonValue::Null),
                    );
                }
            },
            ValueMultiplicity::None => {}
            ValueMultiplicity::Multiple => {
                return Err(RequestError::InvalidParameter {
                    name: self.name.clone(),
                    message: "parameter contains multiple value fields".to_string(),
                });
            }
        }

        if map.is_empty() {
            Ok(JsonValue::Null)
        } else {
            Ok(JsonValue::Object(map))
        }
    }
}

fn json_string(value: Option<String>) -> JsonValue {
    match value {
        Some(v) => JsonValue::String(v),
        None => JsonValue::Null,
    }
}

// -----------------------------------------------------------------------------
// Evaluation result metadata (populated by handlers/results modules)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TraceOutput {
    pub name: String,
    pub parts: Vec<TracePart>,
}

#[derive(Debug, Clone)]
pub struct TracePart {
    pub datatype: String,
    pub value: JsonValue,
}

#[derive(Debug, Clone)]
pub struct EvaluationTiming {
    pub parse: std::time::Duration,
    pub evaluation: std::time::Duration,
    pub total: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ContextEvaluationInfo {
    pub context_expression: Option<String>,
    pub context_item_count: usize,
    pub context_success: bool,
}

#[derive(Debug, Clone)]
pub struct ContextItem {
    pub value: FhirPathValue,
    pub path: Option<String>,
    pub path_segments: Vec<PathSegment>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct EvaluationResultItem {
    pub value: FhirPathValue,
    pub datatype: String,
    pub path: Option<String>,
    pub path_segments: Vec<PathSegment>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ContextualResult {
    pub context: ContextItem,
    pub results: Vec<EvaluationResultItem>,
    pub traces: Vec<TraceOutput>,
}

#[derive(Debug, Clone)]
pub struct EvaluationResultSet {
    pub context_info: ContextEvaluationInfo,
    pub contexts: Vec<ContextualResult>,
    pub timing: EvaluationTiming,
}

// -----------------------------------------------------------------------------
// Decimal helpers
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecimalInput {
    Number(JsonNumber),
    String(String),
}

impl DecimalInput {
    pub fn to_decimal(&self) -> Result<Decimal, String> {
        let text = match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.clone(),
        };
        Decimal::from_str_exact(&text).map_err(|err| err.to_string())
    }

    pub fn to_canonical(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.clone(),
        }
    }
}

/// Representation for FHIR decimal values in responses.
#[derive(Debug, Clone)]
pub enum DecimalRepresentation {
    Float(f64),
    Canonical(String),
}

impl DecimalRepresentation {
    pub fn from_decimal(decimal: &Decimal) -> Self {
        Self::Canonical(canonical_decimal_string(decimal))
    }

    pub fn from_f64(value: f64) -> Self {
        Self::Float(value)
    }
}

impl Serialize for DecimalRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Float(value) => serializer.serialize_f64(*value),
            Self::Canonical(text) => serializer.serialize_str(text),
        }
    }
}

pub fn canonical_decimal_string(decimal: &Decimal) -> String {
    let mut text = decimal.to_string();
    if text.contains(',') {
        text = text.replace(',', ".");
    }
    text
}

// -----------------------------------------------------------------------------
// Request errors
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum RequestError {
    InvalidResourceType(String),
    MissingParameter(&'static str),
    InvalidParameter { name: String, message: String },
}

impl RequestError {
    fn invalid_value(name: String) -> Self {
        Self::InvalidParameter {
            name,
            message: "parameter value missing".to_string(),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResourceType(rt) => {
                write!(f, "Invalid resourceType '{rt}', expected Parameters")
            }
            Self::MissingParameter(name) => {
                write!(f, "Missing required parameter '{name}'")
            }
            Self::InvalidParameter { name, message } => {
                write!(f, "Invalid parameter '{name}': {message}")
            }
        }
    }
}

impl std::error::Error for RequestError {}

// -----------------------------------------------------------------------------
// Value kind helpers
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    String,
    Boolean,
    Integer,
    PositiveInt,
    UnsignedInt,
    Base64Binary,
    Decimal,
    Uri,
    Url,
    Canonical,
    Code,
    Id,
    Oid,
    Uuid,
    Markdown,
    Date,
    DateTime,
    Instant,
    Time,
    Quantity,
    Coding,
    CodeableConcept,
    HumanName,
    Identifier,
    Reference,
    Address,
    ContactPoint,
    Period,
    Attachment,
    SampledData,
    Signature,
    Annotation,
    Dosage,
    ContactDetail,
    Contributor,
    Expression,
    ParameterDefinition,
    TriggerDefinition,
    Age,
    Count,
    Distance,
    Duration,
    Money,
    Ratio,
    Range,
    RelatedArtifact,
    DataRequirement,
    UsageContext,
    Timing,
    Meta,
    Resource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueMultiplicity {
    None,
    Single(ValueKind),
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Property(String),
    Index(usize),
}

// -----------------------------------------------------------------------------
// JSON fallbacks for trace output
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct OperationOutcome {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub issue: Vec<OperationOutcomeIssue>,
}

#[derive(Debug, Serialize)]
pub struct OperationOutcomeIssue {
    pub severity: String,
    pub code: String,
    pub details: OperationOutcomeDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OperationOutcomeDetails {
    pub text: String,
}

impl OperationOutcome {
    pub fn error(code: &str, message: &str, diagnostics: Option<String>) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: code.to_string(),
                details: OperationOutcomeDetails {
                    text: message.to_string(),
                },
                diagnostics,
            }],
        }
    }
}

pub fn path_segments_to_string(root: &str, segments: &[PathSegment]) -> String {
    if segments.is_empty() {
        return root.to_string();
    }

    let mut result = root.to_string();
    let mut pending_property: Option<&str> = None;

    for segment in segments {
        match segment {
            PathSegment::Property(name) => {
                if let Some(prop) = pending_property.take() {
                    result.push('.');
                    result.push_str(prop);
                }
                pending_property = Some(name);
            }
            PathSegment::Index(idx) => {
                if let Some(prop) = pending_property.take() {
                    result.push('.');
                    result.push_str(prop);
                }
                result.push('[');
                result.push_str(&idx.to_string());
                result.push(']');
            }
        }
    }

    if let Some(prop) = pending_property {
        result.push('.');
        result.push_str(prop);
    }

    result
}

pub fn fhir_value_to_json(value: FhirPathValue) -> JsonValue {
    match value {
        FhirPathValue::Boolean(v, _, _) => JsonValue::Bool(v),
        FhirPathValue::Integer(v, _, _) => JsonValue::Number(JsonNumber::from(v)),
        FhirPathValue::Decimal(d, _, _) => JsonValue::String(canonical_decimal_string(&d)),
        FhirPathValue::String(text, _, _) => JsonValue::String(text),
        FhirPathValue::Date(date, _, _) => JsonValue::String(date.to_string()),
        FhirPathValue::DateTime(dt, _, _) => JsonValue::String(dt.to_string()),
        FhirPathValue::Time(time, _, _) => JsonValue::String(time.to_string()),
        FhirPathValue::Quantity { value, unit, .. } => {
            let mut map = JsonMap::new();
            map.insert(
                "value".to_string(),
                JsonValue::String(canonical_decimal_string(&value)),
            );
            if let Some(unit_text) = unit {
                map.insert("unit".to_string(), JsonValue::String(unit_text));
            }
            JsonValue::Object(map)
        }
        FhirPathValue::Resource(json, _, _) => (*json).clone(),
        FhirPathValue::Collection(collection) => {
            JsonValue::Array(collection.iter().cloned().map(fhir_value_to_json).collect())
        }
        FhirPathValue::Empty => JsonValue::Null,
    }
}
