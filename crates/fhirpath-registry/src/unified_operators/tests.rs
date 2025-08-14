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

//! Comprehensive integration tests for all unified operators

use crate::unified_operator_registry::create_unified_operator_registry;
use crate::function::EvaluationContext;
use octofhir_fhirpath_model::{FhirPathValue, Collection, Quantity};
use rust_decimal::Decimal;

/// Test helper to create an evaluation context
fn create_test_context() -> EvaluationContext {
    EvaluationContext::new(FhirPathValue::Empty)
}

/// Comprehensive integration test for arithmetic operators
#[tokio::test]
async fn test_arithmetic_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test addition
    let result = registry.evaluate_binary(
        "+",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(3),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(8));

    // Test subtraction with unary
    let result = registry.evaluate_unary(
        "-",
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(-5));

    // Test multiplication
    let result = registry.evaluate_binary(
        "*",
        FhirPathValue::Integer(4),
        FhirPathValue::Integer(3),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(12));

    // Test division
    let result = registry.evaluate_binary(
        "/",
        FhirPathValue::Integer(10),
        FhirPathValue::Integer(4),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Decimal(Decimal::new(25, 1))); // 2.5

    // Test integer division (div)
    let result = registry.evaluate_binary(
        "div",
        FhirPathValue::Integer(10),
        FhirPathValue::Integer(4),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(2)); // Truncated

    // Test modulo
    let result = registry.evaluate_binary(
        "mod",
        FhirPathValue::Integer(10),
        FhirPathValue::Integer(3),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(1));
}

/// Comprehensive integration test for comparison operators
#[tokio::test]
async fn test_comparison_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test equality
    let result = registry.evaluate_binary(
        "=",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test inequality
    let result = registry.evaluate_binary(
        "!=",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(3),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test less than
    let result = registry.evaluate_binary(
        "<",
        FhirPathValue::Integer(3),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test greater than
    let result = registry.evaluate_binary(
        ">",
        FhirPathValue::Integer(7),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test less than or equal
    let result = registry.evaluate_binary(
        "<=",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test greater than or equal
    let result = registry.evaluate_binary(
        ">=",
        FhirPathValue::Integer(6),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test equivalent (case insensitive)
    let result = registry.evaluate_binary(
        "~",
        FhirPathValue::String("Hello".into()),
        FhirPathValue::String("HELLO".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test not equivalent
    let result = registry.evaluate_binary(
        "!~",
        FhirPathValue::String("Hello".into()),
        FhirPathValue::String("World".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));
}

/// Comprehensive integration test for logical operators
#[tokio::test]
async fn test_logical_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test logical AND
    let result = registry.evaluate_binary(
        "and",
        FhirPathValue::Boolean(true),
        FhirPathValue::Boolean(true),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test logical OR
    let result = registry.evaluate_binary(
        "or",
        FhirPathValue::Boolean(false),
        FhirPathValue::Boolean(true),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test logical XOR
    let result = registry.evaluate_binary(
        "xor",
        FhirPathValue::Boolean(true),
        FhirPathValue::Boolean(false),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test logical IMPLIES
    let result = registry.evaluate_binary(
        "implies",
        FhirPathValue::Boolean(false),
        FhirPathValue::Boolean(true),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test logical NOT
    let result = registry.evaluate_unary(
        "not",
        FhirPathValue::Boolean(true),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(false));
}

/// Comprehensive integration test for type operators
#[tokio::test]
async fn test_type_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test 'is' operator
    let result = registry.evaluate_binary(
        "is",
        FhirPathValue::Integer(5),
        FhirPathValue::String("Integer".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test type hierarchy: Integer is Decimal
    let result = registry.evaluate_binary(
        "is",
        FhirPathValue::Integer(5),
        FhirPathValue::String("Decimal".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test 'as' operator - successful cast
    let result = registry.evaluate_binary(
        "as",
        FhirPathValue::Integer(5),
        FhirPathValue::String("Decimal".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));

    // Test 'as' operator - failed cast
    let result = registry.evaluate_binary(
        "as",
        FhirPathValue::String("not-a-number".into()),
        FhirPathValue::String("Integer".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);
}

/// Comprehensive integration test for collection operators
#[tokio::test]
async fn test_collection_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test union operator
    let left = FhirPathValue::Collection(Collection::from_vec(vec![
        FhirPathValue::Integer(1),
        FhirPathValue::Integer(2),
    ]));
    let right = FhirPathValue::Collection(Collection::from_vec(vec![
        FhirPathValue::Integer(2),
        FhirPathValue::Integer(3),
    ]));

    let result = registry.evaluate_binary("|", left, right, &context).await.unwrap();
    match result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 3); // Deduplicated: [1, 2, 3]
        }
        _ => panic!("Expected collection result"),
    }

    // Test 'in' operator
    let result = registry.evaluate_binary(
        "in",
        FhirPathValue::Integer(5),
        FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(10),
        ])),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));

    // Test 'contains' operator
    let result = registry.evaluate_binary(
        "contains",
        FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(10),
        ])),
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));
}

/// Comprehensive integration test for string operators
#[tokio::test]
async fn test_string_operators_integration() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Test string concatenation
    let result = registry.evaluate_binary(
        "&",
        FhirPathValue::String("Hello".into()),
        FhirPathValue::String(" World".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::String("Hello World".into()));

    // Test concatenation with mixed types
    let result = registry.evaluate_binary(
        "&",
        FhirPathValue::String("Value: ".into()),
        FhirPathValue::Integer(42),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::String("Value: 42".into()));

    // Test concatenation with empty
    let result = registry.evaluate_binary(
        "&",
        FhirPathValue::String("test".into()),
        FhirPathValue::Empty,
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::String("test".into()));
}

/// Test operator precedence through registry
#[tokio::test]
async fn test_operator_precedence() {
    let registry = create_unified_operator_registry();

    // Verify precedence values match FHIRPath specification
    assert_eq!(registry.get_precedence("+"), Some(5));
    assert_eq!(registry.get_precedence("-"), Some(5));
    assert_eq!(registry.get_precedence("&"), Some(5));
    
    assert_eq!(registry.get_precedence("*"), Some(4));
    assert_eq!(registry.get_precedence("/"), Some(4));
    assert_eq!(registry.get_precedence("div"), Some(4));
    assert_eq!(registry.get_precedence("mod"), Some(4));

    assert_eq!(registry.get_precedence("<"), Some(8));
    assert_eq!(registry.get_precedence(">"), Some(8));
    assert_eq!(registry.get_precedence("<="), Some(8));
    assert_eq!(registry.get_precedence(">="), Some(8));

    assert_eq!(registry.get_precedence("="), Some(9));
    assert_eq!(registry.get_precedence("!="), Some(9));
    assert_eq!(registry.get_precedence("~"), Some(9));
    assert_eq!(registry.get_precedence("!~"), Some(9));

    assert_eq!(registry.get_precedence("is"), Some(10));
    assert_eq!(registry.get_precedence("as"), Some(10));
    assert_eq!(registry.get_precedence("in"), Some(10));
    assert_eq!(registry.get_precedence("contains"), Some(10));

    assert_eq!(registry.get_precedence("and"), Some(13));
    assert_eq!(registry.get_precedence("or"), Some(14));
    assert_eq!(registry.get_precedence("xor"), Some(14));
    assert_eq!(registry.get_precedence("implies"), Some(15));
}

/// Test empty value handling across all operators
#[tokio::test]
async fn test_empty_value_handling() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Arithmetic operators with empty should return empty
    let result = registry.evaluate_binary(
        "+",
        FhirPathValue::Empty,
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // Comparison operators with empty follow FHIRPath rules
    let result = registry.evaluate_binary(
        "=",
        FhirPathValue::Empty,
        FhirPathValue::Integer(5),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(false));

    // Collection 'in' operator with empty left operand
    let result = registry.evaluate_binary(
        "in",
        FhirPathValue::Empty,
        FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ])),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // String concatenation with empty
    let result = registry.evaluate_binary(
        "&",
        FhirPathValue::String("test".into()),
        FhirPathValue::Empty,
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::String("test".into()));
}

