//! FHIRPath expression parser
//!
//! This module provides parsing functionality for FHIRPath expressions using nom.

use crate::ast::{BinaryOperator, ExpressionNode, UnaryOperator};
use crate::error::{FhirPathError, Result};
use crate::model::FhirPathValue;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, opt, recognize},
    multi::separated_list0,
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use chrono::{NaiveDate, NaiveTime, DateTime, Utc};

/// Parse an FHIRPath expression string into an AST
pub fn parse_expression(input: &str) -> Result<ExpressionNode> {
    let trimmed = input.trim();

    // Handle empty expressions - return $this (current context)
    if trimmed.is_empty() {
        return Ok(ExpressionNode::identifier("$this"));
    }

    match expression.parse(trimmed) {
        Ok(("", ast)) => Ok(ast),
        Ok((remaining, _)) => Err(FhirPathError::parse_error(
            input.len() - remaining.len(),
            format!("Unexpected input: '{}'", remaining),
        )),
        Err(e) => Err(FhirPathError::parse_error(0, format!("Parse error: {}", e))),
    }
}

/// Parse a complete expression (top-level)
fn expression(input: &str) -> IResult<&str, ExpressionNode> {
    // Try to parse as a predicate expression first (expressions starting with operators)
    alt((
        predicate_expression,
        ws(or_expression),
    )).parse(input)
}

/// Parse predicate expressions that start with operators (implicit $this as left operand)
fn predicate_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, op) = ws(alt((
        // Equality operators
        map(tag("!~"), |_| BinaryOperator::NotEquivalent),
        map(tag("!="), |_| BinaryOperator::NotEqual),
        map(tag("~"), |_| BinaryOperator::Equivalent),
        map(tag("="), |_| BinaryOperator::Equal),
        // Relational operators
        map(tag(">="), |_| BinaryOperator::GreaterThanOrEqual),
        map(tag("<="), |_| BinaryOperator::LessThanOrEqual),
        map(tag(">"), |_| BinaryOperator::GreaterThan),
        map(tag("<"), |_| BinaryOperator::LessThan),
    ))).parse(input)?;

    // Parse the right operand
    let (input, right) = ws(or_expression).parse(input)?;

    // Create binary operation with implicit $this as left operand
    Ok((input, ExpressionNode::binary_op(
        op,
        ExpressionNode::identifier("$this"),
        right
    )))
}


/// Parse OR expressions (lowest precedence)
fn or_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = union_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(tag("or")),
        ws(union_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (_, expr)| {
            ExpressionNode::binary_op(BinaryOperator::Or, acc, expr)
        }),
    ))
}

/// Parse UNION expressions (|)
fn union_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = equality_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(char('|')),
        ws(equality_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (_, expr)| {
            ExpressionNode::binary_op(BinaryOperator::Union, acc, expr)
        }),
    ))
}

/// Parse equality expressions (=, !=, ~, !~, is)
fn equality_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = and_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(alt((
            map(tag("!~"), |_| BinaryOperator::NotEquivalent),
            map(tag("!="), |_| BinaryOperator::NotEqual),
            map(tag("~"), |_| BinaryOperator::Equivalent),
            map(tag("="), |_| BinaryOperator::Equal),
            map(tag("is"), |_| BinaryOperator::Is),
        ))),
        ws(and_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            ExpressionNode::binary_op(op, acc, expr)
        }),
    ))
}

/// Parse AND expressions, XOR expressions, IMPLIES expressions, and string concatenation
fn and_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = relational_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(alt((
            map(tag("implies"), |_| BinaryOperator::Implies),
            map(tag("and"), |_| BinaryOperator::And),
            map(tag("xor"), |_| BinaryOperator::Xor),
            map(char('&'), |_| BinaryOperator::Concatenate),
        ))),
        ws(relational_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            ExpressionNode::binary_op(op, acc, expr)
        }),
    ))
}

