// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utility functions for JSON conversion and compatibility
//!
//! This module provides high-level utility functions for converting between
//! different JSON representations, making it easy for users to integrate
//! the FHIRPath engine with existing code.

use crate::FhirPathValue;
use octofhir_ucum::precision::NumericOps;

/// High-level JSON parsing
///
/// This function uses serde_json for JSON parsing.
/// The resulting JsonValue can be used directly with the FHIRPath engine.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"resourceType": "Patient", "id": "123"}"#;
/// let json_value = utils::parse_json(json_str).unwrap();
/// ```
pub fn parse_json(input: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(input).map_err(|e| format!("JSON parsing error: {e}"))
}

/// Convert FhirPathValue to serde_json::Value for JSON processing
///
/// This provides direct access to the underlying serde_json value for
/// users who want to work with standard JSON.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::{utils, FhirPathValue};
///
/// let value = FhirPathValue::String("hello".into());
/// let json_value = utils::to_json(value).unwrap();
/// ```
pub fn to_json(value: FhirPathValue) -> Result<serde_json::Value, String> {
    match value {
        FhirPathValue::JsonValue(json_val) => Ok(json_val.clone()),
        FhirPathValue::String(s) => Ok(serde_json::Value::String(s)),
        FhirPathValue::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        FhirPathValue::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
        FhirPathValue::Decimal(d) => {
            let f64_val = d.to_f64();
            Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(f64_val).unwrap_or(serde_json::Number::from(0)),
            ))
        }
        _ => Ok(serde_json::Value::Null),
    }
}

/// Convert serde_json::Value to FhirPathValue
///
/// This creates a FhirPathValue from a serde_json::Value, enabling
/// users to use serde_json parsing with FHIRPath evaluation.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"name": "John"}"#;
/// let json_value = serde_json::from_str(json_str).unwrap();
/// let fhir_value = utils::from_json(json_value);
/// ```
pub fn from_json(value: serde_json::Value) -> FhirPathValue {
    FhirPathValue::JsonValue(value)
}

/// Convert between JSON string representations
///
/// This function parses JSON using serde_json and can optionally convert
/// for pretty printing.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let input = r#"{"name":"John","age":30}"#;
/// let pretty = utils::reformat_json(input, true).unwrap();
/// ```
pub fn reformat_json(input: &str, pretty: bool) -> Result<String, String> {
    let json_value = parse_json(input)?;
    if pretty {
        serde_json::to_string_pretty(&json_value).map_err(|e| format!("JSON formatting error: {e}"))
    } else {
        serde_json::to_string(&json_value).map_err(|e| format!("JSON formatting error: {e}"))
    }
}

/// Parse JSON and create a FhirPathValue in one step
///
/// This is a convenience function that combines JSON parsing with
/// FhirPathValue creation.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"resourceType": "Patient", "id": "123"}"#;
/// let fhir_value = utils::parse_as_fhir_value(json_str).unwrap();
/// ```
pub fn parse_as_fhir_value(input: &str) -> Result<FhirPathValue, String> {
    let json_value: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON parsing error: {e}"))?;
    Ok(FhirPathValue::JsonValue(json_value))
}

/// Convert serde_json::Value to FhirPathValue
///
/// This function converts from serde_json::Value to FhirPathValue
/// directly.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let serde_value: serde_json::Value = serde_json::json!({"resourceType": "Patient", "id": "123"});
/// let fhir_value = utils::serde_to_fhir_value(&serde_value).unwrap();
/// ```
pub fn serde_to_fhir_value(value: &serde_json::Value) -> Result<FhirPathValue, String> {
    Ok(from_json(value.clone()))
}

/// Convert FhirPathValue to serde_json::Value
///
/// This function converts from FhirPathValue to serde_json::Value
/// for users who need to integrate with existing serde_json-based code.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::{utils, FhirPathValue};
///
/// let fhir_value = FhirPathValue::String("hello".into());
/// let serde_value = utils::fhir_value_to_serde(&fhir_value).unwrap();
/// ```
pub fn fhir_value_to_serde(value: &FhirPathValue) -> Result<serde_json::Value, String> {
    to_json(value.clone())
}

