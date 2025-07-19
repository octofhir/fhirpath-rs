// FHIRPath Lexer Tests
//
// This file contains tests for the FHIRPath lexer.

use fhirpath_core::lexer::{tokenize, TokenType};

#[test]
fn test_empty_input() {
    let tokens = tokenize("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::EOF);
}

#[test]
fn test_whitespace() {
    let tokens = tokenize("   \t\n  ").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::EOF);
}

#[test]
fn test_identifiers() {
    let tokens = tokenize("name _id identifier123").unwrap();
    assert_eq!(tokens.len(), 4); // 3 identifiers + EOF

    assert_eq!(tokens[0].token_type, TokenType::Identifier);
    assert_eq!(tokens[0].lexeme, "name");

    assert_eq!(tokens[1].token_type, TokenType::Identifier);
    assert_eq!(tokens[1].lexeme, "_id");

    assert_eq!(tokens[2].token_type, TokenType::Identifier);
    assert_eq!(tokens[2].lexeme, "identifier123");
}

#[test]
fn test_keywords() {
    let tokens = tokenize("and or xor implies in true false").unwrap();
    assert_eq!(tokens.len(), 8); // 7 keywords + EOF

    assert_eq!(tokens[0].token_type, TokenType::And);
    assert_eq!(tokens[0].lexeme, "and");

    assert_eq!(tokens[1].token_type, TokenType::Or);
    assert_eq!(tokens[1].lexeme, "or");

    assert_eq!(tokens[2].token_type, TokenType::Xor);
    assert_eq!(tokens[2].lexeme, "xor");

    assert_eq!(tokens[3].token_type, TokenType::Implies);
    assert_eq!(tokens[3].lexeme, "implies");

    assert_eq!(tokens[4].token_type, TokenType::In);
    assert_eq!(tokens[4].lexeme, "in");

    assert_eq!(tokens[5].token_type, TokenType::BooleanLiteral);
    assert_eq!(tokens[5].lexeme, "true");

    assert_eq!(tokens[6].token_type, TokenType::BooleanLiteral);
    assert_eq!(tokens[6].lexeme, "false");
}

#[test]
fn test_string_literals() {
    let tokens = tokenize("'hello' 'world' 'escaped''quote'").unwrap();
    assert_eq!(tokens.len(), 4); // 3 strings + EOF

    assert_eq!(tokens[0].token_type, TokenType::StringLiteral);
    assert_eq!(tokens[0].lexeme, "hello");

    assert_eq!(tokens[1].token_type, TokenType::StringLiteral);
    assert_eq!(tokens[1].lexeme, "world");

    assert_eq!(tokens[2].token_type, TokenType::StringLiteral);
    assert_eq!(tokens[2].lexeme, "escaped'quote");
}

#[test]
fn test_number_literals() {
    let tokens = tokenize("123 45.67 0.5 42").unwrap();
    assert_eq!(tokens.len(), 5); // 4 numbers + EOF

    assert_eq!(tokens[0].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[0].lexeme, "123");

    assert_eq!(tokens[1].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[1].lexeme, "45.67");

    assert_eq!(tokens[2].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[2].lexeme, "0.5");

    assert_eq!(tokens[3].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[3].lexeme, "42");
}

#[test]
fn test_operators() {
    let tokens = tokenize("+ - * / = != < <= > >=").unwrap();
    assert_eq!(tokens.len(), 11); // 10 operators + EOF

    assert_eq!(tokens[0].token_type, TokenType::Plus);
    assert_eq!(tokens[1].token_type, TokenType::Minus);
    assert_eq!(tokens[2].token_type, TokenType::Multiply);
    assert_eq!(tokens[3].token_type, TokenType::Divide);
    assert_eq!(tokens[4].token_type, TokenType::Equal);
    assert_eq!(tokens[5].token_type, TokenType::NotEqual);
    assert_eq!(tokens[6].token_type, TokenType::LessThan);
    assert_eq!(tokens[7].token_type, TokenType::LessOrEqual);
    assert_eq!(tokens[8].token_type, TokenType::GreaterThan);
    assert_eq!(tokens[9].token_type, TokenType::GreaterOrEqual);
}

#[test]
fn test_delimiters() {
    let tokens = tokenize("( ) [ ] , .").unwrap();
    assert_eq!(tokens.len(), 7); // 6 delimiters + EOF

    assert_eq!(tokens[0].token_type, TokenType::LeftParen);
    assert_eq!(tokens[1].token_type, TokenType::RightParen);
    assert_eq!(tokens[2].token_type, TokenType::LeftBracket);
    assert_eq!(tokens[3].token_type, TokenType::RightBracket);
    assert_eq!(tokens[4].token_type, TokenType::Comma);
    assert_eq!(tokens[5].token_type, TokenType::Dot);
}

