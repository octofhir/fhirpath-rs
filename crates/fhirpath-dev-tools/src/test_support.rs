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
    // Fields carried over from the official XML suites
    /// `false` when the outputs may come back in any order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
    /// Expression selecting the focus the test expression is evaluated on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Static type(s) of the context, `|`-separated, for semantic checks.
    #[serde(
        rename = "contextType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub context_type: Option<String>,
    /// The operator or function the test exercises.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub testing: Option<String>,
    /// FHIRPath version that introduced the tested behaviour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
    /// FHIR version an official suite targets (`r5`, `r6`).
    #[serde(
        rename = "fhirVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fhir_version: Option<String>,
    /// Group-level mode of an official suite (`strict`, `tx`, `custom`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
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

/// Like [`compare_results`], but the items may come back in any order.
pub fn compare_results_unordered(expected: &Value, actual: &Collection) -> bool {
    let Ok(Value::Array(mut remaining)) = serde_json::to_value(actual) else {
        return compare_results(expected, actual);
    };
    let Value::Array(wanted) = expected else {
        return compare_results(expected, actual);
    };
    if wanted.len() != remaining.len() {
        return false;
    }
    wanted.iter().all(
        |w| match remaining.iter().position(|a| json_values_equal(a, w)) {
            Some(i) => {
                remaining.swap_remove(i);
                true
            }
            None => false,
        },
    )
}

/// Item types of a result, order-insensitive check against the expected types.
pub fn verify_output_types_unordered(
    expected: &[String],
    actual: &Collection,
) -> Result<(), TypeMismatch> {
    let actual_raw = collect_type_names(actual);
    let mut expected_norm: Vec<String> = expected.iter().map(|t| normalize_type_name(t)).collect();
    let mut actual_norm: Vec<String> = actual_raw.iter().map(|t| normalize_type_name(t)).collect();
    expected_norm.sort();
    actual_norm.sort();
    if expected.is_empty() || expected_norm == actual_norm {
        Ok(())
    } else {
        Err(TypeMismatch {
            expected: expected.to_vec(),
            actual: actual_raw,
        })
    }
}

/// Compare JSON values numerically, independently of integer/float encoding.
/// FHIRPath output types are checked separately by `verify_output_types`.
pub fn json_values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let decimal = octofhir_fhirpath::core::value::utils::decimal_from_json_number;
            match (decimal(a), decimal(b)) {
                (Some(a), Some(b)) => a == b,
                _ => a == b,
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| json_values_equal(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, a)| b.get(k).is_some_and(|b| json_values_equal(a, b)))
        }
        _ => a == b,
    }
}

pub fn compare_results(expected: &Value, actual: &Collection) -> bool {
    let actual_json = match serde_json::to_value(actual) {
        Ok(json) => json,
        Err(_) => return false,
    };

    if json_values_equal(expected, &actual_json) {
        return true;
    }

    match (expected, &actual_json) {
        (expected_single, actual_json) if actual_json.is_array() => {
            if let Some(actual_arr) = actual_json.as_array() {
                if actual_arr.len() == 1 {
                    json_values_equal(expected_single, &actual_arr[0])
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
                    json_values_equal(&expected_arr[0], actual_single)
                } else {
                    json_values_equal(expected, actual_single)
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
                    json_values_equal(&expected_arr[0], actual_single)
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
