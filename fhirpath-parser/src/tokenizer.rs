//! Tokenizer for FHIRPath expressions

use crate::error::{ParseError, ParseResult};
use crate::span::{Span, Spanned};

/// Token types in FHIRPath
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Decimal(rust_decimal::Decimal),
    String(String),
    Boolean(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<chrono::Utc>),
    Time(chrono::NaiveTime),
    
    // Identifiers and keywords
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
    Div,
    Power,
    
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equivalent,
    NotEquivalent,
    
    // Logical
    And,
    Or,
    Xor,
    Implies,
    Not,
    
    // Collection
    Union,
    In,
    Contains,
    
    // Type operators
    Is,
    As,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    
    // Punctuation
    Dot,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    
    // Special
    Dollar,
    Backtick,
    
    // Keywords
    True,
    False,
    Empty,
    Define,
    Where,
    Select,
    All,
    First,
    Last,
    Tail,
    Skip,
    Take,
    Distinct,
    Count,
    OfType,
}

impl Token {
    /// Check if this token is a keyword (reserved word that cannot be used as identifier)
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            // Core literal keywords
            Token::True | Token::False |
            // Boolean operators
            Token::And | Token::Or | Token::Xor | Token::Implies |
            // Type operators  
            Token::Is | Token::As | Token::In | Token::Contains |
            // Arithmetic operators
            Token::Div | Token::Mod
        )
    }
    
    /// Get keyword from string - only true keywords, not function names
    pub fn from_keyword(s: &str) -> Option<Token> {
        match s {
            // Core literal keywords (always reserved)
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            
            // Boolean operators (can be used as operators only)
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "xor" => Some(Token::Xor),
            "implies" => Some(Token::Implies),
            
            // Type operators (always operators)
            "is" => Some(Token::Is),
            "as" => Some(Token::As),
            "in" => Some(Token::In),
            "contains" => Some(Token::Contains),
            
            // Arithmetic operators that are words
            "div" => Some(Token::Div),
            "mod" => Some(Token::Mod),
            
            // NOTE: "not" is intentionally removed from keywords since it can be:
            // 1. A function: Patient.active.not()
            // 2. An operator: not Patient.active  
            // The parser will need to disambiguate based on context
            
            // All function names are treated as identifiers:
            // "not", "empty", "define", "where", "select", "all", "first", "last", 
            // "tail", "skip", "take", "distinct", "count", "ofType"
            _ => None,
        }
    }
}

/// Tokenize a FHIRPath expression
pub fn tokenize(input: &str) -> ParseResult<Vec<Spanned<Token>>> {
    let span = Span::new(input);
    let (_, tokens) = tokenize_all(span).map_err(|e| match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => e,
        nom::Err::Incomplete(_) => ParseError::UnexpectedEof,
    })?;
    Ok(tokens)
}

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{all_consuming, map, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult, Parser,
};

fn tokenize_all(input: Span) -> IResult<Span, Vec<Spanned<Token>>, ParseError> {
    all_consuming(many0(preceded(multispace0, token))).parse(input)
}

fn token(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    alt((
        token_date_literal,
        token_number,
        token_string,
        token_backtick_identifier,
        token_identifier_or_keyword,
        token_multi_char_op,
        token_single_char,
    )).parse(input)
}

