//! Shared parser combinators for FHIRPath tokenization and parsing
//!
//! This module provides reusable Chumsky parser combinators that can be shared
//! between production and analysis parsers. This avoids duplication and ensures
//! consistent tokenization behavior.

use chumsky::prelude::*;
use chumsky::extra;
use rust_decimal::Decimal;

use crate::ast::{ExpressionNode, LiteralNode, LiteralValue, IdentifierNode, VariableNode};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

/// Parser for string literals with single quotes (supports multi-line and escapes)
pub fn string_literal_parser_single<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('\'')
        .ignore_then(
            // Support multi-line strings and proper escape sequences
            none_of(['\'', '\\'])
                .or(
                    just('\'').then(just('\'')).to('\'') // Handle escaped quotes (double single quote)
                )
                .or(
                    just('\\').ignore_then(choice((
                        just('\\').to('\\'),
                        just('n').to('\n'),
                        just('t').to('\t'),
                        just('r').to('\r'),
                        just('\'').to('\''),
                        just('\"').to('\"'),
                        just('`').to('`'),    // backtick escape
                        just('f').to('\x0C'), // form feed
                        just('/').to('/'),    // forward slash
                        // Simple unicode replacement (simplified for now)
                        just('u').ignore_then(
                            one_of("0123456789abcdefABCDEF").repeated().exactly(4).ignored()
                        ).to('?') // placeholder for unicode
                    )))
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

/// Parser for string literals with double quotes (supports multi-line and escapes) 
pub fn string_literal_parser_double<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('"')
        .ignore_then(
            // Support multi-line strings and comprehensive escape sequences
            none_of(['"', '\\'])
                .or(
                    just('\\').ignore_then(choice((
                        just('"').to('"'),
                        just('\'').to('\''),
                        just('\\').to('\\'),
                        just('n').to('\n'),
                        just('t').to('\t'),
                        just('r').to('\r'),
                        just('f').to('\x0C'), // form feed
                        just('/').to('/'),    // forward slash
                        // Simple unicode replacement (simplified for now)
                        just('u').ignore_then(
                            one_of("0123456789abcdefABCDEF").repeated().exactly(4).ignored()
                        ).to('?') // placeholder for unicode
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

/// Parser for integer and decimal numbers (with optional units for quantities)
pub fn number_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    // Handle both regular integers and leading-zero decimals properly
    choice((
        // Handle decimals with leading zeros like 0.0034
        just("0.").ignore_then(one_of("0123456789").repeated().at_least(1).collect::<String>())
            .map(|frac_part| format!("0.{}", frac_part)),
        // Handle regular integers and decimals  
        one_of("0123456789").repeated().at_least(1).collect::<String>()
            .then(just('.').ignore_then(one_of("0123456789").repeated().at_least(1).collect::<String>()).or_not())
            .map(|(int_part, frac_part)| {
                if let Some(frac) = frac_part {
                    format!("{}.{}", int_part, frac)
                } else {
                    int_part
                }
            })
    ))
    .then(
        // Enhanced unit specification - supports both quoted ('mg') and unquoted units (days, hours, etc.)
        just(' ').repeated().at_least(0).ignore_then(
            choice((
                // Quoted units like 'mg', 'kg'
                just('\'')
                    .ignore_then(
                        none_of(['\'']).repeated().collect::<String>()
                    )
                    .then_ignore(just('\'')),
                // Unquoted units like days, hours, weeks, months, years
                choice((
                    just("days").to("days".to_string()),
                    just("day").to("day".to_string()),
                    just("hours").to("hours".to_string()),
                    just("hour").to("hour".to_string()),
                    just("minutes").to("minutes".to_string()),
                    just("minute").to("minute".to_string()),
                    just("seconds").to("seconds".to_string()),
                    just("second").to("second".to_string()),
                    just("weeks").to("weeks".to_string()),
                    just("week").to("week".to_string()),
                    just("months").to("months".to_string()),
                    just("month").to("month".to_string()),
                    just("years").to("years".to_string()),
                    just("year").to("year".to_string()),
                ))
            ))
        )
        .or_not()
    )
    .map(|(number_str, unit): (String, Option<String>)| {
        // Try to parse as decimal first (handles both integers and decimals)
        match number_str.parse::<Decimal>() {
            Ok(decimal) => {
                if let Some(unit_str) = unit {
                    // Create Quantity literal
                    ExpressionNode::Literal(LiteralNode {
                        value: LiteralValue::Quantity { 
                            value: decimal,
                            unit: Some(unit_str)
                        },
                        location: None,
                    })
                } else if number_str.contains('.') {
                    // Plain decimal
                    ExpressionNode::Literal(LiteralNode {
                        value: LiteralValue::Decimal(decimal),
                        location: None,
                    })
                } else {
                    // Try as integer first for whole numbers
                    match number_str.parse::<i64>() {
                        Ok(num) => ExpressionNode::Literal(LiteralNode {
                            value: LiteralValue::Integer(num),
                            location: None,
                        }),
                        Err(_) => ExpressionNode::Literal(LiteralNode {
                            value: LiteralValue::Decimal(decimal),
                            location: None,
                        })
                    }
                }
            },
            Err(_) => ExpressionNode::Literal(LiteralNode {
                value: LiteralValue::String(number_str),
                location: None,
            })
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

/// Parser for DateTime literals (@2021-01-01, @T15:30:00, @2021-01-01T15:30:00Z, @2015-02-04T14:34:28.123)
pub fn datetime_literal_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('@')
        .ignore_then(
            choice((
                // Time only: @T15:30:00 (must be first to avoid conflicts)
                time_only_parser(),
                // Full DateTime with timezone: @2021-01-01T15:30:00Z (must come before date_only)
                datetime_full_parser(),
                // Date only: @2021-01-01 or @2015-02 or @2021 (must be last)
                date_only_parser(),  
            ))
        )
}

/// Parse date format string (YYYY-MM-DD, YYYY-MM, or YYYY)
fn date_format_str<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    // Parse exactly 4 digits for year (YYYY)
    one_of("0123456789").repeated().exactly(4).collect::<String>()
        .then(
            just('-').ignore_then(
                // Parse exactly 2 digits for month (MM)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            ).or_not()
        )
        .then(
            just('-').ignore_then(
                // Parse exactly 2 digits for day (DD)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            ).or_not()
        )
        .map(|((year, month), day)| {
            if let Some(day) = day {
                format!("{}-{}-{}", year, month.unwrap(), day)
            } else if let Some(month) = month {
                format!("{}-{}", year, month)
            } else {
                format!("{}", year)
            }
        })
}

/// Parse time format string (HH:MM:SS.sss, HH:MM:SS, HH:MM, or HH)
fn time_format_str<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    // Parse exactly 2 digits for hour (HH)
    one_of("0123456789").repeated().exactly(2).collect::<String>()
        .then(
            just(':').ignore_then(
                // Parse exactly 2 digits for minute (MM)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            ).or_not()
        )
        .then(
            just(':').ignore_then(
                // Parse exactly 2 digits for second (SS)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            ).or_not()
        )
        .then(
            just('.').ignore_then(
                // Parse 1-3 digits for milliseconds (s, ss, or sss)
                one_of("0123456789").repeated().at_least(1).at_most(3).collect::<String>()
            ).or_not()
        )
        .map(|(((hour, minute), second), millis)| {
            let mut time_str = format!("{}", hour);
            if let Some(min) = minute {
                time_str.push_str(&format!(":{}", min));
                if let Some(sec) = second {
                    time_str.push_str(&format!(":{}", sec));
                    if let Some(ms) = millis {
                        // Pad milliseconds to 3 digits if needed
                        time_str.push_str(&format!(".{:0<3}", ms));
                    }
                }
            }
            time_str
        })
}

/// Parse timezone format string (Z, +HH:MM, -HH:MM)
fn timezone_format_str<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just('Z').to("Z".to_string()),
        just('+').or(just('-'))
            .then(
                // Parse exactly 2 digits for timezone hour (HH)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            )
            .then_ignore(just(':'))
            .then(
                // Parse exactly 2 digits for timezone minute (MM)
                one_of("0123456789").repeated().exactly(2).collect::<String>()
            )
            .map(|((sign, hours), mins)| format!("{}{}:{}", sign, hours, mins)),
    ))
}

/// Parse full datetime format: 2021-01-01T15:30:00Z
fn datetime_full_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    // Use date format parser followed by T separator and time
    date_format_str()
        .then_ignore(just('T'))
        .then(time_format_str())
        .then(timezone_format_str().or_not())
        .try_map(|((date_str, time_str), tz_opt), span| {
            let full_str = if let Some(tz) = tz_opt {
                format!("{}T{}{}", date_str, time_str, tz)
            } else {
                format!("{}T{}", date_str, time_str)
            };

            // Use temporal module for precision-aware parsing
            PrecisionDateTime::parse(&full_str)
                .ok_or_else(|| Rich::custom(span, format!("Invalid datetime format: {}", full_str)))
                .map(|precision_dt| ExpressionNode::Literal(LiteralNode {
                    value: LiteralValue::DateTime(precision_dt),
                    location: None,
                }))
        })
}

/// Parse date only format: 2021-01-01, 2021-01, or 2021 (only if not followed by T)
fn date_only_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    date_format_str()
        .then_ignore(just('T').not()) // Ensure we don't have a T after (would be a datetime)
        .try_map(|date_str, span| {
            // Use temporal module for precision-aware parsing
            PrecisionDate::parse(&date_str)
                .ok_or_else(|| Rich::custom(span, format!("Invalid date format: {}", date_str)))
                .map(|precision_date| ExpressionNode::Literal(LiteralNode {
                    value: LiteralValue::Date(precision_date),
                    location: None,
                }))
        })
}

/// Parse time only format: T15:30:00
fn time_only_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('T')
        .ignore_then(time_format_str())
        .try_map(|time_str, span| {
            // Use temporal module for precision-aware parsing
            PrecisionTime::parse(&time_str)
                .ok_or_else(|| Rich::custom(span, format!("Invalid time format: {}", time_str)))
                .map(|precision_time| ExpressionNode::Literal(LiteralNode {
                    value: LiteralValue::Time(precision_time),
                    location: None,
                }))
        })
}


/// Parser for backtick-delimited identifiers (`identifier`)
pub fn backtick_identifier_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    just('`')
        .ignore_then(
            none_of(['`', '\n', '\r'])
                .repeated()
                .at_least(1)
                .collect::<String>()
        )
        .then_ignore(just('`'))
        .map(|name: String| ExpressionNode::Identifier(IdentifierNode {
            name,
            location: None,
        }))
}

/// Parser for identifiers (including keywords and backtick-delimited)
pub fn identifier_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        backtick_identifier_parser(),
        text::ident().map(|name: &str| ExpressionNode::Identifier(IdentifierNode {
            name: name.to_string(),
            location: None,
        }))
    ))
}

/// Parser for variable references ($variable or %variable)
pub fn variable_parser<'a>() -> impl Parser<'a, &'a str, ExpressionNode, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        // Standard $variable syntax
        just('$')
            .ignore_then(text::ident())
            .map(|name: &str| ExpressionNode::Variable(VariableNode {
                name: name.to_string(),
                location: None,
            })),
        // FHIRPath %variable syntax (context variables)
        just('%')
            .ignore_then(choice((
                // Support backtick-quoted variable names %`ext-patient-birthTime`
                just('`')
                    .ignore_then(
                        none_of(['`', '\n', '\r'])
                            .repeated()
                            .at_least(1)
                            .collect::<String>()
                    )
                    .then_ignore(just('`')),
                // Standard identifier variable names
                text::ident().map(|s: &str| s.to_string())
            )))
            .map(|name: String| ExpressionNode::Variable(VariableNode {
                name,
                location: None,
            })),
    ))
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
        just("xor"),     // Added XOR operator
        just("implies"), // Added IMPLIES operator
        just("not"),
        just("in"),
        just("contains"),
        just("div"),
        just("mod"),
        just("is"),      // Added type operators
        just("as"),
    ))
}

