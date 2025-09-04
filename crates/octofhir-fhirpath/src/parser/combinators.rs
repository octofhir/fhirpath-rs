//! Shared parser combinators for FHIRPath tokenization and parsing
//!
//! This module provides reusable Chumsky parser combinators that can be shared
//! between production and analysis parsers. This avoids duplication and ensures
//! consistent tokenization behavior.

use chumsky::prelude::*;
use chumsky::extra;
use rust_decimal::Decimal;

use crate::ast::{ExpressionNode, LiteralNode, LiteralValue, IdentifierNode, VariableNode};

/// Parser for string literals with single quotes
pub fn string_literal_parser_single<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('\'')
        .ignore_then(
            none_of(['\'', '\\', '\n', '\r'])
                .or(
                    just('\'').then(just('\'')).to('\'') // Handle escaped quotes (double single quote)
                )
                .repeated()
                .collect::<String>()
        )
        .then_ignore(just('\''))
        .map(|s: String| ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String(s),
            location: None,
        }))
}

/// Parser for string literals with double quotes  
pub fn string_literal_parser_double<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('"')
        .ignore_then(
            none_of(['"', '\\', '\n', '\r'])
                .or(
                    just('\\').ignore_then(choice((
                        just('"').to('"'),
                        just('\'').to('\''),
                        just('\\').to('\\'),
                        just('n').to('\n'),
                        just('t').to('\t'),
                        just('r').to('\r'),
                    )))
                )
                .repeated()
                .collect::<String>()
        )
        .then_ignore(just('"'))
        .map(|s: String| ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String(s),
            location: None,
        }))
}

/// Combined string literal parser (single or double quotes)
pub fn string_literal_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        string_literal_parser_single(),
        string_literal_parser_double(),
    ))
}

/// Parser for integer and decimal numbers
pub fn number_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    text::int(10)
        .then(just('.').ignore_then(text::int(10)).or_not())
        .map(|(int_part, decimal_part): (&str, Option<&str>)| {
            if let Some(dec_part) = decimal_part {
                let full_number = format!("{}.{}", int_part, dec_part);
                match full_number.parse::<Decimal>() {
                    Ok(decimal) => ExpressionNode::Literal(LiteralNode {
                        value: LiteralValue::Decimal(decimal),
                        location: None,
                    }),
                    Err(_) => ExpressionNode::Literal(LiteralNode {
                        value: LiteralValue::String(full_number),
                        location: None,
                    })
                }
            } else {
                match int_part.parse::<i64>() {
                    Ok(num) => ExpressionNode::Literal(LiteralNode {
                        value: LiteralValue::Integer(num),
                        location: None,
                    }),
                    Err(_) => match int_part.parse::<Decimal>() {
                        Ok(decimal) => ExpressionNode::Literal(LiteralNode {
                            value: LiteralValue::Decimal(decimal),
                            location: None,
                        }),
                        Err(_) => ExpressionNode::Literal(LiteralNode {
                            value: LiteralValue::String(int_part.to_string()),
                            location: None,
                        })
                    },
                }
            }
        })
}

/// Parser for boolean literals
pub fn boolean_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just("true").to(ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Boolean(true),
            location: None,
        })),
        just("false").to(ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Boolean(false),
            location: None,
        })),
    ))
}

/// Parser for DateTime literals (@2021-01-01, @T15:30:00, @2021-01-01T15:30:00Z)
pub fn datetime_literal_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('@')
        .ignore_then(
            one_of("0123456789-:TZ+.")
                .repeated()
                .at_least(1)
                .collect::<String>()
        )
        .map(|datetime_str: String| {
            // For now, store as string - proper DateTime parsing would go here
            ExpressionNode::Literal(LiteralNode {
                value: LiteralValue::String(format!("@{}", datetime_str)),
                location: None,
            })
        })
}

/// Parser for identifiers (including keywords)
pub fn identifier_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    text::ident()
        .map(|name: &str| ExpressionNode::Identifier(IdentifierNode {
            name: name.to_string(),
            location: None,
        }))
}

/// Parser for variable references ($variable)
pub fn variable_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('$')
        .ignore_then(text::ident())
        .map(|name: &str| ExpressionNode::Variable(VariableNode {
            name: name.to_string(),
            location: None,
        }))
}

/// Parser for all literal types (combined)
pub fn literal_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        string_literal_parser(),
        datetime_literal_parser(),
        number_parser(),
        boolean_parser(),
    ))
}

/// Parser for operators - equality
pub fn equals_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just("==").to("="), // Accept == as = (common mistake)
        just("="),
    ))
}

/// Parser for operators - not equals
pub fn not_equals_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just("!=").to("!="),
        just("<>").to("!="), // SQL style, normalized to !=
    ))
}