fn token_date_literal(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    
    // Must start with @
    let (input, _) = char('@').parse(input)?;
    
    // Try different date/time formats
    let (input, date_str) = alt((
        date_time_literal,
        time_literal,
        date_literal,
    )).parse(input)?;
    
    let date_text = date_str.fragment();
    
    let token = if date_text.starts_with('T') {
        // Time literal
        let time_part = &date_text[1..]; // Remove 'T' prefix
        match chrono::NaiveTime::parse_from_str(time_part, "%H:%M:%S") 
            .or_else(|_| chrono::NaiveTime::parse_from_str(time_part, "%H:%M:%S%.f")) {
            Ok(time) => Token::Time(time),
            Err(_) => return Err(nom::Err::Error(ParseError::InvalidLiteral {
                literal_type: "time".to_string(),
                value: date_text.to_string(),
                position: position(&start),
            })),
        }
    } else if date_text.contains('T') {
        // DateTime literal
        match chrono::DateTime::parse_from_rfc3339(&format!("{}Z", date_text.trim_end_matches('Z'))) {
            Ok(dt) => Token::DateTime(dt.with_timezone(&chrono::Utc)),
            Err(_) => return Err(nom::Err::Error(ParseError::InvalidLiteral {
                literal_type: "datetime".to_string(),
                value: date_text.to_string(),
                position: position(&start),
            })),
        }
    } else {
        // Date literal
        match chrono::NaiveDate::parse_from_str(date_text, "%Y-%m-%d") {
            Ok(date) => Token::Date(date),
            Err(_) => return Err(nom::Err::Error(ParseError::InvalidLiteral {
                literal_type: "date".to_string(),
                value: date_text.to_string(),
                position: position(&start),
            })),
        }
    };
    
    Ok((input, spanned(&start, &input, token)))
}

// Parse full DateTime: 2012-04-15T10:00:00 or 2012-04-15T10:00:00.123Z
fn date_time_literal(input: Span) -> IResult<Span, Span, ParseError> {
    recognize((
        take_while1(|c: char| c.is_ascii_digit()),  // year
        char('-'),
        take_while1(|c: char| c.is_ascii_digit()),  // month  
        char('-'),
        take_while1(|c: char| c.is_ascii_digit()),  // day
        char('T'),
        take_while1(|c: char| c.is_ascii_digit()),  // hour
        char(':'),
        take_while1(|c: char| c.is_ascii_digit()),  // minute
        char(':'),
        take_while1(|c: char| c.is_ascii_digit()),  // second
        opt((char('.'), take_while1(|c: char| c.is_ascii_digit()))), // optional milliseconds
        opt(char('Z')), // optional Z suffix
    )).parse(input)
}

// Parse time only: T10:00:00 or T10:00:00.123
fn time_literal(input: Span) -> IResult<Span, Span, ParseError> {
    recognize((
        char('T'),
        take_while1(|c: char| c.is_ascii_digit()),  // hour
        char(':'),
        take_while1(|c: char| c.is_ascii_digit()),  // minute
        char(':'),
        take_while1(|c: char| c.is_ascii_digit()),  // second
        opt((char('.'), take_while1(|c: char| c.is_ascii_digit()))), // optional milliseconds
    )).parse(input)
}

// Parse date only: 2012-04-15
fn date_literal(input: Span) -> IResult<Span, Span, ParseError> {
    recognize((
        take_while1(|c: char| c.is_ascii_digit()),  // year
        char('-'),
        take_while1(|c: char| c.is_ascii_digit()),  // month
        char('-'),
        take_while1(|c: char| c.is_ascii_digit()),  // day
    )).parse(input)
}

fn token_number(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    let (input, number_str) = recognize((
        opt(char('-')),
        take_while1(|c: char| c.is_ascii_digit()),
        opt((
            char('.'),
            take_while1(|c: char| c.is_ascii_digit())
        ))
    )).parse(input)?;
    
    let number_text = number_str.fragment();
    let token = if number_text.contains('.') {
        match number_text.parse::<rust_decimal::Decimal>() {
            Ok(d) => Token::Decimal(d),
            Err(_) => return Err(nom::Err::Error(ParseError::InvalidLiteral {
                literal_type: "decimal".to_string(),
                value: number_text.to_string(),
                position: position(&start),
            })),
        }
    } else {
        match number_text.parse::<i64>() {
            Ok(i) => Token::Integer(i),
            Err(_) => return Err(nom::Err::Error(ParseError::InvalidLiteral {
                literal_type: "integer".to_string(),
                value: number_text.to_string(),
                position: position(&start),
            })),
        }
    };
    
    Ok((input, spanned(&start, &input, token)))
}

