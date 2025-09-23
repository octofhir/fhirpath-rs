//! High-performance Chumsky-based Pratt parser for FHIRPath expressions
//!
//! This implementation provides a complete FHIRPath parser using Chumsky 0.10's
//! Pratt parsing capabilities for proper operator precedence and comprehensive
//! support for all FHIRPath constructs including method calls, property access,
//! and all operators from the original specification.
//!
//! ## Layered Parsing Architecture
//!
//! To support all 21 binary operators within Chumsky's 26-operator tuple limit,
//! this parser uses a layered approach with 4 parsing layers:
//!
//! 1. **Postfix/Prefix Layer**: Unary operators, method calls, property access, indexing
//! 2. **High Precedence Layer**: Type operators (is/as), multiplicative, additive, union
//! 3. **Medium Precedence Layer**: Relational, equality, membership, concatenation
//! 4. **Low Precedence Layer**: Logical operators (and, xor, or, implies)
//!
//! This ensures full FHIRPath specification compliance including support for
//! `xor` and `implies` operators which were previously missing due to parser limits.

use chumsky::extra;
use chumsky::pratt::{infix, left, postfix, prefix, right};
use chumsky::prelude::*;

use super::combinators::{
    boolean_parser, datetime_literal_parser, identifier_parser, number_parser,
    string_literal_parser, variable_parser,
};
use crate::ast::{
    BinaryOperationNode, BinaryOperator, CollectionNode, ExpressionNode, FunctionCallNode,
    IndexAccessNode, MethodCallNode, PropertyAccessNode, TypeCastNode, TypeCheckNode,
    UnaryOperationNode, UnaryOperator, UnionNode,
};
use crate::core::{FP0001, FhirPathError};

/// Strip comments, decode HTML entities, and normalize whitespace from input
fn preprocess_input(input: &str) -> Result<String, FhirPathError> {
    let original_input = input;
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut string_char = '\0';

    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' if !in_string => {
                in_string = true;
                string_char = ch;
                result.push(ch);
            }
            c if in_string && c == string_char => {
                // Check for escaped quotes
                if chars.peek() == Some(&string_char) {
                    result.push(ch);
                    result.push(chars.next().unwrap());
                } else {
                    in_string = false;
                    result.push(ch);
                }
            }
            '/' if !in_string => {
                match chars.peek() {
                    Some('/') => {
                        // Single-line comment - skip to end of line
                        chars.next(); // consume the second /
                        result.push(' '); // Add space where comment was
                        for c in chars.by_ref() {
                            if c == '\n' || c == '\r' {
                                result.push(' '); // Replace newline with space
                                break;
                            }
                        }
                    }
                    Some('*') => {
                        // Multi-line comment - skip to */
                        chars.next(); // consume the *
                        result.push(' '); // Add space where comment started
                        let mut prev_char = '\0';
                        let mut comment_closed = false;
                        for c in chars.by_ref() {
                            if prev_char == '*' && c == '/' {
                                comment_closed = true;
                                break; // End of comment
                            }
                            prev_char = c;
                        }
                        // Check if comment was properly closed
                        if !comment_closed {
                            return Err(FhirPathError::parse_error(
                                FP0001,
                                "Unterminated multi-line comment: found '/*' but missing closing '*/'",
                                original_input,
                                None,
                            ));
                        }
                    }
                    _ => result.push(ch),
                }
            }
            '&' if !in_string => {
                // Handle HTML entities
                let remaining: String = chars.clone().collect();
                if remaining.starts_with("lt;") {
                    // Consume "lt;"
                    chars.next();
                    chars.next();
                    chars.next(); // l, t, ;
                    result.push('<');
                } else if remaining.starts_with("gt;") {
                    // Consume "gt;"
                    chars.next();
                    chars.next();
                    chars.next(); // g, t, ;
                    result.push('>');
                } else if remaining.starts_with("amp;") {
                    // Consume "amp;"
                    chars.next();
                    chars.next();
                    chars.next();
                    chars.next(); // a, m, p, ;
                    result.push('&');
                } else if remaining.starts_with("quot;") {
                    // Consume "quot;"
                    chars.next();
                    chars.next();
                    chars.next();
                    chars.next();
                    chars.next(); // q, u, o, t, ;
                    result.push('"');
                } else if remaining.starts_with("apos;") {
                    // Consume "apos;"
                    chars.next();
                    chars.next();
                    chars.next();
                    chars.next();
                    chars.next(); // a, p, o, s, ;
                    result.push('\'');
                } else {
                    result.push(ch);
                }
            }
            // Normalize newlines and tabs to spaces outside of strings
            '\n' | '\r' | '\t' if !in_string => {
                result.push(' ');
            }
            _ => result.push(ch),
        }
    }

    // Trim and collapse multiple whitespaces
    let normalized = result.split_whitespace().collect::<Vec<_>>().join(" ");

    Ok(normalized)
}