/// Parser for operators - less than or equal
pub fn less_equal_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    just("<=")
}

/// Parser for operators - greater than or equal  
pub fn greater_equal_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    just(">=")
}

/// Parser for single character operators
pub fn single_char_operators<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    one_of("<>~+-*/|()[].,;")
}

/// Parser for FHIRPath keywords
pub fn keyword_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just("and"),
        just("or"), 
        just("not"),
        just("in"),
        just("contains"),
        just("div"),
        just("mod"),
    ))
}

/// Parser for whitespace and comments (for analysis mode)
pub fn whitespace_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    one_of(" \t\n\r")
        .repeated()
        .at_least(1)
        .collect::<String>()
}

/// Parser for line comments
pub fn comment_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    just("//")
        .ignore_then(
            none_of("\n\r")
                .repeated()
                .collect::<String>()
        )
        .map(|comment| comment.trim().to_string())
}

// The atom_parser is not needed as each parser can directly use the individual combinators

/// Enhanced error recovery parser that tries to continue parsing after errors
pub fn error_recovery_parser<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    // Skip invalid characters until we find something we can parse
    none_of(" \t\n\r()[]{},.;")
        .repeated()
        .ignored()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_literal_single_quotes() {
        let parser = string_literal_parser_single();
        let result = parser.parse("'hello'").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Literal(lit)) = result {
            if let LiteralValue::String(s) = lit.value {
                assert_eq!(s, "hello");
            } else {
                panic!("Expected string literal");
            }
        }
    }

    #[test]
    fn test_string_literal_escaped_quotes() {
        let parser = string_literal_parser_single();
        let result = parser.parse("'can''t'").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Literal(lit)) = result {
            if let LiteralValue::String(s) = lit.value {
                assert_eq!(s, "can't");
            } else {
                panic!("Expected string literal with escaped quote");
            }
        }
    }

    #[test]
    fn test_number_parser_integer() {
        let parser = number_parser();
        let result = parser.parse("42").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Literal(lit)) = result {
            if let LiteralValue::Integer(n) = lit.value {
                assert_eq!(n, 42);
            } else {
                panic!("Expected integer literal");
            }
        }
    }

    #[test]
    fn test_number_parser_decimal() {
        let parser = number_parser();
        let result = parser.parse("3.14").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Literal(lit)) = result {
            matches!(lit.value, LiteralValue::Decimal(_));
        }
    }

    #[test]
    fn test_boolean_parser() {
        let parser = boolean_parser();
        
        let true_result = parser.parse("true").into_result();
        assert!(true_result.is_ok());
        
        let false_result = parser.parse("false").into_result();
        assert!(false_result.is_ok());
    }

    #[test]
    fn test_datetime_literal_parser() {
        let parser = datetime_literal_parser();
        let result = parser.parse("@2021-01-01").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Literal(lit)) = result {
            if let LiteralValue::String(s) = lit.value {
                assert_eq!(s, "@2021-01-01");
            } else {
                panic!("Expected datetime literal");
            }
        }
    }

    #[test]
    fn test_identifier_parser() {
        let parser = identifier_parser();
        let result = parser.parse("Patient").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Identifier(id)) = result {
            assert_eq!(id.name, "Patient");
        }
    }

    #[test]
    fn test_variable_parser() {
        let parser = variable_parser();
        let result = parser.parse("$this").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Variable(var)) = result {
            assert_eq!(var.name, "this");
        }
    }

    #[test]
    fn test_equals_parser_variations() {
        let parser = equals_parser();
        
        assert_eq!(parser.parse("=").into_result(), Ok("="));
        assert_eq!(parser.parse("==").into_result(), Ok("="));
    }

    #[test]
    fn test_not_equals_parser_variations() {
        let parser = not_equals_parser();
        
        assert_eq!(parser.parse("!=").into_result(), Ok("!="));
        assert_eq!(parser.parse("<>").into_result(), Ok("!="));
    }

    #[test]
    fn test_keyword_parser() {
        let parser = keyword_parser();
        
        assert_eq!(parser.parse("and").into_result(), Ok("and"));
        assert_eq!(parser.parse("or").into_result(), Ok("or"));
        assert_eq!(parser.parse("not").into_result(), Ok("not"));
        assert_eq!(parser.parse("in").into_result(), Ok("in"));
        assert_eq!(parser.parse("contains").into_result(), Ok("contains"));
        assert_eq!(parser.parse("div").into_result(), Ok("div"));
        assert_eq!(parser.parse("mod").into_result(), Ok("mod"));
    }

    #[test]
    fn test_comment_parser() {
        let parser = comment_parser();
        let result = parser.parse("// this is a comment").into_result();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a comment");
    }
}