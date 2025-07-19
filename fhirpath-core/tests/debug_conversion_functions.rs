use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::model::FhirPathValue;
use serde_json::json;

// We need to access the validation functions directly for testing
// Let's create a simple test that checks the string validation directly

#[test]
fn debug_conversion_functions() {
    let context = json!({"resourceType": "Patient"});

    // Test convertsToDate
    let test_cases = vec![
        ("'2015'.convertsToDate()", "2015"),
        ("'2015-02'.convertsToDate()", "2015-02"),
        ("'2015-02-04'.convertsToDate()", "2015-02-04"),
        ("'14'.convertsToTime()", "14"),
        ("'14:34'.convertsToTime()", "14:34"),
        ("'14:34:28'.convertsToTime()", "14:34:28"),
    ];

    for (expression, input) in test_cases {
        println!("Testing: {} with input '{}'", expression, input);

        // First test the basic string evaluation
        let string_expr = format!("'{}'", input);
        match evaluate_expression(&string_expr, context.clone()) {
            Ok(string_result) => {
                println!("  String evaluation: {:?}", string_result);
            }
            Err(e) => {
                println!("  String evaluation ERROR: {:?}", e);
            }
        }

        // Then test the conversion function
        match evaluate_expression(expression, context.clone()) {
            Ok(result) => {
                println!("  Conversion result: {:?}", result);
                match result {
                    FhirPathValue::Boolean(b) => {
                        if !b {
                            println!("  ERROR: Expected true but got false!");
                        }
                    }
                    _ => println!("  ERROR: Expected boolean result!"),
                }
            }
            Err(e) => {
                println!("  Conversion ERROR: {:?}", e);
            }
        }
        println!();
    }
}