/// Parse relational expressions (>, <, >=, <=, in, contains)
fn relational_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = additive_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(alt((
            map(tag(">="), |_| BinaryOperator::GreaterThanOrEqual),
            map(tag("<="), |_| BinaryOperator::LessThanOrEqual),
            map(tag(">"), |_| BinaryOperator::GreaterThan),
            map(tag("<"), |_| BinaryOperator::LessThan),
            map(tag("contains"), |_| BinaryOperator::Contains),
            map(tag("in"), |_| BinaryOperator::In),
        ))),
        ws(additive_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            ExpressionNode::binary_op(op, acc, expr)
        }),
    ))
}

/// Parse additive expressions (+ and -)
fn additive_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = multiplicative_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(alt((
            map(char('+'), |_| BinaryOperator::Add),
            map(char('-'), |_| BinaryOperator::Subtract),
        ))),
        ws(multiplicative_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            ExpressionNode::binary_op(op, acc, expr)
        }),
    ))
}

/// Parse multiplicative expressions (*, /, div, mod)
fn multiplicative_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, first) = unary_expression(input)?;
    let (input, rest) = nom::multi::many0(pair(
        ws(alt((
            map(tag("div"), |_| BinaryOperator::Divide),
            map(tag("mod"), |_| BinaryOperator::Modulo),
            map(char('*'), |_| BinaryOperator::Multiply),
            map(char('/'), |_| BinaryOperator::Divide),
        ))),
        ws(unary_expression),
    )).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            ExpressionNode::binary_op(op, acc, expr)
        }),
    ))
}

/// Parse unary expressions (not, -, +)
fn unary_expression(input: &str) -> IResult<&str, ExpressionNode> {
    alt((
        map(
            pair(ws(tag("not")), ws(unary_expression)),
            |(_, expr)| ExpressionNode::unary_op(UnaryOperator::Not, expr),
        ),
        map(
            pair(ws(char('-')), ws(unary_expression)),
            |(_, expr)| ExpressionNode::unary_op(UnaryOperator::Minus, expr),
        ),
        map(
            pair(ws(char('+')), ws(unary_expression)),
            |(_, expr)| ExpressionNode::unary_op(UnaryOperator::Plus, expr),
        ),
        postfix_expression,
    )).parse(input)
}

