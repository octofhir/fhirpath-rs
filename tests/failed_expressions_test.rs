#[cfg(test)]
mod failed_expressions_tests {
    use octofhir_fhirpath::Engine;
    use octofhir_fhirpath::model::FhirPathValue;
    use rust_decimal::Decimal;
    use serde_json::{Value as JsonValue, json};
    use std::str::FromStr;

    fn create_test_patient() -> JsonValue {
        json!({
            "resourceType": "Patient",
            "id": "example",
            "active": true,
            "name": [
                {
                    "use": "official",
                    "family": "Windsor",
                    "given": ["Peter", "James"]
                },
                {
                    "use": "nickname",
                    "family": "Windsor",
                    "given": ["Jim"]
                }
            ],
            "birthDate": "1974-12-25"
        })
    }

    fn create_test_observation() -> JsonValue {
        json!({
            "resourceType": "Observation",
            "value": {
                "value": 185,
                "unit": "[lb_av]"
            }
        })
    }

    fn test_expression(expression: &str, input: JsonValue, expected: Vec<FhirPathValue>) {
        let mut engine = Engine::new();
        let result = engine.evaluate(expression, input);

        match result {
            Ok(actual) => {
                // Convert expected vec to collection or single value
                let expected_value = if expected.is_empty() {
                    FhirPathValue::Empty
                } else if expected.len() == 1 {
                    expected[0].clone()
                } else {
                    FhirPathValue::collection(expected)
                };

                if actual != expected_value {
                    println!("FAILED: {expression}");
                    println!("  Expected: {expected_value:?}");
                    println!("  Actual: {actual:?}");
                }
            }
            Err(e) => {
                if !expected.is_empty() {
                    println!("ERROR: {expression} -> {e}");
                }
            }
        }
    }

    // === HIGH BOUNDARY FUNCTION TESTS (16.7% pass rate) ===

