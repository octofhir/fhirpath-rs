// FHIRPath Parser Tests
//
// This file contains tests for the FHIRPath parser.

use fhirpath_core::lexer::tokenize;
use fhirpath_core::parser::{parse, AstNode, BinaryOperator};

#[test]
fn test_parse_identifier() {
    let tokens = tokenize("Patient").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::Identifier(name) => {
            assert_eq!(name, "Patient");
        }
        _ => panic!("Expected Identifier node, got {:?}", ast),
    }
}

#[test]
fn test_parse_string_literal() {
    let tokens = tokenize("'hello'").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::StringLiteral(value) => {
            assert_eq!(value, "hello");
        }
        _ => panic!("Expected StringLiteral node, got {:?}", ast),
    }
}

#[test]
fn test_parse_number_literal() {
    let tokens = tokenize("42.5").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::NumberLiteral { value, .. } => {
            assert_eq!(value, 42.5);
        }
        _ => panic!("Expected NumberLiteral node, got {:?}", ast),
    }
}

#[test]
fn test_parse_boolean_literal() {
    let tokens = tokenize("true").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::BooleanLiteral(value) => {
            assert!(value);
        }
        _ => panic!("Expected BooleanLiteral node, got {:?}", ast),
    }
}

#[test]
fn test_parse_path_expression() {
    let tokens = tokenize("Patient.name").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::Path(left, right) => {
            match *left {
                AstNode::Identifier(ref name) => {
                    assert_eq!(name, "Patient");
                }
                _ => panic!("Expected Identifier node for left side, got {:?}", left),
            }

            match *right {
                AstNode::Identifier(ref name) => {
                    assert_eq!(name, "name");
                }
                _ => panic!("Expected Identifier node for right side, got {:?}", right),
            }
        }
        _ => panic!("Expected Path node, got {:?}", ast),
    }
}

#[test]
fn test_parse_indexer() {
    let tokens = tokenize("Patient.name[0]").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::Indexer { collection, index } => {
            match *collection {
                AstNode::Path(ref left, ref right) => {
                    match **left {
                        AstNode::Identifier(ref name) => {
                            assert_eq!(name, "Patient");
                        }
                        _ => panic!(
                            "Expected Identifier node for path left side, got {:?}",
                            left
                        ),
                    }

                    match **right {
                        AstNode::Identifier(ref name) => {
                            assert_eq!(name, "name");
                        }
                        _ => panic!(
                            "Expected Identifier node for path right side, got {:?}",
                            right
                        ),
                    }
                }
                _ => panic!("Expected Path node for collection, got {:?}", collection),
            }

            match *index {
                AstNode::NumberLiteral { value, .. } => {
                    assert_eq!(value, 0.0);
                }
                _ => panic!("Expected NumberLiteral node for index, got {:?}", index),
            }
        }
        _ => panic!("Expected Indexer node, got {:?}", ast),
    }
}

#[test]
fn test_parse_function_call() {
    let tokens = tokenize("where(gender = 'male')").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::FunctionCall { name, arguments } => {
            assert_eq!(name, "where");
            assert_eq!(arguments.len(), 1);

            match &arguments[0] {
                AstNode::BinaryOp { op, left, right } => {
                    assert_eq!(*op, BinaryOperator::Equals);

                    match **left {
                        AstNode::Identifier(ref name) => {
                            assert_eq!(name, "gender");
                        }
                        _ => panic!("Expected Identifier node for left operand, got {:?}", left),
                    }

                    match **right {
                        AstNode::StringLiteral(ref value) => {
                            assert_eq!(value, "male");
                        }
                        _ => panic!(
                            "Expected StringLiteral node for right operand, got {:?}",
                            right
                        ),
                    }
                }
                _ => panic!(
                    "Expected BinaryOp node for argument, got {:?}",
                    arguments[0]
                ),
            }
        }
        _ => panic!("Expected FunctionCall node, got {:?}", ast),
    }
}

#[test]
fn test_parse_binary_expression() {
    let tokens = tokenize("age > 18 and gender = 'male'").unwrap();
    let ast = parse(&tokens).unwrap();

    match ast {
        AstNode::BinaryOp {
            op: op_and,
            left: left_and,
            right: right_and,
        } => {
            assert_eq!(op_and, BinaryOperator::And);

            match *left_and {
                AstNode::BinaryOp { op, left, right } => {
                    assert_eq!(op, BinaryOperator::GreaterThan);

                    match *left {
                        AstNode::Identifier(ref name) => {
                            assert_eq!(name, "age");
                        }
                        _ => panic!("Expected Identifier node for left operand, got {:?}", left),
                    }

                    match *right {
                        AstNode::NumberLiteral { value, .. } => {
                            assert_eq!(value, 18.0);
                        }
                        _ => panic!(
                            "Expected NumberLiteral node for right operand, got {:?}",
                            right
                        ),
                    }
                }
                _ => panic!("Expected BinaryOp node for left side, got {:?}", left_and),
            }

            match *right_and {
                AstNode::BinaryOp { op, left, right } => {
                    assert_eq!(op, BinaryOperator::Equals);

                    match *left {
                        AstNode::Identifier(ref name) => {
                            assert_eq!(name, "gender");
                        }
                        _ => panic!("Expected Identifier node for left operand, got {:?}", left),
                    }

                    match *right {
                        AstNode::StringLiteral(ref value) => {
                            assert_eq!(value, "male");
                        }
                        _ => panic!(
                            "Expected StringLiteral node for right operand, got {:?}",
                            right
                        ),
                    }
                }
                _ => panic!("Expected BinaryOp node for right side, got {:?}", right_and),
            }
        }
        _ => panic!("Expected BinaryOp node, got {:?}", ast),
    }
}

#[test]
fn test_parse_complex_expression() {
    let tokens = tokenize("Patient.name[0].given[0] = 'John' and Patient.gender = 'male'").unwrap();
    let ast = parse(&tokens).unwrap();

    // Just verify that it parses without error
    assert!(matches!(ast, AstNode::BinaryOp { .. }));
}

#[test]
fn test_parse_error_invalid_expression() {
    let tokens = tokenize("Patient.").unwrap();
    let result = parse(&tokens);

    assert!(result.is_err());
}
