use crate::cli::server::models::{
    ContextItem, ContextualResult, DecimalInput, EvaluationResultItem, EvaluationResultSet,
    Extension, ExtensionValue, Parameter, ParameterValue, ParametersResource, ParsedServerRequest,
    TraceOutput, TracePart, canonical_decimal_string, path_segments_to_string,
};
use octofhir_fhirpath::FhirPathValue;
use serde_json::{Number as JsonNumber, Value as JsonValue, json};

const JSON_VALUE_EXTENSION_URL: &str = "http://fhir.forms-lab.com/StructureDefinition/json-value";
const RESOURCE_PATH_EXTENSION_URL: &str =
    "http://fhir.forms-lab.com/StructureDefinition/resource-path";

pub struct ParseDebugInfo {
    pub summary: String,
    pub tree: String,
}

pub struct ResponseMetadata<'a> {
    pub evaluator_label: &'a str,
    pub expected_return_type: Option<String>,
    pub parse_debug: &'a ParseDebugInfo,
    pub semantic_diagnostics: &'a [octofhir_fhirpath::diagnostics::Diagnostic],
}

pub fn build_success_response(
    request: &ParsedServerRequest,
    evaluation: &EvaluationResultSet,
    metadata: ResponseMetadata,
) -> ParametersResource {
    let mut response = ParametersResource {
        resource_type: "Parameters".to_string(),
        id: Some("fhirpath".to_string()),
        parameter: Vec::new(),
    };

    let metadata_part = build_metadata_part(request, evaluation, metadata);
    response.parameter.push(metadata_part);

    for contextual in &evaluation.contexts {
        if let Some(result_parameter) = build_result_parameter(request, contextual) {
            response.parameter.push(result_parameter);
        }
    }

    response
}

fn build_metadata_part(
    request: &ParsedServerRequest,
    evaluation: &EvaluationResultSet,
    metadata: ResponseMetadata,
) -> Parameter {
    let mut parts = Vec::new();

    parts.push(make_string_part("evaluator", metadata.evaluator_label));

    if let Some(expected) = metadata.expected_return_type.as_deref() {
        parts.push(make_string_part("expectedReturnType", expected));
    }

    parts.push(make_string_part(
        "parseDebug",
        &metadata.parse_debug.summary,
    ));
    parts.push(make_string_part(
        "parseDebugTree",
        &metadata.parse_debug.tree,
    ));
    parts.push(make_string_part("expression", &request.expression));

    if let Some(context_expr) = &request.context
        && !context_expr.is_empty()
    {
        parts.push(make_string_part("context", context_expr));
    }

    parts.push(make_resource_part("resource", request.resource.clone()));

    if let Some(term) = &request.terminology_server
        && !term.is_empty()
    {
        parts.push(make_string_part("terminologyServerUrl", term));
    }

    if !request.variables.is_empty() {
        let variable_parts = request.variables.clone();
        parts.push(Parameter {
            name: "variables".to_string(),
            part: variable_parts,
            ..Parameter::empty()
        });
    }

    parts.push(build_timing_part(&evaluation.timing));

    // Add semantic diagnostics to the response
    for diagnostic in metadata.semantic_diagnostics {
        if matches!(
            diagnostic.severity,
            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
        ) {
            parts.push(make_string_part(
                "analysis-diagnostic",
                format!("Error: {}", diagnostic.message),
            ));
        } else if matches!(
            diagnostic.severity,
            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Warning
        ) {
            parts.push(make_string_part(
                "analysis-diagnostic",
                format!("Warning: {}", diagnostic.message),
            ));
        }
    }

    Parameter {
        name: "parameters".to_string(),
        part: parts,
        ..Parameter::empty()
    }
}

fn build_result_parameter(
    request: &ParsedServerRequest,
    contextual: &ContextualResult,
) -> Option<Parameter> {
    let mut parts = Vec::new();

    for result in &contextual.results {
        if let Some(parameter) = value_to_parameter(result, request) {
            parts.push(parameter);
        }
    }

    for trace in &contextual.traces {
        parts.push(trace_to_parameter(trace));
    }

    let path = contextual
        .context
        .path
        .clone()
        .or_else(|| context_path_from_segments(request, &contextual.context));

    Some(Parameter {
        name: "result".to_string(),
        value: ParameterValue {
            value_string: path,
            ..ParameterValue::default()
        },
        part: parts,
        ..Parameter::empty()
    })
}

fn value_to_parameter(
    result: &EvaluationResultItem,
    request: &ParsedServerRequest,
) -> Option<Parameter> {
    let datatype = result.datatype.clone();
    let mut parameter = parameter_from_value(&datatype, &result.value);

    if let Some(path) = result.path.as_deref() {
        attach_resource_path(&mut parameter, path);
    } else if !result.path_segments.is_empty() {
        let resource_type = request
            .resource
            .get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("Resource");
        let computed = path_segments_to_string(resource_type, &result.path_segments);
        attach_resource_path(&mut parameter, &computed);
    }

    Some(parameter)
}