/// Parse JSON string using serde_json and convert to FhirPathValue
///
/// This function is for users who prefer serde_json parsing and want
/// to use the result with FHIRPath evaluation.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"resourceType": "Patient", "id": "123"}"#;
/// let fhir_value = utils::parse_with_serde(json_str).unwrap();
/// ```
pub fn parse_with_serde(input: &str) -> Result<FhirPathValue, String> {
    let serde_value: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("Serde JSON parsing error: {e}"))?;
    serde_to_fhir_value(&serde_value)
}

/// Convenience type alias for JSON conversion results
pub type JsonResult<T> = Result<T, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let result = parse_json(json_str);
        assert!(result.is_ok());

        let json_value = result.unwrap();
        assert!(json_value.is_object());
        assert_eq!(
            json_value.get_property("name").unwrap().as_str(),
            Some("John")
        );
    }

    #[test]
    fn test_json_conversion() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let json_value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let fhir_value = from_json(json_value.clone());
        let back_to_json = to_json(fhir_value).unwrap();

        assert_eq!(back_to_json, json_value);
    }

    #[test]
    fn test_parse_as_fhir_value() {
        let json_str = r#"{"resourceType": "Patient", "id": "123"}"#;
        let fhir_value = parse_as_fhir_value(json_str).unwrap();

        match fhir_value {
            FhirPathValue::JsonValue(json_val) => {
                assert_eq!(
                    json_val.get_property("resourceType").unwrap().as_str(),
                    Some("Patient")
                );
                assert_eq!(json_val.get_property("id").unwrap().as_str(), Some("123"));
            }
            _ => panic!("Expected JsonValue"),
        }
    }

    #[test]
    fn test_reformat_json() {
        let compact = r#"{"name":"John","age":30}"#;
        let pretty = reformat_json(compact, true).unwrap();

        // Should be formatted nicely
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));

        let back_to_compact = reformat_json(&pretty, false).unwrap();
        let parsed_original = serde_json::from_str::<serde_json::Value>(compact).unwrap();
        let parsed_reformatted =
            serde_json::from_str::<serde_json::Value>(&back_to_compact).unwrap();
        assert_eq!(parsed_original, parsed_reformatted);
    }

    #[test]
    fn test_serde_to_fhir_value_conversion() {
        let serde_value: serde_json::Value = serde_json::json!({
            "resourceType": "Patient",
            "id": "test-123",
            "active": true
        });

        let fhir_value = serde_to_fhir_value(&serde_value).unwrap();

        match fhir_value {
            FhirPathValue::JsonValue(json_val) => {
                assert_eq!(
                    json_val.get_property("resourceType").unwrap().as_str(),
                    Some("Patient")
                );
                assert_eq!(
                    json_val.get_property("id").unwrap().as_str(),
                    Some("test-123")
                );
                assert_eq!(
                    json_val.get_property("active").unwrap().as_bool(),
                    Some(true)
                );
            }
            _ => panic!("Expected JsonValue"),
        }
    }

    #[test]
    fn test_fhir_value_to_serde_conversion() {
        let json_str = r#"{"resourceType":"Observation","status":"final","code":{"text":"Test"}}"#;
        let fhir_value = parse_as_fhir_value(json_str).unwrap();

        let serde_value = fhir_value_to_serde(&fhir_value).unwrap();

        assert_eq!(serde_value["resourceType"].as_str(), Some("Observation"));
        assert_eq!(serde_value["status"].as_str(), Some("final"));
        assert_eq!(serde_value["code"]["text"].as_str(), Some("Test"));
    }

    #[test]
    fn test_parse_with_serde() {
        let json_str = r#"{"resourceType":"Patient","name":[{"given":["Jane"],"family":"Smith"}]}"#;
        let fhir_value = parse_with_serde(json_str).unwrap();

        match fhir_value {
            FhirPathValue::JsonValue(json_val) => {
                assert_eq!(
                    json_val.get_property("resourceType").unwrap().as_str(),
                    Some("Patient")
                );
                let name_json = json_val.get_property("name").unwrap();
                let name_json_value = name_json.as_value();
                let name_array = name_json_value.as_array().unwrap();
                assert_eq!(name_array.len(), 1);
                let first_name_json = &name_array[0];
                let first_name = first_name_json.clone();
                assert_eq!(
                    first_name.get_property("family").unwrap().as_str(),
                    Some("Smith")
                );
            }
            _ => panic!("Expected JsonValue"),
        }
    }
}