fn token_string(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    let (input, _) = char('\'').parse(input)?;
    let mut chars = Vec::new();
    let mut remaining = input;
    
    loop {
        if let Ok((next, _)) = char::<_, ParseError>('\'').parse(remaining.clone()) {
            // Check for escaped quote
            if let Ok((next2, _)) = char::<_, ParseError>('\'').parse(next.clone()) {
                chars.push('\'');
                remaining = next2;
            } else {
                let token = Token::String(chars.into_iter().collect());
                return Ok((next, spanned(&start, &next, token)));
            }
        } else if let Ok((next, ch)) = nom::character::complete::anychar::<_, ParseError>(remaining) {
            chars.push(ch);
            remaining = next;
        } else {
            return Err(nom::Err::Error(ParseError::UnclosedString {
                position: position(&start),
            }));
        }
    }
}

fn token_identifier_or_keyword(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    
    // Regular identifier
    let (input, ident) = recognize((
        take_while1(|c: char| unicode_xid::UnicodeXID::is_xid_start(c) || c == '_'),
        opt(take_while1(|c: char| unicode_xid::UnicodeXID::is_xid_continue(c)))
    )).parse(input)?;
    
    let ident_str = ident.fragment();
    let token = Token::from_keyword(ident_str)
        .unwrap_or_else(|| Token::Identifier(ident_str.to_string()));
    
    Ok((input, spanned(&start, &input, token)))
}

fn token_backtick_identifier(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    let (input, _) = char('`').parse(input)?;
    
    // Parse identifier content between backticks
    let (input, ident) = take_while1(|c: char| {
        c != '`' && c != '\n' && c != '\r'
    }).parse(input)?;
    
    let (input, _) = char('`').parse(input).map_err(|_: nom::Err<ParseError>| {
        nom::Err::Error(ParseError::UnclosedString {
            position: position(&start),
        })
    })?;
    
    let ident_str = ident.fragment();
    let token = Token::Identifier(ident_str.to_string());
    
    Ok((input, spanned(&start, &input, token)))
}

fn token_multi_char_op(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    
    let (input, token) = alt((
        map(tag("<="), |_| Token::LessThanOrEqual),
        map(tag(">="), |_| Token::GreaterThanOrEqual),
        map(tag("!="), |_| Token::NotEqual),
        map(tag("!~"), |_| Token::NotEquivalent),
        map(tag("~"), |_| Token::Equivalent),
        map(tag("->"), |_| Token::Arrow),
    )).parse(input)?;
    
    Ok((input, spanned(&start, &input, token)))
}

fn token_single_char(input: Span) -> IResult<Span, Spanned<Token>, ParseError> {
    use crate::span::helpers::*;
    
    let start = input.clone();
    
    let (input, token) = alt((
        map(char('+'), |_| Token::Plus),
        map(char('-'), |_| Token::Minus),
        map(char('*'), |_| Token::Multiply),
        map(char('/'), |_| Token::Divide),
        map(char('^'), |_| Token::Power),
        map(char('='), |_| Token::Equal),
        map(char('<'), |_| Token::LessThan),
        map(char('>'), |_| Token::GreaterThan),
        map(char('|'), |_| Token::Union),
        map(char('('), |_| Token::LeftParen),
        map(char(')'), |_| Token::RightParen),
        map(char('['), |_| Token::LeftBracket),
        map(char(']'), |_| Token::RightBracket),
        map(char('{'), |_| Token::LeftBrace),
        map(char('}'), |_| Token::RightBrace),
        map(char('.'), |_| Token::Dot),
        map(char(','), |_| Token::Comma),
        map(char(':'), |_| Token::Colon),
        map(char(';'), |_| Token::Semicolon),
        map(char('$'), |_| Token::Dollar),
    )).parse(input)?;
    
    Ok((input, spanned(&start, &input, token)))
}
