use fhirpath_core::evaluator::{evaluate_expression, is_valid_datetime_string};
use serde_json::Value;

#[test]
fn debug_converts_to_date_function() {
    let patient_json = r#"{"resourceType": "Patient", "id": "example"}"#;
    let patient_data: Value = serde_json::from_str(patient_json).unwrap();

    // First test the is_valid_datetime_string function directly
    println!("=== Testing is_valid_datetime_string directly ===");
    let date_strings = vec![
        "2015",
        "2015-02",
        "2015-02-04",
        "2015-02-04T14:34:28",
        "invalid",
    ];

    for date_str in date_strings {
        let is_valid = is_valid_datetime_string(date_str);
        println!("is_valid_datetime_string('{}') = {}", date_str, is_valid);
    }

    println!("\n=== Testing convertsToDate expressions ===");
    // Test cases that are failing
    let test_cases = vec![
        "'2015'.convertsToDate()",
        "'2015-02'.convertsToDate()",
        "'2015-02-04'.convertsToDate()",
        "'2015-02-04T14:34:28'.convertsToDate()", // This should be false (has T)
        "'invalid'.convertsToDate()", // This should be false
    ];

    for expression in test_cases {
        println!("Testing expression: {}", expression);
        match evaluate_expression(expression, patient_data.clone()) {
            Ok(result) => {
                println!("  Result: {:?}", result);
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
        println!();
    }
}