/// Parse postfix expressions (function calls, property access, indexing, type operations)
fn postfix_expression(input: &str) -> IResult<&str, ExpressionNode> {
    let (input, mut expr) = primary_expression(input)?;
    let (input, postfixes) = nom::multi::many0(alt((
        // Function call: identifier()
        map(
            (
                ws(char('(')),
                separated_list0(ws(char(',')), ws(expression)),
                ws(char(')')),
            ),
            |(_, args, _)| PostfixOp::FunctionCall(args),
        ),
        // Property access: .identifier
        map(
            preceded(ws(char('.')), identifier),
            PostfixOp::PropertyAccess,
        ),
        // Index access: [expression]
        map(
            delimited(ws(char('[')), ws(expression), ws(char(']'))),
            PostfixOp::IndexAccess,
        ),
        // Type cast: as TypeName
        map(
            preceded(ws(tag("as")), ws(identifier)),
            PostfixOp::TypeCast,
        ),
    ))).parse(input)?;

    // Apply postfix operations left-to-right
    for postfix in postfixes {
        expr = match postfix {
            PostfixOp::FunctionCall(args) => {
                match expr {
                    ExpressionNode::Identifier(name) => {
                        ExpressionNode::function_call(name, args)
                    }
                    ExpressionNode::Path { base, path } => {
                        // This is a method call like someExpression.functionName()
                        // Create a function call where the base expression is the implicit context
                        // For now, we'll represent this as a function call with the base as the first argument
                        let mut method_args = vec![*base];
                        method_args.extend(args);
                        ExpressionNode::function_call(path, method_args)
                    }
                    _ => {
                        // For other expression types, we can't determine the function name
                        // This happens when we have something like @2014-12-14() which is invalid
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                }
            }
            PostfixOp::PropertyAccess(property) => {
                ExpressionNode::path(expr, property)
            }
            PostfixOp::IndexAccess(index) => {
                ExpressionNode::index(expr, index)
            }
            PostfixOp::TypeCast(type_name) => {
                ExpressionNode::type_cast(expr, type_name)
            }
        };
    }

    Ok((input, expr))
}

/// Postfix operation types
#[derive(Debug, Clone)]
enum PostfixOp {
    FunctionCall(Vec<ExpressionNode>),
    PropertyAccess(String),
    IndexAccess(ExpressionNode),
    TypeCast(String),
}

/// Parse primary expressions (literals, identifiers, parenthesized expressions)
fn primary_expression(input: &str) -> IResult<&str, ExpressionNode> {
    ws(alt((
        // Parenthesized expression
        delimited(char('('), ws(expression), char(')')),
        // Literals
        literal,
        // Identifiers
        map(identifier, ExpressionNode::identifier),
    ))).parse(input)
}

/// Parse date/time literals (starting with @)
fn datetime_literal(input: &str) -> IResult<&str, ExpressionNode> {
    // Custom datetime parser that's more careful about dots
    let (input, _) = char('@')(input)?;
    
    // Parse the datetime part more carefully
    let mut chars = input.chars().peekable();
    let mut consumed = 0;
    let mut last_was_digit = false;
    let mut has_time_part = false;
    
    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' => {
                chars.next();
                consumed += ch.len_utf8();
                last_was_digit = true;
            }
            '-' | ':' => {
                chars.next();
                consumed += ch.len_utf8();
                last_was_digit = false;
                if ch == ':' {
                    has_time_part = true;
                }
            }
            'T' | 't' => {
                chars.next();
                consumed += ch.len_utf8();
                last_was_digit = false;
                has_time_part = true;
            }
            'Z' | 'z' | '+' => {
                chars.next();
                consumed += ch.len_utf8();
                last_was_digit = false;
            }
            '.' => {
                // Only consume dot if:
                // 1. We're in a time part (has_time_part is true)
                // 2. The last character was a digit
                // 3. The next character is a digit (fractional seconds)
                if has_time_part && last_was_digit {
                    let mut peek_chars = chars.clone();
                    peek_chars.next(); // skip the dot
                    if let Some(&next_ch) = peek_chars.peek() {
                        if next_ch.is_ascii_digit() {
                            chars.next(); // consume the dot
                            consumed += ch.len_utf8();
                            last_was_digit = false;
                            continue;
                        }
                    }
                }
                // Don't consume the dot - it's likely a method call
                break;
            }
            _ => break,
        }
    }
    
    let datetime_str = &input[..consumed];
    let remaining = &input[consumed..];
    
    // Now parse the datetime string
    let node = {
        let s = datetime_str;
        // Try to parse as different date/time formats

        // Try DateTime formats first (most specific)
        // Try RFC3339 format first
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        // Try with timezone offsets
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f%z") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f%:z") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%:z") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        // Try UTC formats
        else if let Ok(dt) = DateTime::parse_from_rfc3339(&format!("{}Z", s)) {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3fZ") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }
        else if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
            ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)))
        }

                // Try without timezone (assume UTC)
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f") {
                    ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)));
                }
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                    ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)));
                }
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M") {
                    ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)));
                }
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H") {
                    ExpressionNode::literal(FhirPathValue::DateTime(dt.with_timezone(&Utc)));
                }

                // Try Time formats
                if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S%.3f") {
                    ExpressionNode::literal(FhirPathValue::Time(time));
                }
                if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
                    ExpressionNode::literal(FhirPathValue::Time(time));
                }
                if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
                    ExpressionNode::literal(FhirPathValue::Time(time));
                }

                // Try Time formats with T prefix (e.g., "T14", "T14:34", "T14:34:28")
                if s.starts_with('T') {
                    let time_part = &s[1..]; // Remove the 'T' prefix
                    if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M:%S%.3f") {
                        ExpressionNode::literal(FhirPathValue::Time(time));
                    }
                    if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M:%S") {
                        ExpressionNode::literal(FhirPathValue::Time(time));
                    }
                    if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M") {
                        ExpressionNode::literal(FhirPathValue::Time(time));
                    }
                    // Handle hour-only format (e.g., "T14")
                    if time_part.len() == 2 && time_part.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(hour) = time_part.parse::<u32>() {
                            if hour < 24 {
                                if let Some(time) = NaiveTime::from_hms_opt(hour, 0, 0) {
                                    ExpressionNode::literal(FhirPathValue::Time(time));
                                }
                            }
                        }
                    }
                }

                // Try Date formats (least specific)
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    ExpressionNode::literal(FhirPathValue::Date(date));
                }

                // Handle year-month format (e.g., "2015-03")
                if s.len() == 7 && s.chars().nth(4) == Some('-') {
                    if let (Ok(year), Ok(month)) = (s[0..4].parse::<i32>(), s[5..7].parse::<u32>()) {
                        if month >= 1 && month <= 12 {
                            if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
                                ExpressionNode::literal(FhirPathValue::Date(date));
                            }
                        }
                    }
                }

                // Handle partial DateTime formats with 'T'
                if s.contains('T') {
                    // Handle year + T format (e.g., "2015T")
                    if s.len() == 5 && s.ends_with('T') && s[0..4].chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(year) = s[0..4].parse::<i32>() {
                            if year >= 1 && year <= 9999 {
                                if let Some(date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                                    let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                                    ExpressionNode::literal(FhirPathValue::DateTime(datetime));
                                }
                            }
                        }
                    }

                    // Handle year-month + T format (e.g., "2015-02T")
                    if s.len() == 8 && s.ends_with('T') && s.chars().nth(4) == Some('-') {
                        let date_part = &s[0..7]; // "2015-02"
                        if let (Ok(year), Ok(month)) = (date_part[0..4].parse::<i32>(), date_part[5..7].parse::<u32>()) {
                            if year >= 1 && year <= 9999 && month >= 1 && month <= 12 {
                                if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
                                    let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                                    ExpressionNode::literal(FhirPathValue::DateTime(datetime));
                                }
                            }
                        }
                    }

                    // Handle year-month-day + T format (e.g., "2015-02-04T")
                    if s.len() == 11 && s.ends_with('T') {
                        let date_part = &s[0..10]; // "2015-02-04"
                        if let Ok(date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                            let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                            ExpressionNode::literal(FhirPathValue::DateTime(datetime));
                        }
                    }
                }

                // Handle year-only format (e.g., "2015")
                if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(year) = s.parse::<i32>() {
                        if year >= 1 && year <= 9999 {
                            if let Some(date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                                ExpressionNode::literal(FhirPathValue::Date(date));
                            }
                        }
                    }
                }

                // If all parsing fails, treat as string
                ExpressionNode::literal(FhirPathValue::String(format!("@{}", s)))
    };
    
    Ok((remaining, node))
}