/// Test operator registry statistics and metadata
#[tokio::test]
async fn test_registry_statistics() {
    let registry = create_unified_operator_registry();
    let stats = registry.get_stats();

    // Should have all major operators registered
    assert!(stats.total_operators >= 25); // All unified operators
    assert!(stats.binary_operators >= 20); // Most operators are binary
    assert!(stats.unary_operators >= 2); // - (unary) and not operators

    // Should have operators in all major categories
    assert!(stats.operators_by_category.len() >= 5); // Arithmetic, Comparison, Logical, Type, Collection, String

    // Should have some commutative operators
    assert!(stats.commutative_operators > 0);

    // Should have optimizable operators
    assert!(stats.optimizable_operators > 0);
}

/// Test error handling for division by zero
#[tokio::test]
async fn test_division_by_zero_handling() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Regular division by zero should return empty
    let result = registry.evaluate_binary(
        "/",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(0),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // Integer division by zero should return empty
    let result = registry.evaluate_binary(
        "div",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(0),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // Modulo by zero should return empty
    let result = registry.evaluate_binary(
        "mod",
        FhirPathValue::Integer(5),
        FhirPathValue::Integer(0),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);
}

/// Test mixed type operations
#[tokio::test]
async fn test_mixed_type_operations() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // Integer + Decimal should work
    let result = registry.evaluate_binary(
        "+",
        FhirPathValue::Integer(5),
        FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Decimal(Decimal::new(75, 1))); // 7.5

    // String comparison should be case sensitive for =
    let result = registry.evaluate_binary(
        "=",
        FhirPathValue::String("Hello".into()),
        FhirPathValue::String("hello".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(false));

    // String comparison should be case insensitive for ~
    let result = registry.evaluate_binary(
        "~",
        FhirPathValue::String("Hello".into()),
        FhirPathValue::String("hello".into()),
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Boolean(true));
}

/// Test three-valued logic for logical operators
#[tokio::test]
async fn test_three_valued_logic() {
    let registry = create_unified_operator_registry();
    let context = create_test_context();

    // true and {} = {}
    let result = registry.evaluate_binary(
        "and",
        FhirPathValue::Boolean(true),
        FhirPathValue::Empty,
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // false or {} = {}
    let result = registry.evaluate_binary(
        "or",
        FhirPathValue::Boolean(false),
        FhirPathValue::Empty,
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);

    // not {} = {}
    let result = registry.evaluate_unary(
        "not",
        FhirPathValue::Empty,
        &context,
    ).await.unwrap();
    assert_eq!(result, FhirPathValue::Empty);
}