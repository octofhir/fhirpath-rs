//! Both parser modes must follow the FHIRPath precedence table, not merely agree
//! with each other. Parenthesized expressions make each expected grouping explicit.

use octofhir_fhirpath::parser::{ParsingMode, parse_ast_with_mode};
use serde_json::Value;

fn without_locations_and_parentheses(value: Value) -> Value {
    match value {
        Value::Object(mut fields) => {
            if let Some(inner) = fields.remove("Parenthesized") {
                return without_locations_and_parentheses(inner);
            }
            fields.remove("location");
            Value::Object(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, without_locations_and_parentheses(value)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(without_locations_and_parentheses)
                .collect(),
        ),
        other => other,
    }
}

#[test]
fn both_modes_follow_the_specification_precedence_and_associativity() {
    let cases = [
        ("a.b[0]", "(a.b)[0]"),
        ("-a.b", "-(a.b)"),
        ("-a[0]", "-(a[0])"),
        ("-a * b", "(-a) * b"),
        ("a * b + c", "(a * b) + c"),
        ("a + b * c", "a + (b * c)"),
        ("a + b is Integer", "(a + b) is Integer"),
        ("a | b is Integer", "a | (b is Integer)"),
        ("a as String | b", "(a as String) | b"),
        ("a | b > c", "(a | b) > c"),
        ("a > b = c", "(a > b) = c"),
        ("a = b in c", "(a = b) in c"),
        ("a in b = c", "a in (b = c)"),
        ("a ~ b contains c", "(a ~ b) contains c"),
        ("a contains b != c", "a contains (b != c)"),
        ("a in b and c", "(a in b) and c"),
        ("a and b xor c", "(a and b) xor c"),
        ("a xor b or c", "(a xor b) or c"),
        ("a or b xor c", "(a or b) xor c"),
        ("a or b implies c", "(a or b) implies c"),
        ("a - b + c", "(a - b) + c"),
        ("a * b / c", "(a * b) / c"),
        ("a implies b implies c", "(a implies b) implies c"),
    ];

    for (expression, grouped) in cases {
        let expected = parse_ast_with_mode(grouped, ParsingMode::Fast)
            .unwrap_or_else(|error| panic!("expected grouping {grouped}: {error}"));
        let expected = without_locations_and_parentheses(serde_json::to_value(expected).unwrap());
        for mode in [ParsingMode::Fast, ParsingMode::Analysis] {
            let actual = parse_ast_with_mode(expression, mode)
                .unwrap_or_else(|error| panic!("{mode}: {expression}: {error}"));
            let actual = without_locations_and_parentheses(serde_json::to_value(actual).unwrap());
            assert_eq!(
                actual, expected,
                "{mode}: {expression} must group as {grouped}"
            );
        }
    }
}
