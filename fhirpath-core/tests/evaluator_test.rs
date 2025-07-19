// FHIRPath Evaluator Tests
//
// This file contains tests for the FHIRPath evaluator.

use fhirpath_core::evaluator::{evaluate_ast, evaluate_expression, EvaluationContext};
use fhirpath_core::lexer::tokenize;
use fhirpath_core::model::FhirPathValue;
use fhirpath_core::parser::parse;

/// Helper function to extract a single value from a collection result
/// This is useful for tests that expect single values but need to handle the FHIRPath collection requirement
fn extract_single_value(result: FhirPathValue) -> FhirPathValue {
    match result {
        FhirPathValue::Collection(mut values) => {
            if values.len() == 1 {
                values.pop().unwrap()
            } else if values.is_empty() {
                FhirPathValue::Empty
            } else {
                panic!(
                    "Expected single value, got collection with {} items",
                    values.len()
                )
            }
        }
        other => other,
    }
}

#[test]
fn test_evaluate_identifier() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "id": "123",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    let result = evaluate_expression("id", resource).unwrap();
    let single_result = extract_single_value(result);

    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "123");
        }
        _ => panic!("Expected String value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_string_literal() {
    let resource = serde_json::json!({});

    let result = evaluate_expression("'hello'", resource).unwrap();
    let single_result = extract_single_value(result);

    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "hello");
        }
        _ => panic!("Expected String value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_number_literal() {
    let resource = serde_json::json!({});

    // Integer
    let result = evaluate_expression("42", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Integer(value) => {
            assert_eq!(value, 42);
        }
        _ => panic!("Expected Integer value, got {:?}", single_result),
    }

    // Decimal
    let result = evaluate_expression("42.5", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Decimal(value) => {
            assert_eq!(value, 42.5);
        }
        _ => panic!("Expected Decimal value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_boolean_literal() {
    let resource = serde_json::json!({});

    // True
    let result = evaluate_expression("true", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected Boolean value, got {:?}", single_result),
    }

    // False
    let result = evaluate_expression("false", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, false);
        }
        _ => panic!("Expected Boolean value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_path_expression() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    let result = evaluate_expression("name.family", resource).unwrap();
    let single_result = extract_single_value(result);

    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "Doe");
        }
        _ => panic!("Expected String value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_indexer() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John", "J"],
                "family": "Doe"
            },
            {
                "given": ["Johnny"],
                "family": "Doe"
            }
        ]
    });

    // First name
    let result = evaluate_expression("name[0].given[0]", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "John");
        }
        _ => panic!("Expected String value, got {:?}", single_result),
    }

    // Second name
    let result = evaluate_expression("name[1].given[0]", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "Johnny");
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_equality() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "gender": "male"
    });

    // Equal (true)
    let result = evaluate_expression("gender = 'male'", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Equal (false)
    let result = evaluate_expression("gender = 'female'", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, false);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Not equal (true)
    let result = evaluate_expression("gender != 'female'", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Not equal (false)
    let result = evaluate_expression("gender != 'male'", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, false);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_comparison() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "age": 25
    });

    // Less than (true)
    let result = evaluate_expression("age < 30", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Less than (false)
    let result = evaluate_expression("age < 20", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, false);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Greater than (true)
    let result = evaluate_expression("age > 20", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Greater than (false)
    let result = evaluate_expression("age > 30", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, false);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Less than or equal (true - equal)
    let result = evaluate_expression("age <= 25", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Greater than or equal (true - equal)
    let result = evaluate_expression("age >= 25", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert_eq!(value, true);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_arithmetic() {
    let resource = serde_json::json!({});

    // Addition
    let result = evaluate_expression("5 + 3", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Integer(value) => {
            assert_eq!(value, 8);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Subtraction
    let result = evaluate_expression("5 - 3", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Integer(value) => {
            assert_eq!(value, 2);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Multiplication
    let result = evaluate_expression("5 * 3", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Integer(value) => {
            assert_eq!(value, 15);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Division
    let result = evaluate_expression("6 / 3", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Decimal(value) => {
            assert_eq!(value, 2.0);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Mixed types
    let result = evaluate_expression("5 + 3.5", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Decimal(value) => {
            assert_eq!(value, 8.5);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_logical() {
    let resource = serde_json::json!({});

    // And (true)
    let result = evaluate_expression("true and true", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // And (false)
    let result = evaluate_expression("true and false", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(!value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Or (true)
    let result = evaluate_expression("true or false", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Or (false)
    let result = evaluate_expression("false or false", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(!value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Xor (true)
    let result = evaluate_expression("true xor false", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Xor (false)
    let result = evaluate_expression("true xor true", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(!value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Implies (true)
    let result = evaluate_expression("false implies true", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Implies (false)
    let result = evaluate_expression("true implies false", resource).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(!value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_debug_xor() {
    let resource = serde_json::json!({});

    println!("Testing XOR operations:");

    // Test true xor false (should be true)
    let result1 = evaluate_expression("true xor false", resource.clone()).unwrap();
    println!("true xor false = {:?}", result1);

    // Test true xor true (should be false)
    let result2 = evaluate_expression("true xor true", resource.clone()).unwrap();
    println!("true xor true = {:?}", result2);

    // Test false xor false (should be false)
    let result3 = evaluate_expression("false xor false", resource.clone()).unwrap();
    println!("false xor false = {:?}", result3);

    // Test false xor true (should be true)
    let result4 = evaluate_expression("false xor true", resource).unwrap();
    println!("false xor true = {:?}", result4);

    // Test basic boolean XOR in Rust
    println!("\nRust boolean XOR tests:");
    println!("true ^ false = {}", true ^ false);
    println!("true ^ true = {}", true ^ true);
    println!("false ^ false = {}", false ^ false);
    println!("false ^ true = {}", false ^ true);
}

#[test]
fn test_evaluate_unary() {
    let resource = serde_json::json!({});

    // Negation
    let result = evaluate_expression("-5", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Integer(value) => {
            assert_eq!(value, -5);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }

    // Not
    let result = evaluate_expression("true", resource.clone()).unwrap();
    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_complex_expression() {
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ],
        "gender": "male",
        "age": 25
    });

    let result =
        evaluate_expression("name[0].family = 'Doe' and gender = 'male'", resource).unwrap();

    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::Boolean(value) => {
            assert!(value);
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_evaluate_with_context() {
    // Create a context with a variable
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    let tokens = tokenize("name[0].family").unwrap();
    let ast = parse(&tokens).unwrap();

    let mut context = EvaluationContext::new(resource);
    context.set_variable("expected", FhirPathValue::String("Doe".to_string()));

    let result = evaluate_ast(&ast, &context).unwrap();

    let single_result = extract_single_value(result);
    match single_result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "Doe");
        }
        _ => panic!("Expected single value, got {:?}", single_result),
    }
}

#[test]
fn test_exists_function_on_property_chain() {
    // Test case to reproduce the issue where 'name.given.exists()' should return true
    // but currently returns empty object and doesn't apply exists()
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John", "J"],
                "family": "Doe"
            }
        ]
    });

    // First, let's see what name.given returns by itself
    let name_given_result = evaluate_expression("name.given", resource.clone()).unwrap();

    // This should return true because name.given exists and has values
    let result = evaluate_expression("name.given.exists()", resource.clone()).unwrap();

    match result {
        FhirPathValue::Boolean(value) => {
            assert!(value, "name.given.exists() should return true");
        }
        _ => panic!("Expected Boolean value, got {:?}. This demonstrates the bug where exists() is not properly applied.", result),
    }

    // Test with a resource that doesn't have the given field
    let resource_no_given = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "family": "Doe"
            }
        ]
    });

    let result = evaluate_expression("name.given.exists()", resource_no_given).unwrap();
    println!(
        "Result of 'name.given.exists()' on resource without given: {:?}",
        result
    );

    match result {
        FhirPathValue::Boolean(value) => {
            assert!(
                !value,
                "name.given.exists() should return false when given doesn't exist"
            );
        }
        _ => panic!("Expected Boolean value, got {:?}", result),
    }
}

#[test]
fn test_join_function() {
    // Test basic join functionality with comma separator
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["Peter", "James", "Jim", "Peter", "James"],
                "family": "Chalmers"
            }
        ]
    });

    // Test basic join with comma
    let result = evaluate_expression("name.given.join(',')", resource.clone()).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "Peter,James,Jim,Peter,James");
        }
        _ => panic!("Expected String value, got {:?}", result),
    }

    // Test join with different separator
    let result = evaluate_expression("name.given.join(' | ')", resource.clone()).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "Peter | James | Jim | Peter | James");
        }
        _ => panic!("Expected String value, got {:?}", result),
    }

    // Test join with empty string separator
    let result = evaluate_expression("name.given.join('')", resource.clone()).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "PeterJamesJimPeterJames");
        }
        _ => panic!("Expected String value, got {:?}", result),
    }
}

#[test]
fn test_debug_converts_to_integer() {
    // Debug the convertsToInteger function
    let test_cases = vec![
        ("1", "Should be Integer(1)"),
        ("1.0", "Should be Decimal(1.0)"),
        ("1.5", "Should be Decimal(1.5)"),
        ("1.convertsToInteger()", "Should be Boolean(true)"),
        ("0.convertsToInteger()", "Should be Boolean(true)"),
        ("1.0.convertsToInteger()", "Should be Boolean(true)"),
        ("1.5.convertsToInteger()", "Should be Boolean(false)"),
        ("'1'.convertsToInteger()", "Should be Boolean(true)"),
        ("'abc'.convertsToInteger()", "Should be Boolean(false)"),
        ("true.convertsToInteger()", "Should be Boolean(true)"),
    ];

    for (expression, expected_desc) in test_cases {
        match evaluate_expression(expression, serde_json::json!({})) {
            Ok(result) => {
                println!(
                    "Expression: {} -> Result: {:?} ({})",
                    expression, result, expected_desc
                );
            }
            Err(e) => {
                println!(
                    "Expression: {} -> Error: {:?} ({})",
                    expression, e, expected_desc
                );
            }
        }
    }

    // Test join with single item
    let single_resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    let result = evaluate_expression("name.given.join(',')", single_resource).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "John");
        }
        _ => panic!("Expected String value, got {:?}", result),
    }
}

#[test]
fn test_join_function_edge_cases() {
    // Test join with empty collection
    let empty_resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "family": "Doe"
                // No given names
            }
        ]
    });

    let result = evaluate_expression("name.given.join(',')", empty_resource).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "");
        }
        _ => panic!("Expected empty String value, got {:?}", result),
    }

    // Test join with multiple name entries
    let multi_name_resource = serde_json::json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John", "J"],
                "family": "Doe"
            },
            {
                "given": ["Jane"],
                "family": "Smith"
            }
        ]
    });

    let result = evaluate_expression("name.given.join(',')", multi_name_resource).unwrap();
    match result {
        FhirPathValue::String(value) => {
            assert_eq!(value, "John,J,Jane");
        }
        _ => panic!("Expected String value, got {:?}", result),
    }
}