/// Parse literals (numbers, strings, booleans, date/time, collections, quantities)
fn literal(input: &str) -> IResult<&str, ExpressionNode> {
    alt((
        // Date/time literals (must come first to handle @ prefix)
        datetime_literal,
        // Empty collection literal
        map(
            delimited(ws(char('{')), multispace0, ws(char('}'))),
            |_| ExpressionNode::literal(FhirPathValue::Collection(vec![]))
        ),
        // Boolean literals
        map(tag("true"), |_| {
            ExpressionNode::literal(FhirPathValue::Boolean(true))
        }),
        map(tag("false"), |_| {
            ExpressionNode::literal(FhirPathValue::Boolean(false))
        }),
        // String literals
        string_literal,
        // Quantity literals (must come before numeric literals)
        quantity_literal,
        // Numeric literals
        numeric_literal,
    )).parse(input)
}

/// Parse string literals (single or double-quoted) with escape sequence support
fn string_literal(input: &str) -> IResult<&str, ExpressionNode> {
    use nom::bytes::complete::{take_while_m_n};
    use nom::character::complete::satisfy;
    use nom::multi::many0;

    alt((
        // Double-quoted strings
        map(
            delimited(
                char('"'),
                many0(alt((
                    // Escape sequences
                    preceded(
                        char('\\'),
                        alt((
                            map(char('n'), |_| '\n'),
                            map(char('t'), |_| '\t'),
                            map(char('r'), |_| '\r'),
                            map(char('f'), |_| '\x0C'), // Form feed
                            map(char('/'), |_| '/'),    // Forward slash
                            map(char('\\'), |_| '\\'),
                            map(char('"'), |_| '"'),
                            map(char('\''), |_| '\''),
                            map(char('`'), |_| '`'),    // Backtick
                            // Unicode escape: \uXXXX
                            map(
                                preceded(
                                    char('u'),
                                    take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit())
                                ),
                                |hex_str: &str| {
                                    if let Ok(code_point) = u32::from_str_radix(hex_str, 16) {
                                        if let Some(ch) = char::from_u32(code_point) {
                                            ch
                                        } else {
                                            '?' // Invalid unicode code point
                                        }
                                    } else {
                                        '?' // Invalid hex
                                    }
                                }
                            ),
                        ))
                    ),
                    // Regular characters (not quote or backslash)
                    satisfy(|c| c != '"' && c != '\\'),
                ))),
                char('"'),
            ),
            |chars: Vec<char>| ExpressionNode::literal(FhirPathValue::String(chars.into_iter().collect())),
        ),
        // Single-quoted strings
        map(
            delimited(
                char('\''),
                many0(alt((
                    // Escape sequences
                    preceded(
                        char('\\'),
                        alt((
                            map(char('n'), |_| '\n'),
                            map(char('t'), |_| '\t'),
                            map(char('r'), |_| '\r'),
                            map(char('f'), |_| '\x0C'), // Form feed
                            map(char('/'), |_| '/'),    // Forward slash
                            map(char('\\'), |_| '\\'),
                            map(char('"'), |_| '"'),
                            map(char('\''), |_| '\''),
                            map(char('`'), |_| '`'),    // Backtick
                            // Unicode escape: \uXXXX
                            map(
                                preceded(
                                    char('u'),
                                    take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit())
                                ),
                                |hex_str: &str| {
                                    if let Ok(code_point) = u32::from_str_radix(hex_str, 16) {
                                        if let Some(ch) = char::from_u32(code_point) {
                                            ch
                                        } else {
                                            '?' // Invalid unicode code point
                                        }
                                    } else {
                                        '?' // Invalid hex
                                    }
                                }
                            ),
                        ))
                    ),
                    // Regular characters (not quote or backslash)
                    satisfy(|c| c != '\'' && c != '\\'),
                ))),
                char('\''),
            ),
            |chars: Vec<char>| ExpressionNode::literal(FhirPathValue::String(chars.into_iter().collect())),
        ),
    )).parse(input)
}