fn attach_resource_path(parameter: &mut Parameter, path: &str) {
    parameter.extension.push(Extension {
        url: RESOURCE_PATH_EXTENSION_URL.to_string(),
        value: ExtensionValue {
            value_string: Some(path.to_string()),
            ..ExtensionValue::default()
        },
    });
    parameter.part.push(Parameter {
        name: "resource-path".to_string(),
        value: ParameterValue {
            value_string: Some(path.to_string()),
            ..ParameterValue::default()
        },
        ..Parameter::empty()
    });
}

fn trace_to_parameter(trace: &TraceOutput) -> Parameter {
    let parts: Vec<Parameter> = trace
        .parts
        .iter()
        .enumerate()
        .map(|(idx, part)| trace_part_to_parameter(idx, part))
        .collect();

    Parameter {
        name: "trace".to_string(),
        value: ParameterValue {
            value_string: Some(trace.name.clone()),
            ..ParameterValue::default()
        },
        part: parts,
        ..Parameter::empty()
    }
}

fn trace_part_to_parameter(index: usize, part: &TracePart) -> Parameter {
    let name = if part.datatype.is_empty() {
        format!("value{}", index)
    } else {
        part.datatype.clone()
    };

    match json_to_parameter_value(&part.value) {
        Some(mut parameter) => {
            parameter.name = name;
            parameter
        }
        None => Parameter {
            name,
            value: ParameterValue {
                value_string: Some(part.value.to_string()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
    }
}

fn build_timing_part(timing: &crate::cli::server::models::EvaluationTiming) -> Parameter {
    let mut parts = Vec::new();

    parts.push(make_decimal_part(
        "parseTime",
        timing.parse.as_secs_f64() * 1000.0,
    ));
    parts.push(make_decimal_part(
        "evaluationTime",
        timing.evaluation.as_secs_f64() * 1000.0,
    ));
    parts.push(make_decimal_part(
        "totalTime",
        timing.total.as_secs_f64() * 1000.0,
    ));

    Parameter {
        name: "timing".to_string(),
        part: parts,
        ..Parameter::empty()
    }
}

fn parameter_from_value(datatype: &str, value: &FhirPathValue) -> Parameter {
    match value {
        FhirPathValue::Boolean(b, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_boolean: Some(*b),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::Integer(i, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_integer: Some(*i),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::Decimal(d, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_decimal: Some(DecimalInput::String(canonical_decimal_string(d))),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::String(s, type_info, _) => {
            let field = type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_ascii_lowercase();
            string_parameter_for_type(datatype, &field, s)
        }
        FhirPathValue::Date(date, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_date: Some(date.to_string()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::DateTime(dt, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_date_time: Some(dt.to_string()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::Time(time, _, _) => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_time: Some(time.to_string()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
        FhirPathValue::Quantity {
            value: quantity_value,
            unit,
            code,
            system,
            ucum_unit: _,
            ..
        } => {
            let mut quantity = json!({
                "value": canonical_decimal_string(quantity_value),
            });
            if let Some(unit) = unit {
                quantity["unit"] = JsonValue::String(unit.clone());
            }
            if let Some(system) = system {
                quantity["system"] = JsonValue::String(system.clone());
            }
            if let Some(code) = code {
                quantity["code"] = JsonValue::String(code.clone());
            }
            Parameter {
                name: datatype.to_string(),
                value: ParameterValue {
                    value_quantity: Some(quantity),
                    ..ParameterValue::default()
                },
                ..Parameter::empty()
            }
        }
        FhirPathValue::Resource(json, type_info, _) => {
            let type_name = type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string();
            complex_parameter_for_type(datatype, &type_name, json.as_ref().clone())
        }
        FhirPathValue::Collection(_) | FhirPathValue::Empty => Parameter {
            name: datatype.to_string(),
            value: ParameterValue {
                value_string: Some("empty".to_string()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        },
    }
}

fn string_parameter_for_type(name: &str, type_hint: &str, value: &str) -> Parameter {
    let mut parameter = Parameter {
        name: name.to_string(),
        value: ParameterValue::default(),
        ..Parameter::empty()
    };

    match type_hint {
        "code" => parameter.value.value_code = Some(value.to_string()),
        "id" => parameter.value.value_id = Some(value.to_string()),
        "oid" => parameter.value.value_oid = Some(value.to_string()),
        "uuid" => parameter.value.value_uuid = Some(value.to_string()),
        "uri" => parameter.value.value_uri = Some(value.to_string()),
        "url" => parameter.value.value_url = Some(value.to_string()),
        "canonical" => parameter.value.value_canonical = Some(value.to_string()),
        "markdown" => parameter.value.value_markdown = Some(value.to_string()),
        "base64binary" => parameter.value.value_base64_binary = Some(value.to_string()),
        _ => parameter.value.value_string = Some(value.to_string()),
    }

    parameter
}

fn complex_parameter_for_type(name: &str, type_name: &str, value: JsonValue) -> Parameter {
    let mut parameter = Parameter {
        name: name.to_string(),
        value: ParameterValue::default(),
        ..Parameter::empty()
    };

    match type_name {
        "HumanName" => parameter.value.value_human_name = Some(value),
        "Identifier" => parameter.value.value_identifier = Some(value),
        "Address" => parameter.value.value_address = Some(value),
        "ContactPoint" => parameter.value.value_contact_point = Some(value),
        "Reference" => parameter.value.value_reference = Some(value),
        "Period" => parameter.value.value_period = Some(value),
        "Coding" => parameter.value.value_coding = Some(value),
        "CodeableConcept" => parameter.value.value_codeable_concept = Some(value),
        "Quantity" => parameter.value.value_quantity = Some(value),
        "ContactDetail" => parameter.value.value_contact_detail = Some(value),
        "Contributor" => parameter.value.value_contributor = Some(value),
        "Expression" => parameter.value.value_expression = Some(value),
        "ParameterDefinition" => parameter.value.value_parameter_definition = Some(value),
        "TriggerDefinition" => parameter.value.value_trigger_definition = Some(value),
        "DataRequirement" => parameter.value.value_data_requirement = Some(value),
        "Meta" => parameter.value.value_meta = Some(value),
        _ => {
            parameter.value.resource = Some(value.clone());
            parameter.extension.push(Extension {
                url: JSON_VALUE_EXTENSION_URL.to_string(),
                value: ExtensionValue {
                    value_string: Some(value.to_string()),
                    ..ExtensionValue::default()
                },
            });
        }
    }

    parameter
}

fn json_to_parameter_value(value: &JsonValue) -> Option<Parameter> {
    match value {
        JsonValue::String(text) => Some(Parameter {
            name: "string".to_string(),
            value: ParameterValue {
                value_string: Some(text.clone()),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        }),
        JsonValue::Bool(flag) => Some(Parameter {
            name: "boolean".to_string(),
            value: ParameterValue {
                value_boolean: Some(*flag),
                ..ParameterValue::default()
            },
            ..Parameter::empty()
        }),
        JsonValue::Number(number) => {
            if let Some(i) = number.as_i64() {
                Some(Parameter {
                    name: "integer".to_string(),
                    value: ParameterValue {
                        value_integer: Some(i),
                        ..ParameterValue::default()
                    },
                    ..Parameter::empty()
                })
            } else {
                number.as_f64().map(|f| make_decimal_part("decimal", f))
            }
        }
        JsonValue::Object(map) => {
            if let Some(type_name) = map
                .get("resourceType")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                Some(complex_parameter_for_type(
                    &type_name,
                    &type_name,
                    value.clone(),
                ))
            } else {
                Some(Parameter {
                    name: "json".to_string(),
                    value: ParameterValue {
                        resource: Some(json!({
                            "extension": [{
                                "url": JSON_VALUE_EXTENSION_URL,
                                "valueString": value.to_string()
                            }]
                        })),
                        ..ParameterValue::default()
                    },
                    ..Parameter::empty()
                })
            }
        }
        JsonValue::Array(_) | JsonValue::Null => None,
    }
}

fn make_string_part(name: &str, value: impl Into<String>) -> Parameter {
    Parameter {
        name: name.to_string(),
        value: ParameterValue {
            value_string: Some(value.into()),
            ..ParameterValue::default()
        },
        ..Parameter::empty()
    }
}

fn make_decimal_part(name: &str, value: f64) -> Parameter {
    Parameter {
        name: name.to_string(),
        value: ParameterValue {
            value_decimal: Some(DecimalInput::Number(
                JsonNumber::from_f64(value).unwrap_or_else(|| JsonNumber::from(0)),
            )),
            ..ParameterValue::default()
        },
        ..Parameter::empty()
    }
}

fn make_resource_part(name: &str, resource: JsonValue) -> Parameter {
    Parameter {
        name: name.to_string(),
        value: ParameterValue {
            resource: Some(resource),
            ..ParameterValue::default()
        },
        ..Parameter::empty()
    }
}

fn context_path_from_segments(
    request: &ParsedServerRequest,
    context: &ContextItem,
) -> Option<String> {
    if context.path_segments.is_empty() {
        return None;
    }

    let resource_type = request
        .resource
        .get("resourceType")
        .and_then(|v| v.as_str())
        .unwrap_or("Resource");

    Some(path_segments_to_string(
        resource_type,
        &context.path_segments,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::server::models::{EvaluationResultItem, PathSegment};
    use octofhir_fhirpath::core::FhirPathValue;
    use rust_decimal::Decimal;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn parameter_from_quantity_populates_value_quantity() {
        let quantity_value = Decimal::from_str("1.50").unwrap();
        let quantity = FhirPathValue::quantity_with_components(
            quantity_value,
            Some("mg".to_string()),
            Some("mg".to_string()),
            Some("http://unitsofmeasure.org".to_string()),
        );

        let parameter = parameter_from_value("Quantity", &quantity);
        let value = parameter.value.value_quantity.expect("quantity value");

        assert_eq!(
            value["value"],
            JsonValue::String(canonical_decimal_string(&quantity_value))
        );
        assert_eq!(value["unit"], JsonValue::String("mg".to_string()));
        assert_eq!(
            value["system"],
            JsonValue::String("http://unitsofmeasure.org".to_string())
        );
        assert_eq!(value["code"], JsonValue::String("mg".to_string()));
    }

    #[test]
    fn make_string_part_accepts_owned_and_borrowed() {
        let owned = make_string_part("expression", "Observation.value");
        assert_eq!(owned.name, "expression");
        assert_eq!(
            owned.value.value_string,
            Some("Observation.value".to_string())
        );

        let borrowed_input = String::from("Patient.name");
        let borrowed = make_string_part("context", borrowed_input.clone());
        assert_eq!(borrowed.name, "context");
        assert_eq!(borrowed.value.value_string, Some(borrowed_input));
    }

    #[test]
    fn string_parameter_supports_base64_binary() {
        let parameter = string_parameter_for_type("binary", "base64binary", "ZGF0YQ==");
        assert_eq!(parameter.name, "binary");
        assert_eq!(
            parameter.value.value_base64_binary,
            Some("ZGF0YQ==".to_string())
        );
    }

    #[test]
    fn value_parameter_includes_resource_path_part() {
        let request = ParsedServerRequest {
            expression: "name.given".to_string(),
            context: None,
            validate: false,
            resource: json!({ "resourceType": "Patient" }),
            variables: Vec::new(),
            terminology_server: None,
        };

        let result = EvaluationResultItem {
            value: FhirPathValue::string("John"),
            datatype: "string".to_string(),
            path: Some("Patient.name[0].given[0]".to_string()),
            path_segments: Vec::new(),
            index: 0,
        };

        let parameter = value_to_parameter(&result, &request).expect("parameter");
        assert_eq!(parameter.part.len(), 1);
        assert_eq!(parameter.part[0].name, "resource-path");
        assert_eq!(
            parameter.part[0].value.value_string,
            Some("Patient.name[0].given[0]".to_string())
        );
        assert!(
            parameter
                .extension
                .iter()
                .any(|ext| ext.url == RESOURCE_PATH_EXTENSION_URL)
        );
    }

    #[test]
    fn trace_to_parameter_preserves_parts() {
        let trace = TraceOutput {
            name: "eval".to_string(),
            parts: vec![TracePart {
                datatype: "string".to_string(),
                value: JsonValue::String("ok".to_string()),
            }],
        };

        let parameter = trace_to_parameter(&trace);
        assert_eq!(parameter.name, "trace");
        assert_eq!(parameter.value.value_string, Some("eval".to_string()));
        assert_eq!(parameter.part.len(), 1);
        assert_eq!(parameter.part[0].name, "string");
    }

    #[test]
    fn context_path_from_segments_returns_path() {
        let request = ParsedServerRequest {
            expression: "Patient.name".to_string(),
            context: None,
            validate: false,
            resource: json!({ "resourceType": "Patient" }),
            variables: Vec::new(),
            terminology_server: None,
        };

        let context = ContextItem {
            value: FhirPathValue::empty(),
            path: None,
            path_segments: vec![PathSegment::Property("name".to_string())],
            index: 0,
        };

        let path = context_path_from_segments(&request, &context);
        assert_eq!(path, Some("Patient.name".to_string()));
    }

    #[test]
    fn complex_parameter_handles_expression_type() {
        let payload = json!({
            "language": "text/fhirpath",
            "expression": "Patient.name"
        });

        let parameter = complex_parameter_for_type("Expression", "Expression", payload.clone());
        assert_eq!(parameter.name, "Expression");
        assert_eq!(parameter.value.value_expression, Some(payload));
    }

    #[test]
    fn complex_parameter_handles_contributor_type() {
        let contributor = json!({
            "type": "author",
            "name": "FHIRPath Maintainer"
        });

        let parameter =
            complex_parameter_for_type("Contributor", "Contributor", contributor.clone());
        assert_eq!(parameter.value.value_contributor, Some(contributor));
    }
}
