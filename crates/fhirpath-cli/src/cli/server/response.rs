//! Standardized API response builder for FHIRPath Lab compliance
//!
//! This module provides a unified response format that matches the reference implementation
//! and ensures consistent API responses across all endpoints.

use crate::cli::server::models::*;
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Standard response builder for FHIRPath Lab API
pub struct StandardResponseBuilder {
    response: FhirPathLabResponse,
}

impl StandardResponseBuilder {
    /// Create a new standard response builder
    pub fn new() -> Self {
        Self {
            response: FhirPathLabResponse::new(),
        }
    }

    /// Add evaluation results to the response
    pub fn with_evaluation_results(mut self, results: &[JsonValue], expression: &str) -> Self {
        // Add the expression that was evaluated
        self.response
            .add_string_parameter("expression", expression.to_string());

        // Add evaluation results as proper "result" parameters with typed parts
        for result in results {
            self.add_result_parameter(result.clone());
        }

        self
    }

    /// Add timing information to the response
    pub fn with_timing(
        mut self,
        parse_time: Duration,
        eval_time: Duration,
        total_time: Duration,
    ) -> Self {
        #[allow(clippy::vec_init_then_push)]
        let mut timing_parts = Vec::new();

        timing_parts.push(FhirPathLabResponseParameter {
            name: "parse".to_string(),
            value_decimal: Some(parse_time.as_secs_f64() * 1000.0),
            ..Default::default()
        });

        timing_parts.push(FhirPathLabResponseParameter {
            name: "evaluation".to_string(),
            value_decimal: Some(eval_time.as_secs_f64() * 1000.0),
            ..Default::default()
        });

        timing_parts.push(FhirPathLabResponseParameter {
            name: "total".to_string(),
            value_decimal: Some(total_time.as_secs_f64() * 1000.0),
            ..Default::default()
        });

        self.response.add_complex_parameter("timing", timing_parts);
        self
    }

    /// Add trace information to the response
    pub fn with_trace(mut self, traces: Vec<String>) -> Self {
        if !traces.is_empty() {
            self.response.add_trace_from_collection(traces);
        }
        self
    }

    /// Add parse debug information
    pub fn with_parse_debug(mut self, parse_debug: String, parse_debug_tree: JsonValue) -> Self {
        self.response
            .add_ast_debug_info(parse_debug, parse_debug_tree);
        self
    }

    /// Add an error to the response
    pub fn with_error(
        mut self,
        error_code: &str,
        error_message: &str,
        details: Option<String>,
    ) -> Self {
        let mut error_parts = Vec::new();

        error_parts.push(FhirPathLabResponseParameter {
            name: "severity".to_string(),
            value_code: Some("error".to_string()),
            ..Default::default()
        });

        error_parts.push(FhirPathLabResponseParameter {
            name: "code".to_string(),
            value_code: Some(error_code.to_string()),
            ..Default::default()
        });

        error_parts.push(FhirPathLabResponseParameter {
            name: "message".to_string(),
            value_string: Some(error_message.to_string()),
            ..Default::default()
        });

        if let Some(details_text) = details {
            error_parts.push(FhirPathLabResponseParameter {
                name: "details".to_string(),
                value_string: Some(details_text),
                ..Default::default()
            });
        }

        self.response.add_complex_parameter("error", error_parts);
        self
    }

    /// Add metadata about the evaluation
    pub fn with_metadata(mut self, fhir_version: &str, engine_reused: bool) -> Self {
        #[allow(clippy::vec_init_then_push)]
        let mut metadata_parts = Vec::new();

        metadata_parts.push(FhirPathLabResponseParameter {
            name: "fhirVersion".to_string(),
            value_string: Some(fhir_version.to_string()),
            ..Default::default()
        });

        metadata_parts.push(FhirPathLabResponseParameter {
            name: "engineReused".to_string(),
            value_string: Some(engine_reused.to_string()),
            ..Default::default()
        });

        self.response
            .add_complex_parameter("metadata", metadata_parts);
        self
    }

    /// Add the "parameters" parameter with "evaluator" part (required by test runner)
    pub fn with_evaluator(mut self, evaluator_name: &str) -> Self {
        let evaluator_part = vec![FhirPathLabResponseParameter {
            name: "evaluator".to_string(),
            value_string: Some(evaluator_name.to_string()),
            ..Default::default()
        }];

        self.response
            .add_complex_parameter("parameters", evaluator_part);
        self
    }

    /// Build the final response
    pub fn build(self) -> FhirPathLabResponse {
        self.response
    }