/// Parser for whitespace and comments (for analysis mode)
pub fn whitespace_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    one_of(" \t\n\r")
        .repeated()
        .at_least(1)
        .collect::<String>()
}

/// Parser for line comments (single-line and multi-line)
pub fn comment_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        // Single-line comment: // comment
        just("//")
            .ignore_then(
                none_of("\n\r")
                    .repeated()
                    .collect::<String>()
            )
            .map(|comment| comment.trim().to_string()),
        // Multi-line comment: /* comment */
        just("/*")
            .ignore_then(
                // Match any character until we find */
                none_of("*")
                    .or(just('*').then(none_of("/")).to('*'))
                    .repeated()
                    .collect::<String>()
            )
            .then_ignore(just("*/"))
            .map(|comment| comment.trim().to_string()),
    ))
}

/// Parser that consumes comments and whitespace (for filtering)
pub fn comment_or_whitespace<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    choice((
        comment_parser().ignored(),
        whitespace_parser().ignored(),
    ))
}

/// Parser for HTML entities (commonly found in test data)
pub fn html_entity_parser<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        just("&lt;").to("<"),
        just("&gt;").to(">"),
        just("&amp;").to("&"),
        just("&quot;").to("\""),
        just("&apos;").to("'"),
    ))
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
    fn test_backtick_identifier_parser() {
        let parser = backtick_identifier_parser();
        let result = parser.parse("`given`").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Identifier(id)) = result {
            assert_eq!(id.name, "given");
        }
    }

    #[test]
    fn test_backtick_identifier_with_special_chars() {
        let parser = backtick_identifier_parser();
        let result = parser.parse("`PID-1`").into_result();
        
        assert!(result.is_ok());
        if let Ok(ExpressionNode::Identifier(id)) = result {
            assert_eq!(id.name, "PID-1");
        }
    }

    #[test]
    fn test_identifier_parser_choice() {
        let parser = identifier_parser();
        
        // Test regular identifier
        let result1 = parser.parse("Patient").into_result();
        assert!(result1.is_ok());
        if let Ok(ExpressionNode::Identifier(id)) = result1 {
            assert_eq!(id.name, "Patient");
        }
        
        // Test backtick identifier
        let result2 = parser.parse("`given`").into_result();
        assert!(result2.is_ok());
        if let Ok(ExpressionNode::Identifier(id)) = result2 {
            assert_eq!(id.name, "given");
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