/// Parse quantity literals (numbers with units like 5 'mg' or 10 days)
fn quantity_literal(input: &str) -> IResult<&str, ExpressionNode> {
    use nom::bytes::complete::take_while1;

    map(
        (
            // Parse the numeric part
            recognize((
                opt(char('-')),
                digit1,
                opt((char('.'), digit1)),
            )),
            // Parse whitespace
            multispace0,
            // Parse the unit (either quoted string or unquoted identifier)
            alt((
                // Quoted unit like 'mg'
                delimited(
                    char('\''),
                    take_while1(|c: char| c != '\''),
                    char('\'')
                ),
                // Unquoted unit like days, years, etc.
                take_while1(|c: char| c.is_alphabetic())
            ))
        ),
        |(num_str, _, unit_str): (&str, &str, &str)| {
            // Parse the numeric value
            let value = if num_str.contains('.') {
                // Decimal
                Decimal::from_str(num_str).unwrap_or_default()
            } else {
                // Integer converted to decimal
                Decimal::from_str(num_str).unwrap_or_default()
            };

            ExpressionNode::literal(FhirPathValue::Quantity {
                value,
                unit: Some(unit_str.to_string()),
                ucum_expr: None,
            })
        }
    ).parse(input)
}

/// Parse numeric literals (integers and decimals)
fn numeric_literal(input: &str) -> IResult<&str, ExpressionNode> {
    map(
        recognize((
            opt(char('-')),
            digit1,
            opt((char('.'), digit1)),
        )),
        |s: &str| {
            if s.contains('.') {
                // Decimal
                if let Ok(decimal) = Decimal::from_str(s) {
                    ExpressionNode::literal(FhirPathValue::Decimal(decimal))
                } else {
                    ExpressionNode::literal(FhirPathValue::String(s.to_string()))
                }
            } else {
                // Integer
                if let Ok(int) = s.parse::<i64>() {
                    ExpressionNode::literal(FhirPathValue::Integer(int))
                } else {
                    ExpressionNode::literal(FhirPathValue::String(s.to_string()))
                }
            }
        },
    ).parse(input)
}

