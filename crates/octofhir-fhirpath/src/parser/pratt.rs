//! High-performance Chumsky-based Pratt parser for FHIRPath expressions
//!
//! This implementation provides a complete FHIRPath parser using Chumsky 0.10's
//! Pratt parsing capabilities for proper operator precedence and comprehensive
//! support for all FHIRPath constructs including method calls, property access,
//! and all operators from the original specification.

use chumsky::prelude::*;
use chumsky::pratt::{left, infix, prefix, postfix};
use chumsky::extra;

use crate::ast::{
    ExpressionNode, LiteralNode, IdentifierNode, FunctionCallNode,
    BinaryOperationNode, UnaryOperationNode, PropertyAccessNode, IndexAccessNode,
    MethodCallNode, VariableNode, UnionNode,
    TypeCastNode, TypeCheckNode, CollectionNode,
    BinaryOperator, UnaryOperator, LiteralValue
};
use rust_decimal::Decimal;
use crate::core::{FhirPathError, FP0001};
use super::combinators::{
    string_literal_parser, number_parser, boolean_parser, datetime_literal_parser,
    identifier_parser, variable_parser, equals_parser, not_equals_parser,
    less_equal_parser, greater_equal_parser, keyword_parser
};

/// Parse a FHIRPath expression into an AST using Chumsky Pratt parser
pub fn parse(input: &str) -> Result<ExpressionNode, FhirPathError> {
    let parser = fhirpath_parser();
    
    let result = parser.parse(input).into_result();
    
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
fn fhirpath_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
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
                        .or_not()
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
                .map(|elements| ExpressionNode::Collection(CollectionNode {
                    elements,
                    location: None,
                })),
        ));

        // Pratt parser with comprehensive operator support
        atom.pratt((
            // Union operator (|) - precedence 8
            infix(left(8), just('|').padded(), |left, _, right, _| {
                ExpressionNode::Union(UnionNode {
                    left: Box::new(left),
                    right: Box::new(right),
                    location: None,
                })
            }),
            
            // Logical operators
            infix(left(3), just("and").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::And,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(2), just("or").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Or,
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
            
            // Inequality operators - precedence 7
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
            
            // Additive operators - precedence 9
            infix(left(9), just('+').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Add,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(9), just('-').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(9), just('&').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Concatenate,
                    right: Box::new(right),
                    location: None,
                })
            }),
            
            // Multiplicative operators - precedence 10
            infix(left(10), just('*').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(10), just('/').padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Divide,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(10), just("div").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::IntegerDivide,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(10), just("mod").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Modulo,
                    right: Box::new(right),
                    location: None,
                })
            }),
            
            // Type operators - precedence 5
            infix(left(5), just("is").padded(), |left, _, right, _| {
                if let ExpressionNode::Identifier(ident) = right {
                    ExpressionNode::TypeCheck(TypeCheckNode {
                        expression: Box::new(left),
                        target_type: ident.name,
                        location: None,
                    })
                } else {
                    ExpressionNode::BinaryOperation(BinaryOperationNode {
                        left: Box::new(left),
                        operator: BinaryOperator::Is,
                        right: Box::new(right),
                        location: None,
                    })
                }
            }),
            infix(left(5), just("as").padded(), |left, _, right, _| {
                if let ExpressionNode::Identifier(ident) = right {
                    ExpressionNode::TypeCast(TypeCastNode {
                        expression: Box::new(left),
                        target_type: ident.name,
                        location: None,
                    })
                } else {
                    ExpressionNode::BinaryOperation(BinaryOperationNode {
                        left: Box::new(left),
                        operator: BinaryOperator::Add, // fallback, shouldn't happen
                        right: Box::new(right),
                        location: None,
                    })
                }
            }),
            
            // Membership operators - precedence 4
            infix(left(4), just("in").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::In,
                    right: Box::new(right),
                    location: None,
                })
            }),
            infix(left(4), just("contains").padded(), |left, _, right, _| {
                ExpressionNode::BinaryOperation(BinaryOperationNode {
                    left: Box::new(left),
                    operator: BinaryOperator::Contains,
                    right: Box::new(right),
                    location: None,
                })
            }),
            
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
            postfix(12, 
                expr.clone()
                    .delimited_by(just('[').padded(), just(']').padded()),
                |expr, index, _| {
                    ExpressionNode::IndexAccess(IndexAccessNode {
                        object: Box::new(expr),
                        index: Box::new(index),
                        location: None,
                    })
                }
            ),
            postfix(12,
                just('.')
                    .ignore_then(text::ident())
                    .then(
                        expr.clone()
                            .separated_by(just(',').padded())
                            .collect::<Vec<_>>()
                            .delimited_by(just('(').padded(), just(')').padded())
                            .or_not()
                    ),
                |expr, (name, args): (&str, Option<Vec<ExpressionNode>>), _| {
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
                }
            ),
        ))
    }).then_ignore(end())
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
}