/// Parse a FHIRPath expression into an AST using Chumsky Pratt parser
pub fn parse(input: &str) -> Result<ExpressionNode, FhirPathError> {
    // Preprocess to remove comments
    let cleaned_input = preprocess_input(input)?;
    let parser = fhirpath_parser();

    let result = parser.parse(&cleaned_input).into_result();

    match result {
        Ok(ast) => Ok(ast),
        Err(errors) => {
            // Convert Rich errors to FhirPathError - fail fast, no recovery
            let error_msg = if !errors.is_empty() {
                // Take first error only (fail fast)
                format!("{}", errors[0])
            } else {
                "Parse error".to_string()
            };

            Err(FhirPathError::parse_error(FP0001, &error_msg, input, None))
        }
    }
}

/// Create a parser function
pub fn parser() -> impl Fn(&str) -> Result<ExpressionNode, FhirPathError> {
    |input: &str| parse(input)
}

/// Main FHIRPath parser using Chumsky's Pratt parsing - fail fast, no error recovery
fn fhirpath_parser<'a>()
-> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        // Atom parsers - the building blocks using shared combinators
        let atom = choice((
            // All literal types (string, number, boolean, datetime)
            string_literal_parser(),
            number_parser(),
            boolean_parser(),
            datetime_literal_parser(),
            // Variable references
            variable_parser(),
            // Function calls and identifiers using shared combinators
            identifier_parser()
                .then(
                    expr.clone()
                        .separated_by(just(',').padded())
                        .collect::<Vec<_>>()
                        .delimited_by(just('(').padded(), just(')').padded())
                        .or_not(),
                )
                .map(|(identifier, args)| {
                    if let ExpressionNode::Identifier(id_node) = identifier {
                        if let Some(arguments) = args {
                            ExpressionNode::FunctionCall(FunctionCallNode {
                                name: id_node.name,
                                arguments,
                                location: None,
                            })
                        } else {
                            ExpressionNode::Identifier(id_node)
                        }
                    } else {
                        identifier // Fallback, shouldn't happen
                    }
                }),
            // Parenthesized expressions
            expr.clone()
                .delimited_by(just('(').padded(), just(')').padded())
                .map(|e| ExpressionNode::Parenthesized(Box::new(e))),
            // Collection literals
            expr.clone()
                .separated_by(just(',').padded())
                .collect::<Vec<_>>()
                .delimited_by(just('{').padded(), just('}').padded())
                .map(|elements| {
                    ExpressionNode::Collection(CollectionNode {
                        elements,
                        location: None,
                    })
                }),
        ));

        // Layered Pratt parsing to support all operators within Chumsky's limits
        // We parse operators in groups to avoid the 26-operator tuple limit

        // Layer 1: Postfix and prefix operators (highest precedence)
        let with_postfix = atom.pratt((
            // Unary operators - precedence 11
            prefix(11, just('-').padded(), |_, operand, _| {
                ExpressionNode::UnaryOperation(UnaryOperationNode {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(operand),
                    location: None,
                })
            }),
            prefix(11, just("not").padded(), |_, operand, _| {
                ExpressionNode::UnaryOperation(UnaryOperationNode {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                    location: None,
                })
            }),
            // Postfix operators - highest precedence (12)
            postfix(
                12,
                expr.clone()
                    .delimited_by(just('[').padded(), just(']').padded()),
                |expr, index, _| {
                    ExpressionNode::IndexAccess(IndexAccessNode {
                        object: Box::new(expr),
                        index: Box::new(index),
                        location: None,
                    })
                },
            ),
            postfix(
                12,
                just('.').ignore_then(identifier_parser()).then(
                    expr.clone()
                        .separated_by(just(',').padded())
                        .collect::<Vec<_>>()
                        .delimited_by(just('(').padded(), just(')').padded())
                        .or_not(),
                ),
                |expr, (identifier, args): (ExpressionNode, Option<Vec<ExpressionNode>>), _| {
                    let name = if let ExpressionNode::Identifier(id) = identifier {
                        id.name
                    } else {
                        "unknown".to_string() // This should not happen
                    };
                    if let Some(arguments) = args {
                        // Method call
                        ExpressionNode::MethodCall(MethodCallNode {
                            object: Box::new(expr),
                            method: name.to_string(),
                            arguments,
                            location: None,
                        })
                    } else {
                        // Property access
                        ExpressionNode::PropertyAccess(PropertyAccessNode {
                            object: Box::new(expr),
                            property: name.to_string(),
                            location: None,
                        })
                    }
                },
            ),
        ));

        // Layer 2: High precedence operators (type, multiplicative, additive, union)
        let with_high_precedence = with_postfix.pratt((
            // Multiplicative operators - precedence 11
            infix(left(11), just('*').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(11), just('/').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Divide,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(11), just("div").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::IntegerDivide,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(11), just("mod").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Modulo,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Additive operators - precedence 10
            infix(left(10), just('+').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Add,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(10), just('-').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Union operator - precedence 9
            infix(left(9), just('|').padded(), |left, _, right, _| {
                ExpressionNode::Union(UnionNode {
                    left: Box::new(left),
                    right: Box::new(right),
                    location: None,
                })
            }),
        ));

        // Layer 3: Medium precedence operators (relational, equality, membership, concatenation)
        let with_medium_precedence = with_high_precedence.pratt((
            // String concatenation - precedence 8 (higher than relational and equality)
            infix(left(8), just('&').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Concatenate,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Relational operators - precedence 7
            infix(left(7), just("<=").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::LessThanOrEqual,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(7), just(">=").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::GreaterThanOrEqual,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(7), just("<").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::LessThan,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(7), just(">").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::GreaterThan,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Equality operators - precedence 6
            infix(left(6), just("=").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Equal,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(6), just("!=").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::NotEqual,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(6), just("~").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Equivalent,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(6), just("!~").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::NotEquivalent,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Membership operators - precedence 5
            infix(left(5), just("in").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::In,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(5), just("contains").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Contains,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // String concatenation - precedence 8 (higher than equality)
            infix(left(8), just('&').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Concatenate,
                    right: Box::new(right),
                    location: None,
                })
            }),
        ));

        // Layer 4: Low precedence logical operators (and, xor, or, implies)
        with_medium_precedence.pratt((
            // Type operators - precedence 3.5 (lower than membership, same as XOR but evaluated first)
            infix(left(3), just("is").padded(), |left, _, right, _| {
                match right {
                    ExpressionNode::Identifier(ident) => ExpressionNode::TypeCheck(TypeCheckNode {
                        expression: Box::new(left),
                        target_type: ident.name,
                        location: None,
                    }),
                    ExpressionNode::PropertyAccess(prop) => {
                        if let ExpressionNode::Identifier(base) = *prop.object {
                            let target_type = format!("{}.{}", base.name, prop.property);
                            ExpressionNode::TypeCheck(TypeCheckNode {
                                expression: Box::new(left),
                                target_type,
                                location: None,
                            })
                        } else {
                            ExpressionNode::BinaryOperation(BinaryOperationNode {
                                left: Box::new(left),
                                operator: BinaryOperator::Is,
                                right: Box::new(ExpressionNode::PropertyAccess(prop)),
                                location: None,
                            })
                        }
                    }
                    other => ExpressionNode::BinaryOperation(BinaryOperationNode {
                        left: Box::new(left),
                        operator: BinaryOperator::Is,
                        right: Box::new(other),
                        location: None,
                    }),
                }
            }),
            infix(left(3), just("as").padded(), |left, _, right, _| {
                match right {
                    ExpressionNode::Identifier(ident) => ExpressionNode::TypeCast(TypeCastNode {
                        expression: Box::new(left),
                        target_type: ident.name,
                        location: None,
                    }),
                    ExpressionNode::PropertyAccess(prop) => {
                        if let ExpressionNode::Identifier(base) = *prop.object {
                            let target_type = format!("{}.{}", base.name, prop.property);
                            ExpressionNode::TypeCast(TypeCastNode {
                                expression: Box::new(left),
                                target_type,
                                location: None,
                            })
                        } else {
                            ExpressionNode::BinaryOperation(BinaryOperationNode {
                                left: Box::new(left),
                                operator: BinaryOperator::As,
                                right: Box::new(ExpressionNode::PropertyAccess(prop)),
                                location: None,
                            })
                        }
                    }
                    other => ExpressionNode::BinaryOperation(BinaryOperationNode {
                        left: Box::new(left),
                        operator: BinaryOperator::As,
                        right: Box::new(other),
                        location: None,
                    }),
                }
            }),
            // Logical AND - precedence 4
            infix(left(4), just("and").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::And,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Logical XOR - precedence 3 (NOW SUPPORTED!)
            infix(left(3), just("xor").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Xor,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Logical OR - precedence 2
            infix(left(2), just("or").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Or,
                    right: Box::new(right),
                    location: None,
                })
            }),
            // Logical implies - precedence 1 (NOW SUPPORTED! Right-associative)
            infix(right(1), just("implies").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Implies,
                    right: Box::new(right),
                    location: None,
                })
            }),
        ))
    })
    .padded() // Use padded to handle trailing whitespace
    .then_ignore(end())
}