    #[test]
    fn test_high_boundary_functions() {
        let input = JsonValue::Null;

        // These should all work but currently fail
        test_expression(
            "1.587.highBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.5875").unwrap())],
        );
        test_expression(
            "1.587.highBoundary(2)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.59").unwrap())],
        );
        test_expression(
            "1.587.highBoundary(6)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.5875").unwrap())],
        );
        test_expression("1.587.highBoundary(-1)", input.clone(), vec![]);
        test_expression(
            "(-1.587).highBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(
                Decimal::from_str("-1.5865").unwrap(),
            )],
        );
        test_expression(
            "(-1.587).highBoundary(2)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("-1.58").unwrap())],
        );
        test_expression(
            "1.highBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.5").unwrap())],
        );
        test_expression(
            "1.highBoundary(0)",
            input.clone(),
            vec![FhirPathValue::Integer(2)],
        );
        test_expression(
            "120.highBoundary(2)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("120.5").unwrap())],
        );
        test_expression(
            "1.587 'm'.highBoundary(8)",
            input.clone(),
            vec![FhirPathValue::String("1.58750000 'm'".to_string())],
        );
        test_expression(
            "@2014.highBoundary(6)",
            input.clone(),
            vec![FhirPathValue::String("@2014-12".to_string())],
        );
    }

    // === LOW BOUNDARY FUNCTION TESTS (25.0% pass rate) ===

    #[test]
    fn test_low_boundary_functions() {
        let input = JsonValue::Null;

        // These should all work but currently fail
        test_expression(
            "1.587.lowBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.5865").unwrap())],
        );
        test_expression(
            "1.587.lowBoundary(6)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.5865").unwrap())],
        );
        test_expression(
            "1.587.lowBoundary(2)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("1.58").unwrap())],
        );
        test_expression("1.587.lowBoundary(-1)", input.clone(), vec![]);
        test_expression(
            "1.587.lowBoundary(0)",
            input.clone(),
            vec![FhirPathValue::Integer(1)],
        );
        test_expression(
            "(-1.587).lowBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(
                Decimal::from_str("-1.5875").unwrap(),
            )],
        );
        test_expression(
            "(-1.587).lowBoundary(2)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("-1.59").unwrap())],
        );
        test_expression(
            "(-1.587).lowBoundary(0)",
            input.clone(),
            vec![FhirPathValue::Integer(-2)],
        );
        test_expression(
            "1.toDecimal().lowBoundary()",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("0.5").unwrap())],
        );
        test_expression(
            "1.lowBoundary(0)",
            input.clone(),
            vec![FhirPathValue::Integer(0)],
        );
        test_expression(
            "1.lowBoundary(5)",
            input.clone(),
            vec![FhirPathValue::Decimal(Decimal::from_str("0.5").unwrap())],
        );
        test_expression(
            "1.587 'cm'.lowBoundary(8)",
            input.clone(),
            vec![FhirPathValue::String("1.58650000 'cm'".to_string())],
        );
    }

    // === PLUS OPERATOR TESTS (23.5% pass rate) ===

    #[test]
    fn test_plus_operations() {
        let input = create_test_patient();

        // Date/time arithmetic - many failing
        test_expression(
            "@1973-12-25 + 7 days",
            input.clone(),
            vec![FhirPathValue::String("@1974-01-01".to_string())],
        );
        test_expression(
            "@1973-12-25 + 7.7 days",
            input.clone(),
            vec![FhirPathValue::String("@1974-01-01".to_string())],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 7 days",
            input.clone(),
            vec![FhirPathValue::String(
                "@1974-01-01T00:00:00.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 second",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:00:01.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 10 millisecond",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:00:00.010+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 minute",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:01:00.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 hour",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T01:00:00.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25 + 1 day",
            input.clone(),
            vec![FhirPathValue::String("@1973-12-26".to_string())],
        );
        test_expression(
            "@1973-12-25 + 1 month",
            input.clone(),
            vec![FhirPathValue::String("@1974-01-25".to_string())],
        );
        test_expression(
            "@1973-12-25 + 1 week",
            input.clone(),
            vec![FhirPathValue::String("@1974-01-01".to_string())],
        );
        test_expression(
            "@1973-12-25 + 1 year",
            input.clone(),
            vec![FhirPathValue::String("@1974-12-25".to_string())],
        );

        // UCUM unit support
        test_expression(
            "@1973-12-25 + 1 'd'",
            input.clone(),
            vec![FhirPathValue::String("@1973-12-26".to_string())],
        );
        test_expression(
            "@1973-12-25 + 1 'wk'",
            input.clone(),
            vec![FhirPathValue::String("@1974-01-01".to_string())],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 's'",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:00:01.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 0.1 's'",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:00:00.100+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 10 'ms'",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:00:00.010+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 'min'",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T00:01:00.000+10:00".to_string(),
            )],
        );
        test_expression(
            "@1973-12-25T00:00:00.000+10:00 + 1 'h'",
            input.clone(),
            vec![FhirPathValue::String(
                "@1973-12-25T01:00:00.000+10:00".to_string(),
            )],
        );

        // Time operations
        test_expression(
            "@T01:00:00 + 2 hours",
            input.clone(),
            vec![FhirPathValue::String("@T03:00:00".to_string())],
        );
        test_expression(
            "@T23:00:00 + 2 hours",
            input.clone(),
            vec![FhirPathValue::String("@T01:00:00".to_string())],
        );
        test_expression(
            "@T23:00:00 + 50 hours",
            input.clone(),
            vec![FhirPathValue::String("@T01:00:00".to_string())],
        );

        // Invalid units - should return empty
        test_expression("@1973-12-25 + 1 'mo'", input.clone(), vec![]);
        test_expression("@1973-12-25 + 1 'a'", input.clone(), vec![]);
        test_expression("@1975-12-25 + 1 'a'", input.clone(), vec![]);
        test_expression("@1974-12-25 + 7", input.clone(), vec![]);
    }

    // === MINUS OPERATOR TESTS (63.6% pass rate) ===

    #[test]
    fn test_minus_operations() {
        let input = create_test_patient();

        // String subtraction should fail
        test_expression("'a'-'b' = 'ab'", input.clone(), vec![]);

        // Date/time arithmetic
        test_expression(
            "@1974-12-25 - 1 'month'",
            input.clone(),
            vec![FhirPathValue::String("@1974-11-25".to_string())],
        );
        test_expression("@1974-12-25 - 1 'cm'", input.clone(), vec![]);
        test_expression(
            "@T00:30:00 - 1 hour",
            input.clone(),
            vec![FhirPathValue::String("@T23:30:00".to_string())],
        );
        test_expression(
            "@T01:00:00 - 2 hours",
            input.clone(),
            vec![FhirPathValue::String("@T23:00:00".to_string())],
        );
    }

    // === EQUALITY TESTS (96.4% pass rate) ===

    #[test]
    fn test_equality_edge_cases() {
        let input = create_test_patient();
        let obs_input = create_test_observation();

        // The one failing case
        test_expression(
            "@2012-04-15T15:00:00Z = @2012-04-15T10:00:00",
            input.clone(),
            vec![],
        );
        test_expression(
            "Observation.value = 185 '[lb_av]'",
            obs_input,
            vec![FhirPathValue::Boolean(true)],
        );
    }

    // === LITERAL TESTS (92.7% pass rate) ===

    #[test]
    fn test_literal_edge_cases() {
        let input = create_test_patient();

        // Negative number parsing issues
        test_expression("-1.convertsToInteger()", input.clone(), vec![]);
        test_expression("-0.1.convertsToDecimal()", input.clone(), vec![]);

        // Time with timezone - should fail
        test_expression("@T14:34:28Z.is(Time)", input.clone(), vec![]);
        test_expression("@T14:34:28+10:00.is(Time)", input.clone(), vec![]);

        // Invalid unicode literal parsing might fail
        test_expression(
            "'P\\u0065ter'",
            input.clone(),
            vec![FhirPathValue::String("Peter".to_string())],
        );

        // Multi-value boolean operations
        test_expression("(1|2).not() = false", input.clone(), vec![]);
    }

    // === TYPE SYSTEM TESTS (66.7% pass rate) ===

    #[test]
    fn test_type_system_edge_cases() {
        let input = create_test_patient();

        // Namespace resolution issues
        test_expression(
            "Patient.ofType(FHIR.`Patient`).type().name",
            input.clone(),
            vec![FhirPathValue::String("Patient".to_string())],
        );
        test_expression(
            "Patient.is(FHIR.`Patient`)",
            input.clone(),
            vec![FhirPathValue::Boolean(true)],
        );

        // System types vs FHIR types
        test_expression(
            "Patient.is(System.Patient).not()",
            input.clone(),
            vec![FhirPathValue::Boolean(true)],
        );
        test_expression(
            "Patient.active.is(Boolean).not()",
            input.clone(),
            vec![FhirPathValue::Boolean(true)],
        );
        test_expression(
            "Patient.active.is(System.Boolean).not()",
            input.clone(),
            vec![FhirPathValue::Boolean(true)],
        );
    }

    // === DEFINE VARIABLE TESTS (76.2% pass rate) ===

    #[test]
    fn test_define_variable_edge_cases() {
        let input = create_test_patient();

        // Variable scoping issues
        test_expression(
            "defineVariable('n1', name.first()).active | defineVariable('n2', name.skip(1).first()).select(%n1.given)",
            input.clone(),
            vec![],
        );
        test_expression("select(%fam.given)", input.clone(), vec![]);
        test_expression(
            "defineVariable('v1').defineVariable('v1').select(%v1)",
            input.clone(),
            vec![],
        );
        test_expression(
            "Patient.name.defineVariable('n1', first()).active | Patient.name.defineVariable('n2', skip(1).first()).select(%n1.given)",
            input.clone(),
            vec![],
        );
        test_expression(
            "defineVariable('root', 'r1-').select(defineVariable('v1', 'v1').defineVariable('v2', 'v2').select(%v1 | %v2)).select(%root & $this & %v1)",
            input.clone(),
            vec![],
        );
        test_expression("defineVariable('context', 'oops')", input.clone(), vec![]);
        test_expression(
            "defineVariable('n1', 'v1').active | defineVariable('n2', 'v2').select(%n1)",
            input.clone(),
            vec![],
        );
    }

    // === QUANTITY/UCUM TESTS (72.7% pass rate) ===

    #[test]
    fn test_quantity_edge_cases() {
        let input = JsonValue::Null;

        // UCUM unit parsing issues
        test_expression("'1 \\'mo\\''.toQuantity() = 1 month", input.clone(), vec![]);
        test_expression("'1 \\'a\\''.toQuantity() = 1 year", input.clone(), vec![]);
    }

    // === COMPARISON TESTS (90% pass rate each) ===

    #[test]
    fn test_comparison_edge_cases() {
        let _input = JsonValue::Null;

        // Timezone comparison issues probably exist in the failing cases
        // Would need to examine specific failing test cases to identify them
    }
}
