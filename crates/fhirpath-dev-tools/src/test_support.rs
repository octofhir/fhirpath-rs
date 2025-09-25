use octofhir_fhirpath::Collection;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub fn deserialize_nullable_input<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let option = Option::<Value>::deserialize(deserializer)?;
    Ok(option.map(|v| if v.is_null() { Value::Null } else { v }))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    pub name: String,
    pub expression: String,
    #[serde(default, deserialize_with = "deserialize_nullable_input")]
    pub input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputfile: Option<String>,
    pub expected: Value,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "expectError", alias = "expecterror")]
    pub expect_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub predicate: Option<bool>,
    #[serde(rename = "skipStaticCheck", skip_serializing_if = "Option::is_none")]
    pub skip_static_check: Option<bool>,
    #[serde(rename = "invalidKind", skip_serializing_if = "Option::is_none")]
    pub invalid_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "outputTypes", default)]
    pub output_types: Vec<String>,
    // New fields for organized test structure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestSuite {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    pub tests: Vec<TestCase>,
    // New fields for organized test structure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

pub fn normalize_type_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub fn collect_type_names(collection: &Collection) -> Vec<String> {
    collection
        .iter()
        .map(|value| value.display_type_name())
        .collect()
}

pub struct TypeMismatch {
    pub expected: Vec<String>,
    pub actual: Vec<String>,
}

pub fn verify_output_types(expected: &[String], actual: &Collection) -> Result<(), TypeMismatch> {
    if expected.is_empty() {
        return Ok(());
    }

    let actual_raw = collect_type_names(actual);
    let actual_norm: Vec<String> = actual_raw.iter().map(|t| normalize_type_name(t)).collect();
    let expected_norm: Vec<String> = expected.iter().map(|t| normalize_type_name(t)).collect();

    if actual_norm == expected_norm {
        Ok(())
    } else {
        Err(TypeMismatch {
            expected: expected.to_vec(),
            actual: actual_raw,
        })
    }
}

pub fn compare_results(expected: &Value, actual: &Collection) -> bool {
    let actual_json = match serde_json::to_value(actual) {
        Ok(json) => json,
        Err(_) => return false,
    };

    if expected == &actual_json {
        return true;
    }

    match (expected, &actual_json) {
        (expected_single, actual_json) if actual_json.is_array() => {
            if let Some(actual_arr) = actual_json.as_array() {
                if actual_arr.len() == 1 {
                    expected_single == &actual_arr[0]
                } else {
                    false
                }
            } else {
                false
            }
        }
        (expected, actual_single) if expected.is_array() => {
            if let Some(expected_arr) = expected.as_array() {
                if expected_arr.len() == 1 {
                    &expected_arr[0] == actual_single
                } else {
                    expected == actual_single
                }
            } else {
                false
            }
        }
        (expected, actual_json) if expected.is_array() && actual_json.is_null() => expected
            .as_array()
            .map(|arr| arr.is_empty())
            .unwrap_or(false),
        (expected, actual_json) if expected.is_null() && actual_json.is_array() => actual_json
            .as_array()
            .map(|arr| arr.is_empty())
            .unwrap_or(false),
        (expected, actual_single) if expected.is_array() => {
            if let Some(expected_arr) = expected.as_array() {
                if expected_arr.len() == 1 {
                    &expected_arr[0] == actual_single
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
