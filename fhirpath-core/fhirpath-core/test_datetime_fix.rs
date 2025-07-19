use fhirpath_core::evaluator::evaluate_expression;
use serde_json::Value;

fn main() {
    // Test a simple DateTime literal
    let patient_json = r#"{"resourceType": "Patient", "id": "example"}"#;
    let patient_data: Value = serde_json::from_str(patient_json).unwrap();

    // Test cases from the official tests
    let test_cases = vec![
        ("@2015T.is(DateTime)", "Should be true - year precision DateTime"),
        ("@2015-02T.is(DateTime)", "Should be true - month precision DateTime"),
        ("@2015-02-04T.is(DateTime)", "Should be true - day precision DateTime"),
        ("@2015-02-04T14:34:28.is(DateTime)", "Should be true - full DateTime"),
        ("@2015-02-04T14:34:28Z.is(DateTime)", "Should be true - UTC DateTime"),
        ("'2015'.convertsToDateTime()", "Should be true - string converts to DateTime"),
    ];

    for (expression, description) in test_cases {
        println!("Testing: {} - {}", expression, description);
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