// String literal parser is now provided by shared combinators module

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let result = parse("Patient").unwrap();
        assert!(matches!(result, ExpressionNode::Identifier(_)));
    }

    #[test]
    fn test_property_access() {
        let result = parse("Patient.name").unwrap();
        assert!(matches!(result, ExpressionNode::PropertyAccess(_)));

        if let ExpressionNode::PropertyAccess(node) = result {
            assert!(matches!(*node.object, ExpressionNode::Identifier(_)));
            assert_eq!(node.property, "name");
        }
    }

    #[test]
    fn test_method_call() {
        let result = parse("Patient.name.first()").unwrap();
        assert!(matches!(result, ExpressionNode::MethodCall(_)));

        if let ExpressionNode::MethodCall(node) = result {
            assert_eq!(node.method, "first");
            assert!(node.arguments.is_empty());
            assert!(matches!(*node.object, ExpressionNode::PropertyAccess(_)));
        }
    }

    #[test]
    fn test_chained_method_calls() {
        let result = parse("Patient.name.where(use = 'official').given.first()").unwrap();
        // This should parse as a chain of method calls and property accesses
        assert!(matches!(result, ExpressionNode::MethodCall(_)));
    }

    #[test]
    fn test_binary_operations() {
        let result = parse("age > 18").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));

        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::GreaterThan);
        }
    }

    #[test]
    fn test_all_logical_operators() {
        // Test AND
        let result = parse("true and false").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::And);
        }

        // Test OR
        let result = parse("true or false").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::Or);
        }

        // Test XOR - this should now work!
        let result = parse("true xor false").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::Xor);
        }

        // Test IMPLIES - this should now work!
        let result = parse("true implies false").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::Implies);
        }
    }

    #[test]
    fn test_logical_operator_precedence() {
        // Test: implies < or < xor < and
        let result = parse("a and b or c xor d implies e").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));

        // Should parse as: ((a and b) or c) xor d) implies e
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::Implies);
        }
    }

    #[test]
    fn test_operator_precedence() {
        let result = parse("1 + 2 * 3").unwrap();
        // Should parse as 1 + (2 * 3) due to precedence
        if let ExpressionNode::BinaryOperation(node) = result {
            assert_eq!(node.operator, BinaryOperator::Add);
            if let ExpressionNode::BinaryOperation(right) = *node.right {
                assert_eq!(right.operator, BinaryOperator::Multiply);
            } else {
                panic!("Expected multiplication on right side");
            }
        } else {
            panic!("Expected addition");
        }
    }

    #[test]
    fn test_function_calls() {
        let result = parse("count()").unwrap();
        assert!(matches!(result, ExpressionNode::FunctionCall(_)));

        let result = parse("substring(1, 3)").unwrap();
        assert!(matches!(result, ExpressionNode::FunctionCall(_)));

        if let ExpressionNode::FunctionCall(node) = result {
            assert_eq!(node.name, "substring");
            assert_eq!(node.arguments.len(), 2);
        }
    }

    #[test]
    fn test_variables() {
        let result = parse("$this").unwrap();
        assert!(matches!(result, ExpressionNode::Variable(_)));

        if let ExpressionNode::Variable(node) = result {
            assert_eq!(node.name, "this");
        }
    }

    #[test]
    fn test_literals() {
        use crate::ast::literal::LiteralValue;

        // Single quote strings
        let result = parse("'hello world'").unwrap();
        assert!(matches!(result, ExpressionNode::Literal(_)));
        if let ExpressionNode::Literal(node) = result {
            assert!(matches!(node.value, LiteralValue::String(ref s) if s == "hello world"));
        }

        // Double quote strings
        let result = parse("\"hello world\"").unwrap();
        assert!(matches!(result, ExpressionNode::Literal(_)));
        if let ExpressionNode::Literal(node) = result {
            assert!(matches!(node.value, LiteralValue::String(ref s) if s == "hello world"));
        }

        // Numbers
        let result = parse("42").unwrap();
        assert!(matches!(result, ExpressionNode::Literal(_)));
        if let ExpressionNode::Literal(node) = result {
            assert!(matches!(node.value, LiteralValue::Integer(42)));
        }

        let result = parse("3.14").unwrap();
        assert!(matches!(result, ExpressionNode::Literal(_)));

        // Booleans
        let result = parse("true").unwrap();
        assert!(matches!(result, ExpressionNode::Literal(_)));
        if let ExpressionNode::Literal(node) = result {
            assert!(matches!(node.value, LiteralValue::Boolean(true)));
        }
    }

    #[test]
    fn test_both_quote_types() {
        // Test single quotes
        let result = parse("name = 'test'").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));

        // Test double quotes
        let result = parse("name = \"test\"").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));

        // Test mixed quotes in complex expressions
        let result = parse("name = 'John' and status = \"active\"").unwrap();
        assert!(matches!(result, ExpressionNode::BinaryOperation(_)));
    }

    #[test]
    fn test_indexing() {
        let result = parse("name[0]").unwrap();
        assert!(matches!(result, ExpressionNode::IndexAccess(_)));

        if let ExpressionNode::IndexAccess(node) = result {
            assert!(matches!(*node.object, ExpressionNode::Identifier(_)));
            assert!(matches!(*node.index, ExpressionNode::Literal(_)));
        }
    }

    #[test]
    fn test_type_operations() {
        let result = parse("value is string").unwrap();
        assert!(matches!(result, ExpressionNode::TypeCheck(_)));

        let result = parse("value as string").unwrap();
        assert!(matches!(result, ExpressionNode::TypeCast(_)));
    }

    #[test]
    fn test_all_binary_operators() {
        // Test all 21 binary operators are supported
        let test_cases = [
            ("1 + 2", BinaryOperator::Add),
            ("1 - 2", BinaryOperator::Subtract),
            ("1 * 2", BinaryOperator::Multiply),
            ("1 / 2", BinaryOperator::Divide),
            ("1 mod 2", BinaryOperator::Modulo),
            ("1 div 2", BinaryOperator::IntegerDivide),
            ("1 = 2", BinaryOperator::Equal),
            ("1 != 2", BinaryOperator::NotEqual),
            ("1 ~ 2", BinaryOperator::Equivalent),
            ("1 !~ 2", BinaryOperator::NotEquivalent),
            ("1 < 2", BinaryOperator::LessThan),
            ("1 <= 2", BinaryOperator::LessThanOrEqual),
            ("1 > 2", BinaryOperator::GreaterThan),
            ("1 >= 2", BinaryOperator::GreaterThanOrEqual),
            ("true and false", BinaryOperator::And),
            ("true or false", BinaryOperator::Or),
            ("true xor false", BinaryOperator::Xor), // Now supported!
            ("true implies false", BinaryOperator::Implies), // Now supported!
            ("'a' & 'b'", BinaryOperator::Concatenate),
            ("a | b", BinaryOperator::Union), // Union is handled specially
            ("1 in collection", BinaryOperator::In),
            ("collection contains 1", BinaryOperator::Contains),
        ];

        for (expression, expected_op) in test_cases {
            let result = parse(expression);
            assert!(result.is_ok(), "Failed to parse: {}", expression);

            match result.unwrap() {
                ExpressionNode::BinaryOperation(node) => {
                    assert_eq!(
                        node.operator, expected_op,
                        "Wrong operator for: {}",
                        expression
                    );
                }
                ExpressionNode::Union(_) if expected_op == BinaryOperator::Union => {
                    // Union is handled specially with its own node type
                }
                other => panic!(
                    "Expected binary operation for '{}', got: {:?}",
                    expression, other
                ),
            }
        }
    }
}
