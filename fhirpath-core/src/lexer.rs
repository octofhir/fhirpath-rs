// FHIRPath Lexer
//
// This module implements the lexical analysis for FHIRPath expressions.

use crate::errors::FhirPathError;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// Token types for FHIRPath expressions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Literals
    Identifier,
    DelimitedIdentifier,
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    DateLiteral,
    DateTimeLiteral,
    TimeLiteral,

    // Operators
    Dot,            // .
    Equal,          // =
    NotEqual,       // !=
    Equivalent,     // ~
    NotEquivalent,  // !~
    LessThan,       // <
    LessOrEqual,    // <=
    GreaterThan,    // >
    GreaterOrEqual, // >=
    Plus,           // +
    Minus,          // -
    Multiply,       // *
    Divide,         // /
    Div,            // div
    Ampersand,      // &

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Pipe,         // |
    Colon,        // :

    // Special characters
    Backtick,  // `
    Dollar,    // $
    At,        // @
    Backslash, // \
    Percent,   // %

    // Keywords
    And,      // and
    Or,       // or
    Xor,      // xor
    Implies,  // implies
    In,       // in
    Contains, // contains
    Mod,      // mod
    Is,       // is
    As,       // as

    // End of input
    EOF,
}

/// A token in a FHIRPath expression
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

