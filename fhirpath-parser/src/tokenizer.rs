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
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            Token::True | Token::False | Token::Empty |
            Token::Define | Token::Where | Token::Select |
            Token::All | Token::First | Token::Last |
            Token::Tail | Token::Skip | Token::Take |
            Token::Distinct | Token::Count | Token::OfType |
            Token::And | Token::Or | Token::Xor | Token::Implies |
            Token::Not | Token::In | Token::Contains |
            Token::Is | Token::As | Token::Div | Token::Mod
        )
    }
    
    /// Get keyword from string
    pub fn from_keyword(s: &str) -> Option<Token> {
        match s {
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            "empty" => Some(Token::Empty),
            "define" => Some(Token::Define),
            "where" => Some(Token::Where),
            "select" => Some(Token::Select),
            "all" => Some(Token::All),
            "first" => Some(Token::First),
            "last" => Some(Token::Last),
            "tail" => Some(Token::Tail),
            "skip" => Some(Token::Skip),
            "take" => Some(Token::Take),
            "distinct" => Some(Token::Distinct),
            "count" => Some(Token::Count),
            "ofType" => Some(Token::OfType),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "xor" => Some(Token::Xor),
            "implies" => Some(Token::Implies),
            "not" => Some(Token::Not),
            "in" => Some(Token::In),
            "contains" => Some(Token::Contains),
            "is" => Some(Token::Is),
            "as" => Some(Token::As),
            "div" => Some(Token::Div),
            "mod" => Some(Token::Mod),
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
        token_number,
        token_string,
        token_identifier_or_keyword,
        token_multi_char_op,
        token_single_char,
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