#[test]
fn test_complex_expression() {
    let expr = "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'";
    let tokens = tokenize(expr).unwrap();

    // Verify token count (excluding EOF)
    assert_eq!(tokens.len() - 1, 19);

    // Check a few key tokens
    assert_eq!(tokens[0].token_type, TokenType::Identifier);
    assert_eq!(tokens[0].lexeme, "Patient");

    assert_eq!(tokens[1].token_type, TokenType::Dot);

    assert_eq!(tokens[2].token_type, TokenType::Identifier);
    assert_eq!(tokens[2].lexeme, "name");

    assert_eq!(tokens[3].token_type, TokenType::LeftBracket);

    assert_eq!(tokens[4].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[4].lexeme, "0");

    assert_eq!(tokens[9].token_type, TokenType::NumberLiteral);
    assert_eq!(tokens[9].lexeme, "0");

    assert_eq!(tokens[10].token_type, TokenType::RightBracket);

    assert_eq!(tokens[11].token_type, TokenType::Equal);

    assert_eq!(tokens[12].token_type, TokenType::StringLiteral);
    assert_eq!(tokens[12].lexeme, "John");

    assert_eq!(tokens[13].token_type, TokenType::And);
}

#[test]
fn test_position_tracking() {
    let expr = "a + b";
    let tokens = tokenize(expr).unwrap();

    assert_eq!(tokens[0].position, 0);
    assert_eq!(tokens[0].column, 1);
    assert_eq!(tokens[0].line, 1);

    assert_eq!(tokens[1].position, 2);
    assert_eq!(tokens[1].column, 3);
    assert_eq!(tokens[1].line, 1);

    assert_eq!(tokens[2].position, 4);
    assert_eq!(tokens[2].column, 5);
    assert_eq!(tokens[2].line, 1);
}

#[test]
fn test_multiline_input() {
    let expr = "a\n+ b";
    let tokens = tokenize(expr).unwrap();

    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[0].column, 1);

    assert_eq!(tokens[1].line, 2);
    assert_eq!(tokens[1].column, 1);

    assert_eq!(tokens[2].line, 2);
    assert_eq!(tokens[2].column, 3);
}

#[test]
fn test_error_unterminated_string() {
    let result = tokenize("'unterminated");
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unterminated string literal"));
}

#[test]
fn test_error_invalid_decimal() {
    let result = tokenize("123.");
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Expected digit after decimal point"));
}

#[test]
fn test_error_unexpected_character() {
    let result = tokenize("#");
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unexpected character"));
}

#[test]
fn test_integer_method_call_tokenization() {
    // Test tokenization of integer literals with method calls
    let expressions = vec![
        ("1.convertsToInteger()", "Integer with method call"),
        ("1.0.convertsToInteger()", "Decimal with method call"),
        ("'1'.convertsToInteger()", "String with method call"),
    ];

    for (expr, description) in expressions {
        println!("\nTesting {}: {}", description, expr);
        let tokens = tokenize(expr).unwrap();

        println!("Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?} - '{}'", i, token.token_type, token.lexeme);
        }

        // All expressions should have at least: literal, dot, identifier, left_paren, right_paren, EOF
        assert!(
            tokens.len() >= 6,
            "Expected at least 6 tokens for {}, got {}",
            expr,
            tokens.len()
        );

        // Check that we have a dot token
        let has_dot = tokens.iter().any(|t| t.token_type == TokenType::Dot);
        assert!(has_dot, "Expected dot token in {}", expr);

        // Check that we have the function name
        let has_converts = tokens.iter().any(|t| t.lexeme == "convertsToInteger");
        assert!(
            has_converts,
            "Expected 'convertsToInteger' identifier in {}",
            expr
        );
    }
}

#[test]
fn test_datetime_tokenization() {
    let test_expressions = vec![
        "@2015-02-04",
        "@2015-02-04T14",
        "@2015-02-04T14:30",
        "@2015-02-04T14:30:45",
    ];

    for expr in test_expressions {
        println!("\n=== Tokenizing: {} ===", expr);
        match tokenize(expr) {
            Ok(tokens) => {
                for (i, token) in tokens.iter().enumerate() {
                    println!("  {}: {:?} - '{}'", i, token.token_type, token.lexeme);
                }
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
    }
}