/// Parse identifiers (including backtick-quoted identifiers, $-prefixed identifiers, and %-prefixed variables)
fn identifier(input: &str) -> IResult<&str, String> {
    alt((
        // Backtick-quoted identifiers
        map(
            delimited(
                char('`'),
                take_while1(|c: char| c != '`'),
                char('`'),
            ),
            |s: &str| s.to_string(),
        ),
        // Percent-prefixed variables like %sct, %loinc, %ucum, %`ext-patient-birthTime`, etc.
        alt((
            // % followed by backtick-quoted identifier
            map(
                recognize((
                    char('%'),
                    char('`'),
                    take_while1(|c: char| c != '`'),
                    char('`'),
                )),
                |s: &str| s.to_string(),
            ),
            // % followed by regular identifier
            map(
                recognize((
                    char('%'),
                    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
                )),
                |s: &str| s.to_string(),
            ),
        )),
        // Dollar-prefixed identifiers like $this, $index, etc.
        map(
            recognize((
                char('$'),
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            )),
            |s: &str| s.to_string(),
        ),
        // Regular identifiers
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |s: &str| s.to_string(),
        ),
    )).parse(input)
}

/// Whitespace consumer
fn ws<'a, F, O>(inner: F) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    F: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_failing_expressions() {
        // Test the specific failing expressions from the test suite
        let expressions = vec![
            "Patient.name.given.where(substring($this.length()-3) = 'ter')",
            "Patient.name.where($this.given = 'Jim').count() = 1",
            "where(substring($this.length()-3) = 'ter')",
            "$this.length()",
            "substring($this.length()-3)",
            "$this.given = 'Jim'",
        ];

        for expr in expressions {
            println!("Testing expression: {}", expr);
            match parse_expression(expr) {
                Ok(ast) => println!("  SUCCESS: {:?}", ast),
                Err(e) => println!("  ERROR: {:?}", e),
            }
            println!();
        }
    }

    #[test]
    fn test_new_operators() {
        // Test the newly added operators
        let expressions = vec![
            "'b' in ('a' | 'c' | 'd')",
            "(1|2|3) contains 1",
            "true xor false",
            "(true xor true) = false",
            "true implies false",
            "(false implies true) = true",
            "{}",
            "{}.empty()",
            "10 div 2",
            "10 mod 3",
            "10 div 1 = 10",
        ];

        for expr in expressions {
            println!("Testing new operator expression: {}", expr);
            match parse_expression(expr) {
                Ok(ast) => println!("  SUCCESS: {:?}", ast),
                Err(e) => println!("  ERROR: {:?}", e),
            }
            println!();
        }
    }

    #[test]
    fn test_parse_literal() {
        let result = parse_expression("42").unwrap();
        assert_eq!(result, ExpressionNode::literal(FhirPathValue::Integer(42)));

        let result = parse_expression("true").unwrap();
        assert_eq!(result, ExpressionNode::literal(FhirPathValue::Boolean(true)));

        let result = parse_expression("'hello'").unwrap();
        assert_eq!(
            result,
            ExpressionNode::literal(FhirPathValue::String("hello".to_string()))
        );
    }

    #[test]
    fn test_parse_identifier() {
        let result = parse_expression("name").unwrap();
        assert_eq!(result, ExpressionNode::identifier("name"));
    }

    #[test]
    fn test_parse_function_call() {
        let result = parse_expression("count()").unwrap();
        assert_eq!(result, ExpressionNode::function_call("count", vec![]));
    }

    #[test]
    fn test_parse_property_access() {
        let result = parse_expression("patient.name").unwrap();
        assert_eq!(
            result,
            ExpressionNode::path(ExpressionNode::identifier("patient"), "name")
        );
    }

    #[test]
    fn test_parse_binary_operation() {
        let result = parse_expression("1 + 2").unwrap();
        assert_eq!(
            result,
            ExpressionNode::binary_op(
                BinaryOperator::Add,
                ExpressionNode::literal(FhirPathValue::Integer(1)),
                ExpressionNode::literal(FhirPathValue::Integer(2))
            )
        );
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = parse_expression("patient.name.first() + ' ' + patient.name.last()").unwrap();
        // Just verify it parses without error - structure testing would be complex
        match result {
            ExpressionNode::BinaryOp { .. } => {}
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_debug_method_call_with_equals() {
        // Test the pattern that's failing in official tests
        let result = parse_expression("'test' = 'test'");
        println!("Simple equals result: {:?}", result);

        let result = parse_expression("'t2'.toChars()");
        println!("Method call result: {:?}", result);

        let result = parse_expression("'t' | '2'");
        println!("Union result: {:?}", result);

        let result = parse_expression("'t2'.toChars() = 't' | '2'");
        println!("Complex expression result: {:?}", result);
    }

    #[test]
    fn test_parse_predicate_expressions() {
        // Test predicate expressions that start with operators
        let result = parse_expression("= true").unwrap();
        match result {
            ExpressionNode::BinaryOp { op: BinaryOperator::Equal, left, right } => {
                assert_eq!(*left, ExpressionNode::identifier("$this"));
                assert_eq!(*right, ExpressionNode::literal(FhirPathValue::Boolean(true)));
            }
            _ => panic!("Expected binary operation with = operator, got: {:?}", result),
        }

        let result = parse_expression("> 5").unwrap();
        match result {
            ExpressionNode::BinaryOp { op: BinaryOperator::GreaterThan, left, right } => {
                assert_eq!(*left, ExpressionNode::identifier("$this"));
                assert_eq!(*right, ExpressionNode::literal(FhirPathValue::Integer(5)));
            }
            _ => panic!("Expected binary operation with > operator, got: {:?}", result),
        }

        let result = parse_expression("!= false").unwrap();
        match result {
            ExpressionNode::BinaryOp { op: BinaryOperator::NotEqual, left, right } => {
                assert_eq!(*left, ExpressionNode::identifier("$this"));
                assert_eq!(*right, ExpressionNode::literal(FhirPathValue::Boolean(false)));
            }
            _ => panic!("Expected binary operation with != operator, got: {:?}", result),
        }
    }

    #[test]
    fn debug_date_literal_parsing() {
        println!("Testing @ date literal parsing:");

        let test_cases = vec![
            "@2014-12-14",
            "@2015",
            "@2015-02",
            "@2014-12-14.toString()",
            "@2015.is(Date)",
        ];

        for case in test_cases {
            println!("\nTesting: {}", case);
            match parse_expression(case) {
                Ok(ast) => println!("  Success: {:#?}", ast),
                Err(e) => println!("  Error: {:?}", e),
            }
        }
    }

    #[test]
    fn debug_predicate_parsing() {
        println!("Testing predicate expression parsing:");

        let test_expressions = vec![
            "= true",
            "= false",
            "= 'Peter'",
            "",
            "true",
            "false",
        ];

        for expr in test_expressions {
            println!("\nTesting expression: '{}'", expr);
            match parse_expression(expr) {
                Ok(ast) => println!("  SUCCESS: {:?}", ast),
                Err(e) => println!("  ERROR: {:?}", e),
            }
        }
    }
}