/// Source span information for error reporting
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// Lexer for FHIRPath expressions
#[allow(dead_code)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    position: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("xor".to_string(), TokenType::Xor);
        keywords.insert("implies".to_string(), TokenType::Implies);
        keywords.insert("in".to_string(), TokenType::In);
        keywords.insert("contains".to_string(), TokenType::Contains);
        keywords.insert("mod".to_string(), TokenType::Mod);
        keywords.insert("div".to_string(), TokenType::Div);
        keywords.insert("is".to_string(), TokenType::Is);
        keywords.insert("as".to_string(), TokenType::As);
        keywords.insert("true".to_string(), TokenType::BooleanLiteral);
        keywords.insert("false".to_string(), TokenType::BooleanLiteral);

        // Date/time precision units (singular)
        keywords.insert("year".to_string(), TokenType::Identifier);
        keywords.insert("month".to_string(), TokenType::Identifier);
        keywords.insert("week".to_string(), TokenType::Identifier);
        keywords.insert("day".to_string(), TokenType::Identifier);
        keywords.insert("hour".to_string(), TokenType::Identifier);
        keywords.insert("minute".to_string(), TokenType::Identifier);
        keywords.insert("second".to_string(), TokenType::Identifier);
        keywords.insert("millisecond".to_string(), TokenType::Identifier);

        // Date/time precision units (plural)
        keywords.insert("years".to_string(), TokenType::Identifier);
        keywords.insert("months".to_string(), TokenType::Identifier);
        keywords.insert("weeks".to_string(), TokenType::Identifier);
        keywords.insert("days".to_string(), TokenType::Identifier);
        keywords.insert("hours".to_string(), TokenType::Identifier);
        keywords.insert("minutes".to_string(), TokenType::Identifier);
        keywords.insert("seconds".to_string(), TokenType::Identifier);
        keywords.insert("milliseconds".to_string(), TokenType::Identifier);

        Lexer {
            input,
            chars: input.chars().peekable(),
            position: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    /// Advances the lexer by one character
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.position += 1;
            self.column += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            }
        }
        c
    }

    /// Peeks at the next character without advancing
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Creates a token with the current position information
    fn make_token(&self, token_type: TokenType, lexeme: String) -> Token {
        let len = lexeme.len();
        Token {
            token_type,
            lexeme,
            position: self.position - len,
            line: self.line,
            column: self.column - len,
        }
    }

    /// Skips whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    /// Skips a block comment /* ... */
    fn skip_block_comment(&mut self) -> Result<(), FhirPathError> {
        let start_line = self.line;
        let start_column = self.column;

        while let Some(&c) = self.peek() {
            if c == '*' {
                self.advance();
                if let Some(&'/') = self.peek() {
                    self.advance(); // consume '/'
                    return Ok(());
                }
            } else {
                self.advance();
            }
        }

        // If we get here, the comment wasn't terminated
        Err(FhirPathError::LexerError(format!(
            "Unterminated block comment starting at line {}, column {}",
            start_line, start_column
        )))
    }

    /// Skips a line comment // ...
    fn skip_line_comment(&mut self) {
        while let Some(&c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.advance();
        }
    }

    /// Tokenizes an identifier or keyword
    fn identifier(&mut self) -> Result<Token, FhirPathError> {
        let start_pos = self.position;
        let start_column = self.column;
        let start_line = self.line;

        let mut identifier = String::new();

        // First character is already checked to be valid by the caller
        if let Some(c) = self.advance() {
            identifier.push(c);
        }

        // Continue reading valid identifier characters
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        let token_type = if let Some(keyword_type) = self.keywords.get(&identifier) {
            *keyword_type
        } else {
            TokenType::Identifier
        };

        Ok(Token {
            token_type,
            lexeme: identifier,
            position: start_pos,
            line: start_line,
            column: start_column,
        })
    }

    /// Tokenizes a number literal
    fn number(&mut self) -> Result<Token, FhirPathError> {
        let start_pos = self.position;
        let start_column = self.column;
        let start_line = self.line;

        let mut number = String::new();
        let mut has_decimal = false;

        // Continue reading digits
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                number.push(c);
                self.advance();
            } else if c == '.' && !has_decimal {
                // Check if there's a digit after the decimal point
                // Look ahead without consuming the dot
                let mut temp_pos = self.position + 1;
                if temp_pos < self.input.len() {
                    let next_char = self.input.chars().nth(temp_pos).unwrap();
                    if next_char.is_ascii_digit() {
                        // It's a decimal number, consume the dot and include it
                        self.advance(); // consume the dot
                        has_decimal = true;
                        number.push(c);
                        // Continue to read the digits after the decimal point
                    } else {
                        // It's not a decimal number (probably a method call like "1.round()")
                        // Don't consume the dot, let it be tokenized separately
                        break;
                    }
                } else {
                    // End of input after decimal point - not a valid decimal
                    return Err(FhirPathError::LexerError(format!(
                        "Expected digit after decimal point at line {}, column {}",
                        self.line,
                        self.column + 1
                    )));
                }
            } else {
                break;
            }
        }

        Ok(Token {
            token_type: TokenType::NumberLiteral,
            lexeme: number,
            position: start_pos,
            line: start_line,
            column: start_column,
        })
    }

    /// Tokenizes a string literal
    fn string(&mut self) -> Result<Token, FhirPathError> {
        let start_pos = self.position;
        let start_column = self.column;
        let start_line = self.line;

        // Skip the opening quote
        self.advance();

        let mut string = String::new();

        // Read until closing quote
        while let Some(&c) = self.peek() {
            if c == '\'' {
                // Skip the closing quote
                self.advance();

                // Check for escaped quote (double single quotes)
                if let Some(&next) = self.peek() {
                    if next == '\'' {
                        // It's an escaped quote, include it and continue
                        string.push('\'');
                        self.advance();
                        continue;
                    }
                }

                // It's the end of the string
                return Ok(Token {
                    token_type: TokenType::StringLiteral,
                    lexeme: string,
                    position: start_pos,
                    line: start_line,
                    column: start_column,
                });
            } else if c == '\\' {
                // Handle backslash escape sequences
                self.advance();
                if let Some(&escaped) = self.peek() {
                    match escaped {
                        '\'' => {
                            string.push('\'');
                            self.advance();
                        }
                        '"' => {
                            string.push('"');
                            self.advance();
                        }
                        '\\' => {
                            string.push('\\');
                            self.advance();
                        }
                        '/' => {
                            string.push('/');
                            self.advance();
                        }
                        'f' => {
                            string.push('\x0C'); // Form feed
                            self.advance();
                        }
                        'n' => {
                            string.push('\n');
                            self.advance();
                        }
                        'r' => {
                            string.push('\r');
                            self.advance();
                        }
                        't' => {
                            string.push('\t');
                            self.advance();
                        }
                        'u' => {
                            // Unicode escape sequence \uXXXX
                            self.advance();
                            let mut unicode_value = 0u32;
                            for _ in 0..4 {
                                if let Some(&hex_char) = self.peek() {
                                    if hex_char.is_ascii_hexdigit() {
                                        unicode_value =
                                            unicode_value * 16 + hex_char.to_digit(16).unwrap();
                                        self.advance();
                                    } else {
                                        return Err(FhirPathError::LexerError(format!(
                                            "Invalid unicode escape sequence at line {}, column {}",
                                            self.line, self.column
                                        )));
                                    }
                                } else {
                                    return Err(FhirPathError::LexerError(format!(
                                        "Incomplete unicode escape sequence at line {}, column {}",
                                        self.line, self.column
                                    )));
                                }
                            }
                            if let Some(unicode_char) = char::from_u32(unicode_value) {
                                string.push(unicode_char);
                            } else {
                                return Err(FhirPathError::LexerError(format!(
                                    "Invalid unicode value in escape sequence at line {}, column {}",
                                    self.line, self.column
                                )));
                            }
                        }
                        _ => {
                            return Err(FhirPathError::LexerError(format!(
                                "Invalid escape sequence '\\{}' at line {}, column {}",
                                escaped, self.line, self.column
                            )));
                        }
                    }
                } else {
                    return Err(FhirPathError::LexerError(format!(
                        "Incomplete escape sequence at line {}, column {}",
                        self.line, self.column
                    )));
                }
            } else if c == '\n' {
                return Err(FhirPathError::LexerError(format!(
                    "Unterminated string literal at line {}",
                    start_line
                )));
            } else {
                string.push(c);
                self.advance();
            }
        }

        // If we get here, the string wasn't terminated
        Err(FhirPathError::LexerError(format!(
            "Unterminated string literal at line {}, column {}",
            start_line, start_column
        )))
    }

    /// Scans a delimited identifier (backtick-enclosed)
    fn delimited_identifier(&mut self) -> Result<Token, FhirPathError> {
        let start_line = self.line;
        let start_column = self.column;

        // Consume opening backtick
        self.advance();

        let mut value = String::new();

        while let Some(&c) = self.peek() {
            if c == '`' {
                // Consume closing backtick
                self.advance();
                return Ok(self.make_token(TokenType::DelimitedIdentifier, value));
            } else if c == '\\' {
                // Handle escape sequences
                self.advance();
                if let Some(&escaped) = self.peek() {
                    match escaped {
                        '`' | '\\' | '/' | 'f' | 'n' | 'r' | 't' => {
                            value.push('\\');
                            value.push(escaped);
                            self.advance();
                        }
                        'u' => {
                            // Unicode escape sequence
                            value.push('\\');
                            value.push('u');
                            self.advance();
                            // Read 4 hex digits
                            for _ in 0..4 {
                                if let Some(&hex_char) = self.peek() {
                                    if hex_char.is_ascii_hexdigit() {
                                        value.push(hex_char);
                                        self.advance();
                                    } else {
                                        return Err(FhirPathError::LexerError(format!(
                                            "Invalid unicode escape sequence at line {}, column {}",
                                            self.line, self.column
                                        )));
                                    }
                                } else {
                                    return Err(FhirPathError::LexerError(format!(
                                        "Incomplete unicode escape sequence at line {}, column {}",
                                        self.line, self.column
                                    )));
                                }
                            }
                        }
                        _ => {
                            return Err(FhirPathError::LexerError(format!(
                                "Invalid escape sequence '\\{}' at line {}, column {}",
                                escaped, self.line, self.column
                            )));
                        }
                    }
                } else {
                    return Err(FhirPathError::LexerError(format!(
                        "Incomplete escape sequence at line {}, column {}",
                        self.line, self.column
                    )));
                }
            } else {
                value.push(c);
                self.advance();
            }
        }

        // If we get here, the delimited identifier wasn't terminated
        Err(FhirPathError::LexerError(format!(
            "Unterminated delimited identifier at line {}, column {}",
            start_line, start_column
        )))
    }

    /// Scans a date/time literal starting with @
    fn date_time_literal(&mut self) -> Result<Token, FhirPathError> {
        let start_line = self.line;
        let start_column = self.column;

        // Consume @
        self.advance();
        let mut value = String::from("@");

        // Check if this is a TIME literal (@T...)
        if let Some(&'T') = self.peek() {
            self.advance();
            value.push('T');

            // Parse time format: HH:MM:SS.fff
            if let Some(time_part) = self.scan_time_format() {
                value.push_str(&time_part);

                // Check if there's timezone information following - this is invalid for Time literals
                if self.scan_timezone_format().is_some() {
                    return Err(FhirPathError::LexerError(format!(
                        "Time literals cannot have timezone information at line {}, column {}",
                        start_line, start_column
                    )));
                }

                return Ok(self.make_token(TokenType::TimeLiteral, value));
            } else {
                return Err(FhirPathError::LexerError(format!(
                    "Invalid time format after @T at line {}, column {}",
                    start_line, start_column
                )));
            }
        }

        // Parse date format: YYYY-MM-DD
        if let Some(date_part) = self.scan_date_format() {
            value.push_str(&date_part);

            // Check if this continues as a datetime with T
            if let Some(&'T') = self.peek() {
                self.advance();
                value.push('T');

                // Parse optional time and timezone
                if let Some(time_part) = self.scan_time_format() {
                    value.push_str(&time_part);

                    // Parse optional timezone
                    if let Some(tz_part) = self.scan_timezone_format() {
                        value.push_str(&tz_part);
                    }
                }

                return Ok(self.make_token(TokenType::DateTimeLiteral, value));
            } else {
                return Ok(self.make_token(TokenType::DateLiteral, value));
            }
        }

        Err(FhirPathError::LexerError(format!(
            "Invalid date/time format after @ at line {}, column {}",
            start_line, start_column
        )))
    }

    /// Scans date format: YYYY-MM-DD
    fn scan_date_format(&mut self) -> Option<String> {
        let mut result = String::new();

        // YYYY
        for _ in 0..4 {
            if let Some(&c) = self.peek() {
                if c.is_ascii_digit() {
                    result.push(c);
                    self.advance();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        // Optional -MM-DD
        if let Some(&'-') = self.peek() {
            result.push('-');
            self.advance();

            // MM
            for _ in 0..2 {
                if let Some(&c) = self.peek() {
                    if c.is_ascii_digit() {
                        result.push(c);
                        self.advance();
                    } else {
                        return Some(result);
                    }
                } else {
                    return Some(result);
                }
            }

            // Optional -DD
            if let Some(&'-') = self.peek() {
                result.push('-');
                self.advance();

                // DD
                for _ in 0..2 {
                    if let Some(&c) = self.peek() {
                        if c.is_ascii_digit() {
                            result.push(c);
                            self.advance();
                        } else {
                            return Some(result);
                        }
                    } else {
                        return Some(result);
                    }
                }
            }
        }

        Some(result)
    }

    /// Scans time format: HH:MM:SS.fff
    fn scan_time_format(&mut self) -> Option<String> {
        let mut result = String::new();

        // HH
        for _ in 0..2 {
            if let Some(&c) = self.peek() {
                if c.is_ascii_digit() {
                    result.push(c);
                    self.advance();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        // Optional :MM:SS.fff
        if let Some(&':') = self.peek() {
            result.push(':');
            self.advance();

            // MM
            for _ in 0..2 {
                if let Some(&c) = self.peek() {
                    if c.is_ascii_digit() {
                        result.push(c);
                        self.advance();
                    } else {
                        return Some(result);
                    }
                } else {
                    return Some(result);
                }
            }

            // Optional :SS.fff
            if let Some(&':') = self.peek() {
                result.push(':');
                self.advance();

                // SS
                for _ in 0..2 {
                    if let Some(&c) = self.peek() {
                        if c.is_ascii_digit() {
                            result.push(c);
                            self.advance();
                        } else {
                            return Some(result);
                        }
                    } else {
                        return Some(result);
                    }
                }

                // Optional .fff (only if followed by digits)
                if let Some(&'.') = self.peek() {
                    // Look ahead to see if there are digits after the dot
                    let mut temp_pos = self.position + 1;
                    let mut has_digits_after_dot = false;

                    while temp_pos < self.input.len() {
                        if let Some(c) = self.input.chars().nth(temp_pos) {
                            if c.is_ascii_digit() {
                                has_digits_after_dot = true;
                                break;
                            } else if c.is_whitespace() {
                                temp_pos += 1;
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    // Only consume the dot if it's followed by digits (milliseconds)
                    if has_digits_after_dot {
                        result.push('.');
                        self.advance();

                        // One or more digits
                        let mut has_digits = false;
                        while let Some(&c) = self.peek() {
                            if c.is_ascii_digit() {
                                result.push(c);
                                self.advance();
                                has_digits = true;
                            } else {
                                break;
                            }
                        }

                        if !has_digits {
                            return Some(result);
                        }
                    }
                }
            }
        }

        Some(result)
    }

    /// Scans timezone format: Z or +HH:MM or -HH:MM
    fn scan_timezone_format(&mut self) -> Option<String> {
        if let Some(&c) = self.peek() {
            match c {
                'Z' => {
                    self.advance();
                    Some("Z".to_string())
                }
                '+' | '-' => {
                    let mut result = String::new();
                    result.push(c);
                    self.advance();

                    // HH
                    for _ in 0..2 {
                        if let Some(&digit) = self.peek() {
                            if digit.is_ascii_digit() {
                                result.push(digit);
                                self.advance();
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }

                    // :MM
                    if let Some(&':') = self.peek() {
                        result.push(':');
                        self.advance();

                        for _ in 0..2 {
                            if let Some(&digit) = self.peek() {
                                if digit.is_ascii_digit() {
                                    result.push(digit);
                                    self.advance();
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        }
                    }

                    Some(result)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Scans the next token
    pub fn scan_token(&mut self) -> Result<Token, FhirPathError> {
        self.skip_whitespace();

        if let Some(&c) = self.peek() {
            match c {
                // Single-character tokens
                '(' => {
                    self.advance();
                    Ok(self.make_token(TokenType::LeftParen, "(".to_string()))
                }
                ')' => {
                    self.advance();
                    Ok(self.make_token(TokenType::RightParen, ")".to_string()))
                }
                '[' => {
                    self.advance();
                    Ok(self.make_token(TokenType::LeftBracket, "[".to_string()))
                }
                ']' => {
                    self.advance();
                    Ok(self.make_token(TokenType::RightBracket, "]".to_string()))
                }
                '{' => {
                    self.advance();
                    Ok(self.make_token(TokenType::LeftBrace, "{".to_string()))
                }
                '}' => {
                    self.advance();
                    Ok(self.make_token(TokenType::RightBrace, "}".to_string()))
                }
                ',' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Comma, ",".to_string()))
                }
                '|' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Pipe, "|".to_string()))
                }
                ':' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Colon, ":".to_string()))
                }
                '.' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Dot, ".".to_string()))
                }
                '+' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Plus, "+".to_string()))
                }
                '-' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Minus, "-".to_string()))
                }
                '*' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Multiply, "*".to_string()))
                }
                '/' => {
                    self.advance();
                    // Check for comments
                    if let Some(&next) = self.peek() {
                        if next == '*' {
                            // Block comment /* ... */
                            self.advance(); // consume '*'
                            self.skip_block_comment()?;
                            return self.scan_token(); // Recursively scan next token
                        } else if next == '/' {
                            // Line comment // ...
                            self.advance(); // consume second '/'
                            self.skip_line_comment();
                            return self.scan_token(); // Recursively scan next token
                        }
                    }
                    Ok(self.make_token(TokenType::Divide, "/".to_string()))
                }

                // Special characters
                '`' => self.delimited_identifier(),
                '$' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Dollar, "$".to_string()))
                }
                '@' => self.date_time_literal(),
                '\\' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Backslash, "\\".to_string()))
                }
                '%' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Percent, "%".to_string()))
                }
                '&' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Ampersand, "&".to_string()))
                }

                // Two-character tokens
                '=' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Equal, "=".to_string()))
                }
                '!' => {
                    self.advance();
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            return Ok(self.make_token(TokenType::NotEqual, "!=".to_string()));
                        } else if next == '~' {
                            self.advance();
                            return Ok(self.make_token(TokenType::NotEquivalent, "!~".to_string()));
                        }
                    }
                    Err(FhirPathError::LexerError(format!(
                        "Unexpected character '!' at line {}, column {}",
                        self.line,
                        self.column - 1
                    )))
                }
                '<' => {
                    self.advance();
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            return Ok(self.make_token(TokenType::LessOrEqual, "<=".to_string()));
                        }
                    }
                    Ok(self.make_token(TokenType::LessThan, "<".to_string()))
                }
                '>' => {
                    self.advance();
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            return Ok(self.make_token(TokenType::GreaterOrEqual, ">=".to_string()));
                        }
                    }
                    Ok(self.make_token(TokenType::GreaterThan, ">".to_string()))
                }
                '~' => {
                    self.advance();
                    Ok(self.make_token(TokenType::Equivalent, "~".to_string()))
                }

                // String literals
                '\'' => self.string(),

                // Number literals
                '0'..='9' => self.number(),

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

                // Unexpected character
                _ => Err(FhirPathError::LexerError(format!(
                    "Unexpected character '{}' at line {}, column {}",
                    c, self.line, self.column
                ))),
            }
        } else {
            // End of input
            Ok(self.make_token(TokenType::EOF, "".to_string()))
        }
    }
}

/// Tokenizes a FHIRPath expression
pub fn tokenize(input: &str) -> Result<Vec<Token>, FhirPathError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.scan_token()?;
        let is_eof = token.token_type == TokenType::EOF;
        tokens.push(token);

        if is_eof {
            break;
        }
    }

    Ok(tokens)
}
