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

use crate::{FhirPathValue, JsonValue};

/// High-level JSON parsing with automatic optimization
///
/// This function uses sonic-rs for maximum performance by default.
/// The resulting JsonValue can be used directly with the FHIRPath engine.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"resourceType": "Patient", "id": "123"}"#;
/// let json_value = utils::parse_json(json_str).unwrap();
/// ```
pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    JsonValue::parse(input).map_err(|e| format!("JSON parsing error: {e}"))
}

/// Convert FhirPathValue to sonic_rs::Value for high-performance processing
///
/// This provides direct access to the underlying sonic-rs value for
/// users who want maximum performance.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::{utils, FhirPathValue};
///
/// let value = FhirPathValue::String("hello".into());
/// let sonic_value = utils::to_sonic(value).unwrap();
/// ```
pub fn to_sonic(value: FhirPathValue) -> Result<sonic_rs::Value, String> {
    match value {
        FhirPathValue::JsonValue(json_val) => Ok(json_val.as_sonic_value().clone()),
        _ => {
            // Convert to JSON string and parse with sonic-rs
            let json_str = value.to_sonic_value()?.to_string();
            sonic_rs::from_str(&json_str).map_err(|e| format!("Sonic parse error: {e}"))
        }
    }
}

/// Convert sonic_rs::Value to FhirPathValue
///
/// This creates a FhirPathValue from a sonic_rs::Value, enabling
/// users to leverage sonic-rs parsing with FHIRPath evaluation.
///
/// # Example
/// ```rust
/// use octofhir_fhirpath::utils;
///
/// let json_str = r#"{"name": "John"}"#;
/// let sonic_value = sonic_rs::from_str(json_str).unwrap();
/// let fhir_value = utils::from_sonic(sonic_value);
/// ```
pub fn from_sonic(value: sonic_rs::Value) -> FhirPathValue {
    FhirPathValue::JsonValue(octofhir_fhirpath_model::JsonValue::new(value))
}

/// Convert between JSON string representations with automatic optimization
///
/// This function parses JSON using sonic-rs and can optionally convert
/// for high performance JSON processing.
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
        json_value
            .to_string_pretty()
            .map_err(|e| format!("JSON formatting error: {e}"))
    } else {
        json_value
            .to_string()
            .map_err(|e| format!("JSON formatting error: {e}"))
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
    let sonic_value: sonic_rs::Value =
        sonic_rs::from_str(input).map_err(|e| format!("JSON parsing error: {e}"))?;
    Ok(FhirPathValue::JsonValue(
        octofhir_fhirpath_model::JsonValue::new(sonic_value),
    ))
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
    fn test_sonic_conversion() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let sonic_value: sonic_rs::Value = sonic_rs::from_str(json_str).unwrap();
        let fhir_value = from_sonic(sonic_value.clone());
        let back_to_sonic = to_sonic(fhir_value).unwrap();

        assert_eq!(back_to_sonic, sonic_value);
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
        let parsed_original = sonic_rs::from_str::<sonic_rs::Value>(compact).unwrap();
        let parsed_reformatted = sonic_rs::from_str::<sonic_rs::Value>(&back_to_compact).unwrap();
        assert_eq!(parsed_original, parsed_reformatted);
    }
}