    /// Add a result parameter with properly typed parts (expected by test runner)
    fn add_result_parameter(&mut self, value: JsonValue) {
        let result_parts = match &value {
            JsonValue::String(s) => {
                // Without concrete TypeInfo here, default to valueString. Code-like strings are
                // handled in higher-level handlers that have access to TypeInfo.
                vec![FhirPathLabResponseParameter {
                    name: "string".to_string(),
                    value_string: Some(s.clone()),
                    ..Default::default()
                }]
            }
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    vec![FhirPathLabResponseParameter {
                        name: "integer".to_string(),
                        value_integer: Some(i as i32),
                        ..Default::default()
                    }]
                } else {
                    vec![FhirPathLabResponseParameter {
                        name: "decimal".to_string(),
                        value_decimal: n.as_f64(),
                        ..Default::default()
                    }]
                }
            }
            JsonValue::Bool(b) => {
                vec![FhirPathLabResponseParameter {
                    name: "boolean".to_string(),
                    value_boolean: Some(*b),
                    ..Default::default()
                }]
            }
            JsonValue::Object(obj) => {
                // Heuristics for common FHIR complex types
                if self.is_fhir_human_name(obj) {
                    vec![FhirPathLabResponseParameter {
                        name: "humanName".to_string(),
                        value_human_name: Some(value.clone()),
                        ..Default::default()
                    }]
                } else if self.is_fhir_identifier(obj) {
                    vec![FhirPathLabResponseParameter {
                        name: "identifier".to_string(),
                        value_identifier: Some(value.clone()),
                        ..Default::default()
                    }]
                } else if self.is_fhir_address(obj) {
                    vec![FhirPathLabResponseParameter {
                        name: "address".to_string(),
                        value_address: Some(value.clone()),
                        ..Default::default()
                    }]
                } else if self.is_fhir_contact_point(obj) {
                    vec![FhirPathLabResponseParameter {
                        name: "contactPoint".to_string(),
                        value_contact_point: Some(value.clone()),
                        ..Default::default()
                    }]
                } else {
                    vec![FhirPathLabResponseParameter {
                        name: "resource".to_string(),
                        resource: Some(value.clone()),
                        ..Default::default()
                    }]
                }
            }
            _ => {
                // Arrays and other types as resources
                vec![FhirPathLabResponseParameter {
                    name: "resource".to_string(),
                    resource: Some(value.clone()),
                    ..Default::default()
                }]
            }
        };

        let result_param = FhirPathLabResponseParameter {
            name: "result".to_string(),
            part: Some(result_parts),
            ..Default::default()
        };

        self.response.parameter.push(result_param);
    }

    /// Check if a string looks like a FHIR code
    fn is_fhir_code(&self, s: &str) -> bool {
        // Simple heuristic: codes are typically short and contain no spaces
        s.len() < 50
            && !s.contains(' ')
            && s.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Check if an object is a FHIR HumanName
    fn is_fhir_human_name(&self, obj: &serde_json::Map<String, JsonValue>) -> bool {
        obj.contains_key("family") || obj.contains_key("given") || obj.contains_key("use")
    }

    /// Check if an object is a FHIR Identifier
    fn is_fhir_identifier(&self, obj: &serde_json::Map<String, JsonValue>) -> bool {
        obj.contains_key("system") || obj.contains_key("value") || obj.contains_key("use")
    }

    /// Check if an object is a FHIR Address
    fn is_fhir_address(&self, obj: &serde_json::Map<String, JsonValue>) -> bool {
        obj.contains_key("line")
            || obj.contains_key("city")
            || obj.contains_key("state")
            || obj.contains_key("postalCode")
    }

    /// Check if an object is a FHIR ContactPoint
    fn is_fhir_contact_point(&self, obj: &serde_json::Map<String, JsonValue>) -> bool {
        obj.contains_key("system") || obj.contains_key("value") || obj.contains_key("use")
    }
}

impl Default for StandardResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a standard success response
pub fn create_success_response() -> StandardResponseBuilder {
    StandardResponseBuilder::new()
}

/// Create a standard error response
pub fn create_error_response(
    error_code: &str,
    error_message: &str,
    details: Option<String>,
) -> StandardResponseBuilder {
    StandardResponseBuilder::new().with_error(error_code, error_message, details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_response_builder() {
        let response = StandardResponseBuilder::new()
            .with_evaluation_results(&[serde_json::json!("test")], "Patient.name")
            .with_timing(
                Duration::from_millis(10),
                Duration::from_millis(50),
                Duration::from_millis(60),
            )
            .with_metadata("r4", true)
            .build();

        assert_eq!(response.resource_type, "Parameters");
        assert!(!response.parameter.is_empty());
    }

    #[test]
    fn test_error_response() {
        let response = create_error_response(
            "PARSE_ERROR",
            "Invalid expression",
            Some("Syntax error at position 5".to_string()),
        )
        .build();

        assert_eq!(response.resource_type, "Parameters");

        // Should have an error parameter
        let error_param = response.parameter.iter().find(|p| p.name == "error");
        assert!(error_param.is_some());
    }
}
