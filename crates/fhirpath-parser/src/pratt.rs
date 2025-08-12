// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! High-performance Pratt parser for FHIRPath expressions
//!
//! This implementation focuses on maximum performance through:
//! - Zero-cost abstractions with const generics
//! - Compile-time optimized precedence tables
//! - Branch prediction friendly code
//! - Minimal allocations during parsing
//! - Cache-efficient memory layout

use super::error::{ParseError, ParseResult};
use super::tokenizer::{Token, Tokenizer};
use fhirpath_ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};

/// Operator precedence levels (higher = tighter binding)
/// Designed for optimal branch prediction with sequential spacing
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    /// Lowest precedence - implies (right associative)
    Implies = 1,
    /// Logical OR and XOR
    Or = 2,
    /// Logical AND
    And = 3,
    /// Membership operators (in, contains)
    Membership = 4,
    /// Type operators (is, as) - lower precedence than comparisons
    Type = 5,
    /// Equality operators (=, !=, ~, !~)
    Equality = 6,
    /// Inequality operators (<, >, <=, >=)
    Inequality = 7,
    /// Union operator (|)
    Union = 8,
    /// Additive operators (+, -, &)
    Additive = 9,
    /// Multiplicative operators (*, /, div, mod)
    Multiplicative = 10,
    /// Unary operators (+, -)
    Unary = 11,
    /// Invocation/Indexing (., [])
    Invocation = 12,
}

impl Precedence {
    /// Convert precedence to raw u8 for fast comparison
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the next higher precedence level for left-associative operators
    #[inline(always)]
    pub const fn next_level(self) -> Self {
        match self {
            Precedence::Implies => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Membership,
            Precedence::Membership => Precedence::Type,
            Precedence::Type => Precedence::Equality,
            Precedence::Equality => Precedence::Inequality,
            Precedence::Inequality => Precedence::Union,
            Precedence::Union => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Unary,
            Precedence::Unary => Precedence::Invocation,
            Precedence::Invocation => Precedence::Invocation, // Already highest
        }
    }

    /// Check if this precedence is right associative
    #[inline(always)]
    pub const fn is_right_associative(self) -> bool {
        matches!(self, Precedence::Implies)
    }
}

/// Token kind for efficient precedence lookup - zero-overhead abstraction
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    // Invocation operators (most frequent)
    Dot = 0,
    LeftBracket = 1,

    // Equality operators (very common)
    Equal = 2,
    NotEqual = 3,
    Equivalent = 4,
    NotEquivalent = 5,

    // Additive operators (common)
    Plus = 6,
    Minus = 7,
    Ampersand = 8,

    // Logical operators
    And = 9,
    Or = 10,
    Xor = 11,

    // Multiplicative operators
    Multiply = 12,
    Divide = 13,
    Div = 14,
    Mod = 15,

    // Inequality operators
    LessThan = 16,
    LessThanOrEqual = 17,
    GreaterThan = 18,
    GreaterThanOrEqual = 19,

    // Membership operators
    In = 20,
    Contains = 21,

    // Type operators
    Is = 22,
    As = 23,

    // Union operator
    Union = 24,

    // Implies (least common)
    Implies = 25,
}

/// Const precedence lookup table - O(1) array access with branch-free lookup
/// Ordered by frequency for optimal branch prediction and cache performance
const PRECEDENCE_TABLE: &[Precedence; 26] = &[
    // Most common operators first (indices 0-1)
    Precedence::Invocation, // 0: Dot
    Precedence::Invocation, // 1: LeftBracket
    // Equality operators (indices 2-5)
    Precedence::Equality, // 2: Equal
    Precedence::Equality, // 3: NotEqual
    Precedence::Equality, // 4: Equivalent
    Precedence::Equality, // 5: NotEquivalent
    // Additive operators (indices 6-8)
    Precedence::Additive, // 6: Plus
    Precedence::Additive, // 7: Minus
    Precedence::Additive, // 8: Ampersand
    // Logical operators (indices 9-11)
    Precedence::And, // 9: And
    Precedence::Or,  // 10: Or
    Precedence::Or,  // 11: Xor
    // Multiplicative operators (indices 12-15)
    Precedence::Multiplicative, // 12: Multiply
    Precedence::Multiplicative, // 13: Divide
    Precedence::Multiplicative, // 14: Div
    Precedence::Multiplicative, // 15: Mod
    // Inequality operators (indices 16-19)
    Precedence::Inequality, // 16: LessThan
    Precedence::Inequality, // 17: LessThanOrEqual
    Precedence::Inequality, // 18: GreaterThan
    Precedence::Inequality, // 19: GreaterThanOrEqual
    // Membership operators (indices 20-21)
    Precedence::Membership, // 20: In
    Precedence::Membership, // 21: Contains
    // Type operators (indices 22-23)
    Precedence::Type, // 22: Is
    Precedence::Type, // 23: As
    // Union operator (index 24)
    Precedence::Union, // 24: Union
    // Implies (index 25, least common)
    Precedence::Implies, // 25: Implies
];

/// Convert token to token kind for table lookup - branch-free when possible
#[inline(always)]
fn token_to_kind<'input>(token: &Token<'input>) -> Option<TokenKind> {
    match token {
        // Most common operators first for branch prediction optimization
        Token::Dot => Some(TokenKind::Dot),
        Token::LeftBracket => Some(TokenKind::LeftBracket),
        Token::Equal => Some(TokenKind::Equal),
        Token::NotEqual => Some(TokenKind::NotEqual),
        Token::Equivalent => Some(TokenKind::Equivalent),
        Token::NotEquivalent => Some(TokenKind::NotEquivalent),
        Token::Plus => Some(TokenKind::Plus),
        Token::Minus => Some(TokenKind::Minus),
        Token::Ampersand => Some(TokenKind::Ampersand),
        Token::And => Some(TokenKind::And),
        Token::Or => Some(TokenKind::Or),
        Token::Xor => Some(TokenKind::Xor),
        Token::Multiply => Some(TokenKind::Multiply),
        Token::Divide => Some(TokenKind::Divide),
        Token::Div => Some(TokenKind::Div),
        Token::Mod => Some(TokenKind::Mod),
        Token::LessThan => Some(TokenKind::LessThan),
        Token::LessThanOrEqual => Some(TokenKind::LessThanOrEqual),
        Token::GreaterThan => Some(TokenKind::GreaterThan),
        Token::GreaterThanOrEqual => Some(TokenKind::GreaterThanOrEqual),
        Token::In => Some(TokenKind::In),
        Token::Contains => Some(TokenKind::Contains),
        Token::Is => Some(TokenKind::Is),
        Token::As => Some(TokenKind::As),
        Token::Union => Some(TokenKind::Union),
        Token::Implies => Some(TokenKind::Implies),
        _ => None,
    }
}

/// High-performance precedence lookup using const lookup table
///
/// This implementation provides O(1) precedence lookup with:
/// - Compile-time const lookup table for zero runtime cost
/// - Branch prediction optimization through frequency-ordered matching
/// - Cache-friendly memory layout with sequential array access
/// - Zero bounds checking overhead due to known table size
///
/// ## Performance Benefits:
/// - **O(1) lookup**: Direct array access after token kind conversion
/// - **Cache efficient**: Single cache line access for entire precedence table
/// - **Branch predictable**: Most common operators handled first
/// - **Zero allocations**: Entirely stack-based with const data
///
/// ## Adding New Operators:
/// 1. Add token variant to `TokenKind` enum with appropriate index
/// 2. Add corresponding precedence to `PRECEDENCE_TABLE` at same index
/// 3. Add token-to-kind mapping in `token_to_kind` function
/// 4. Update table size in const array declaration
#[inline(always)]
fn get_precedence<'input>(token: &Token<'input>) -> Option<Precedence> {
    // Convert token to kind and perform O(1) table lookup
    token_to_kind(token).map(|kind| {
        // Safe: TokenKind values are guaranteed to be valid table indices
        unsafe { *PRECEDENCE_TABLE.get_unchecked(kind as usize) }
    })
}

/// Convert token to binary operator with zero-cost abstraction
/// Optimized ordering based on operator frequency in typical FHIRPath expressions
#[inline(always)]
fn token_to_binary_op<'input>(token: &Token<'input>) -> Option<BinaryOperator> {
    match token {
        // Most common operators first for better branch prediction
        Token::Equal => Some(BinaryOperator::Equal),
        Token::NotEqual => Some(BinaryOperator::NotEqual),
        Token::Plus => Some(BinaryOperator::Add),
        Token::Minus => Some(BinaryOperator::Subtract),
        Token::And => Some(BinaryOperator::And),
        Token::Or => Some(BinaryOperator::Or),

        // Moderately common operators
        Token::Equivalent => Some(BinaryOperator::Equivalent),
        Token::NotEquivalent => Some(BinaryOperator::NotEquivalent),
        Token::LessThan => Some(BinaryOperator::LessThan),
        Token::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
        Token::GreaterThan => Some(BinaryOperator::GreaterThan),
        Token::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
        Token::In => Some(BinaryOperator::In),
        Token::Contains => Some(BinaryOperator::Contains),

        // Less common operators
        Token::Multiply => Some(BinaryOperator::Multiply),
        Token::Divide => Some(BinaryOperator::Divide),
        Token::Div => Some(BinaryOperator::IntegerDivide),
        Token::Mod => Some(BinaryOperator::Modulo),
        Token::Union => Some(BinaryOperator::Union),
        Token::Ampersand => Some(BinaryOperator::Concatenate),
        Token::Xor => Some(BinaryOperator::Xor),
        Token::Implies => Some(BinaryOperator::Implies),

        _ => None,
    }
}

/// High-performance Pratt parser with zero-allocation design
///
/// ## Pratt Parser Architecture
///
/// This implementation uses the Pratt parsing algorithm (also known as precedence climbing
/// or top-down operator precedence parsing) to efficiently parse FHIRPath expressions.
///
/// ### Key Benefits over Recursive Descent:
/// - **Data-driven precedence**: All operator precedence is defined in a single table
/// - **Better performance**: Direct precedence lookups avoid deep call stacks
/// - **Easier maintenance**: Adding new operators only requires updating the precedence table
/// - **Cleaner code**: Single parsing algorithm handles all binary operators
///
/// ### Algorithm Overview:
/// 1. Parse left-hand side expression (primary + postfix operations)
/// 2. While next token is a binary operator with sufficient precedence:
///    - Parse right-hand side with appropriate precedence climbing
///    - Combine left and right operands with the operator
/// 3. Return the final expression tree
///
/// ### Precedence Levels (highest to lowest):
/// - **Invocation** (12): `.`, `[]` - method calls and indexing
/// - **Unary** (11): `+`, `-` - unary plus/minus
/// - **Multiplicative** (10): `*`, `/`, `div`, `mod` - multiplication, division
/// - **Additive** (9): `+`, `-`, `&` - addition, subtraction, concatenation
/// - **Type** (8): `is`, `as` - type checking and casting
/// - **Union** (7): `|` - union operations
/// - **Inequality** (6): `<`, `>`, `<=`, `>=` - comparison operators
/// - **Equality** (5): `=`, `!=`, `~`, `!~` - equality and equivalence
/// - **Membership** (4): `in`, `contains` - membership tests
/// - **And** (3): `and` - logical conjunction
/// - **Or** (2): `or`, `xor` - logical disjunction
/// - **Implies** (1): `implies` - logical implication (right-associative)
///
/// ### Performance Optimizations:
/// - Compile-time precedence tables with `#[repr(u8)]` enum
/// - Branch-prediction friendly token dispatch (hot paths first)
/// - Aggressive inlining with `#[inline(always)]` on hot functions
/// - Zero-allocation parsing with lifetime parameters
/// - Direct pattern matching for O(1) precedence lookups
pub struct PrattParser<'input> {
    tokenizer: Tokenizer<'input>,
    current_token: Option<Token<'input>>,
}

impl<'input> PrattParser<'input> {
    /// Create new parser with minimal overhead
    #[inline]
    pub fn new(input: &'input str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
            current_token: None,
        }
    }

    /// Advance to next token with branch-free error handling
    /// Optimized for hot path with minimal allocations
    #[inline(always)]
    fn advance(&mut self) -> ParseResult<()> {
        self.current_token = self.tokenizer.next_token()?;
        Ok(())
    }

    /// Peek at current token with zero-cost abstraction
    #[inline(always)]
    fn current(&self) -> Option<&Token<'input>> {
        self.current_token.as_ref()
    }

    /// Check if current token matches expected with branch prediction hint
    #[inline(always)]
    fn expect(&mut self, expected: Token<'input>) -> ParseResult<()> {
        match &self.current_token {
            Some(token) if Self::tokens_match(token, &expected) => self.advance(),
            Some(token) => Err(ParseError::UnexpectedToken {
                token: format!(
                    "Expected {expected:?}, found {token:?}. Context: parsing expression"
                )
                .into(),
                position: 0,
            }),
            None => Err(ParseError::UnexpectedToken {
                token: std::borrow::Cow::Borrowed(
                    "Unexpected end of input while parsing expression",
                ),
                position: 0,
            }),
        }
    }

    /// Fast token type matching using direct pattern matching
    #[inline(always)]
    fn tokens_match(token: &Token<'input>, expected: &Token<'input>) -> bool {
        match (token, expected) {
            // Literals
            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Decimal(_), Token::Decimal(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::Date(_), Token::Date(_)) => true,
            (Token::DateTime(_), Token::DateTime(_)) => true,
            (Token::Time(_), Token::Time(_)) => true,
            (Token::Quantity { .. }, Token::Quantity { .. }) => true,

            // Identifiers (regular and interned)
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (Token::Identifier(_), Token::InternedIdentifier(_)) => true,
            (Token::InternedIdentifier(_), Token::Identifier(_)) => true,
            (Token::InternedIdentifier(_), Token::InternedIdentifier(_)) => true,

            // Unit tokens (exact matches)
            (Token::Plus, Token::Plus) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Multiply, Token::Multiply) => true,
            (Token::Divide, Token::Divide) => true,
            (Token::Mod, Token::Mod) => true,
            (Token::Div, Token::Div) => true,
            (Token::Power, Token::Power) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::NotEqual, Token::NotEqual) => true,
            (Token::LessThan, Token::LessThan) => true,
            (Token::LessThanOrEqual, Token::LessThanOrEqual) => true,
            (Token::GreaterThan, Token::GreaterThan) => true,
            (Token::GreaterThanOrEqual, Token::GreaterThanOrEqual) => true,
            (Token::Equivalent, Token::Equivalent) => true,
            (Token::NotEquivalent, Token::NotEquivalent) => true,
            (Token::And, Token::And) => true,
            (Token::Or, Token::Or) => true,
            (Token::Xor, Token::Xor) => true,
            (Token::Implies, Token::Implies) => true,
            (Token::Not, Token::Not) => true,
            (Token::Union, Token::Union) => true,
            (Token::In, Token::In) => true,
            (Token::Contains, Token::Contains) => true,
            (Token::Ampersand, Token::Ampersand) => true,
            (Token::Is, Token::Is) => true,
            (Token::As, Token::As) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::RightParen, Token::RightParen) => true,
            (Token::LeftBracket, Token::LeftBracket) => true,
            (Token::RightBracket, Token::RightBracket) => true,
            (Token::LeftBrace, Token::LeftBrace) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::Semicolon, Token::Semicolon) => true,
            (Token::Arrow, Token::Arrow) => true,
            (Token::Dollar, Token::Dollar) => true,
            (Token::Percent, Token::Percent) => true,
            (Token::Backtick, Token::Backtick) => true,
            (Token::DollarThis, Token::DollarThis) => true,
            (Token::DollarIndex, Token::DollarIndex) => true,
            (Token::DollarTotal, Token::DollarTotal) => true,
            (Token::True, Token::True) => true,
            (Token::False, Token::False) => true,
            (Token::Empty, Token::Empty) => true,
            (Token::Define, Token::Define) => true,
            (Token::Where, Token::Where) => true,
            (Token::Select, Token::Select) => true,
            (Token::All, Token::All) => true,
            (Token::First, Token::First) => true,
            (Token::Last, Token::Last) => true,
            (Token::Tail, Token::Tail) => true,
            (Token::Skip, Token::Skip) => true,
            (Token::Take, Token::Take) => true,
            (Token::Distinct, Token::Distinct) => true,
            (Token::Count, Token::Count) => true,
            (Token::OfType, Token::OfType) => true,

            // No match
            _ => false,
        }
    }

    /// Get precedence information for error messages
    fn precedence_context(precedence: Precedence) -> &'static str {
        match precedence {
            Precedence::Implies => "implies expression (lowest precedence)",
            Precedence::Or => "or/xor expression",
            Precedence::And => "and expression",
            Precedence::Membership => "membership expression (in/contains)",
            Precedence::Equality => "equality expression (=/!=/~/!~)",
            Precedence::Inequality => "comparison expression (</>/<=/>=/is/as)",
            Precedence::Union => "union expression (|)",
            Precedence::Type => "type expression (is/as)",
            Precedence::Additive => "additive expression (+/-/&)",
            Precedence::Multiplicative => "multiplicative expression (*/div/mod)",
            Precedence::Unary => "unary expression (+/-)",
            Precedence::Invocation => "invocation expression (./[])",
        }
    }

    /// Parse primary expression (literals, identifiers, parenthesized expressions)
    /// Optimized for the most common cases first
    #[inline]
    fn parse_primary(&mut self) -> ParseResult<ExpressionNode> {
        if self.current_token.is_none() {
            self.advance()?;
        }

        match self.current() {
            // Most common case: identifiers (regular and interned)
            Some(Token::Identifier(name)) => {
                let name = *name;
                self.advance()?;
                // Check for function call
                if let Some(Token::LeftParen) = self.current() {
                    self.parse_function_call(name)
                } else if let Some(Token::Arrow) = self.current() {
                    // Single parameter lambda: param => expression
                    self.advance()?; // consume =>
                    let body = self.parse_expression_with_precedence(Precedence::Implies)?;
                    Ok(ExpressionNode::lambda_single(name, body))
                } else {
                    Ok(ExpressionNode::identifier(name))
                }
            }

            // Interned identifiers
            Some(Token::InternedIdentifier(name_arc)) => {
                let name = name_arc.as_ref().to_string(); // Clone to avoid borrowing issues
                self.advance()?;
                // Check for function call
                if let Some(Token::LeftParen) = self.current() {
                    self.parse_function_call(&name)
                } else if let Some(Token::Arrow) = self.current() {
                    // Single parameter lambda: param => expression
                    self.advance()?; // consume =>
                    let body = self.parse_expression_with_precedence(Precedence::Implies)?;
                    Ok(ExpressionNode::lambda_single(&name, body))
                } else {
                    Ok(ExpressionNode::identifier(&name))
                }
            }

            // Integer literals - hot path optimization
            Some(Token::Integer(value)) => {
                let value = *value;
                self.advance()?;
                // Check for quantity (number followed by unit - string or identifier)
                match self.current() {
                    Some(Token::String(unit)) => {
                        let unit_str = *unit;
                        self.advance()?;
                        Ok(ExpressionNode::literal(LiteralValue::Quantity {
                            value: value.to_string(),
                            unit: unit_str.to_string(),
                        }))
                    }
                    Some(Token::Identifier(unit)) => {
                        let unit_str = *unit;
                        self.advance()?;
                        Ok(ExpressionNode::literal(LiteralValue::Quantity {
                            value: value.to_string(),
                            unit: unit_str.to_string(),
                        }))
                    }
                    _ => Ok(ExpressionNode::literal(LiteralValue::Integer(value))),
                }
            }

            // String literals
            Some(Token::String(value)) => {
                let value = *value;
                self.advance()?;

                // Process escape sequences including Unicode escapes
                let processed_string = Self::process_string_escapes(value)?;

                Ok(ExpressionNode::literal(LiteralValue::String(
                    processed_string,
                )))
            }

            // Decimal literals
            Some(Token::Decimal(value)) => {
                let value = *value;
                self.advance()?;
                // Check for quantity (decimal followed by unit - string or identifier)
                match self.current() {
                    Some(Token::String(unit)) => {
                        let unit_str = *unit;
                        self.advance()?;
                        Ok(ExpressionNode::literal(LiteralValue::Quantity {
                            value: value.to_string(),
                            unit: unit_str.to_string(),
                        }))
                    }
                    Some(Token::Identifier(unit)) => {
                        let unit_str = *unit;
                        self.advance()?;
                        Ok(ExpressionNode::literal(LiteralValue::Quantity {
                            value: value.to_string(),
                            unit: unit_str.to_string(),
                        }))
                    }
                    _ => Ok(ExpressionNode::literal(LiteralValue::Decimal(
                        value.to_string(),
                    ))),
                }
            }

            // Boolean literals
            Some(Token::True) => {
                self.advance()?;
                Ok(ExpressionNode::literal(LiteralValue::Boolean(true)))
            }
            Some(Token::False) => {
                self.advance()?;
                Ok(ExpressionNode::literal(LiteralValue::Boolean(false)))
            }

            // Date/time literals
            Some(Token::Date(value)) => {
                let value = *value;
                self.advance()?;
                Ok(ExpressionNode::literal(LiteralValue::Date(
                    value.to_string(),
                )))
            }
            Some(Token::DateTime(value)) => {
                let value = *value;
                self.advance()?;
                Ok(ExpressionNode::literal(LiteralValue::DateTime(
                    value.to_string(),
                )))
            }
            Some(Token::Time(value)) => {
                let value = *value;
                self.advance()?;
                Ok(ExpressionNode::literal(LiteralValue::Time(
                    value.to_string(),
                )))
            }

            // Parenthesized expressions or multi-parameter lambdas
            Some(Token::LeftParen) => {
                self.advance()?;

                // Check for empty parameter list: () => expression
                if let Some(Token::RightParen) = self.current() {
                    self.advance()?; // consume )
                    if let Some(Token::Arrow) = self.current() {
                        // Anonymous lambda: () => expression
                        self.advance()?; // consume =>
                        let body = self.parse_expression_with_precedence(Precedence::Implies)?;
                        return Ok(ExpressionNode::lambda_anonymous(body));
                    } else {
                        // This should not happen in valid FHIRPath - empty parentheses without =>
                        return Err(ParseError::UnexpectedToken {
                            token: std::borrow::Cow::Borrowed(
                                "Empty parentheses are not valid in FHIRPath",
                            ),
                            position: 0,
                        });
                    }
                }

                // Try to parse as parameter list or regular expression
                // First, check if it looks like parameters (identifier followed by comma or ))
                if let Some(Token::Identifier(first_param)) = self.current() {
                    let first_param = *first_param;
                    self.advance()?;

                    if let Some(Token::Comma) = self.current() {
                        // Multi-parameter lambda: (param1, param2, ...) => expression
                        let mut params = vec![first_param.to_string()];

                        while let Some(Token::Comma) = self.current() {
                            self.advance()?; // consume comma
                            if let Some(Token::Identifier(param)) = self.current() {
                                params.push(param.to_string());
                                self.advance()?;
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: std::borrow::Cow::Borrowed(
                                        "Expected parameter name in lambda parameter list",
                                    ),
                                    position: 0,
                                });
                            }
                        }

                        self.expect(Token::RightParen)?;
                        self.expect(Token::Arrow)?;
                        let body = self.parse_expression_with_precedence(Precedence::Implies)?;
                        Ok(ExpressionNode::lambda(params, body))
                    } else if let Some(Token::RightParen) = self.current() {
                        // Could be single parameter lambda: (param) => expression
                        self.advance()?; // consume )
                        if let Some(Token::Arrow) = self.current() {
                            self.advance()?; // consume =>
                            let body =
                                self.parse_expression_with_precedence(Precedence::Implies)?;
                            Ok(ExpressionNode::lambda_single(first_param, body))
                        } else {
                            // Regular parenthesized identifier
                            Ok(ExpressionNode::identifier(first_param))
                        }
                    } else {
                        // Regular parenthesized expression starting with identifier
                        // We have already consumed the identifier, so we need to manually
                        // continue the expression parsing from this point
                        let mut left = ExpressionNode::identifier(first_param);
                        left = self.parse_postfix(left)?;

                        // Process binary operators manually for this special case
                        if let Some(current_token) = self.current() {
                            if !matches!(current_token, Token::RightParen) {
                                // Handle binary operators within parentheses
                                if let Some(op) = token_to_binary_op(current_token) {
                                    self.advance()?; // consume operator
                                    let right =
                                        self.parse_expression_with_precedence(Precedence::Implies)?;
                                    left = ExpressionNode::binary_op(op, left, right);
                                } else {
                                    return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Unexpected token in parentheses: {current_token:?}"
                                        )
                                        .into(),
                                        position: 0,
                                    });
                                }
                            }
                        }

                        self.expect(Token::RightParen)?;
                        Ok(left)
                    }
                } else {
                    // Regular parenthesized expression
                    let expr = self.parse_expression_with_precedence(Precedence::Implies)?;
                    self.expect(Token::RightParen)?;
                    Ok(expr)
                }
            }

            // Variable references
            Some(Token::Dollar) => {
                self.advance()?;
                if let Some(Token::Identifier(name)) = self.current() {
                    let var_name = *name;
                    self.advance()?;
                    Ok(ExpressionNode::variable(var_name))
                } else {
                    Err(ParseError::UnexpectedToken {
                        token: std::borrow::Cow::Borrowed("Expected variable name after '$'"),
                        position: 0,
                    })
                }
            }

            // Special variable references
            Some(Token::DollarThis) => {
                self.advance()?;
                Ok(ExpressionNode::variable("this"))
            }

            Some(Token::DollarIndex) => {
                self.advance()?;
                Ok(ExpressionNode::variable("index"))
            }

            Some(Token::DollarTotal) => {
                self.advance()?;
                Ok(ExpressionNode::variable("total"))
            }

            // Context variables
            Some(Token::Percent) => {
                self.advance()?;
                match self.current() {
                    Some(Token::Identifier(name)) => {
                        let var_name = *name;
                        self.advance()?;
                        Ok(ExpressionNode::variable(var_name))
                    }
                    Some(Token::Backtick) => {
                        self.advance()?; // consume opening backtick

                        // Collect all tokens until closing backtick to form the variable name
                        let mut var_name_parts = Vec::new();

                        loop {
                            match self.current() {
                                Some(Token::Backtick) => {
                                    // Found closing backtick
                                    self.advance()?; // consume closing backtick
                                    break;
                                }
                                Some(Token::Identifier(name)) => {
                                    let name = (*name).to_string();
                                    self.advance()?;
                                    var_name_parts.push(name);
                                }
                                Some(Token::InternedIdentifier(name)) => {
                                    let name = name.as_ref().to_string();
                                    self.advance()?;
                                    var_name_parts.push(name);
                                }
                                Some(Token::Minus) => {
                                    self.advance()?;
                                    var_name_parts.push("-".to_string());
                                }
                                Some(token) => {
                                    return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Unexpected token in backtick variable name: {token:?}"
                                        )
                                        .into(),
                                        position: 0,
                                    });
                                }
                                None => {
                                    return Err(ParseError::UnexpectedToken {
                                        token: "Unexpected end of input in backtick variable name"
                                            .into(),
                                        position: 0,
                                    });
                                }
                            }
                        }

                        if var_name_parts.is_empty() {
                            return Err(ParseError::UnexpectedToken {
                                token: std::borrow::Cow::Borrowed(
                                    "Empty variable name in backticks",
                                ),
                                position: 0,
                            });
                        }

                        // For now, we'll use a heap-allocated string for complex variable names
                        // This is a limitation of the current AST design
                        let var_name = var_name_parts.join("");
                        Ok(ExpressionNode::variable(Box::leak(
                            var_name.into_boxed_str(),
                        )))
                    }
                    _ => Err(ParseError::UnexpectedToken {
                        token: std::borrow::Cow::Borrowed("Expected variable name after '%'"),
                        position: 0,
                    }),
                }
            }

            // Unary operators
            Some(Token::Minus) => {
                self.advance()?;
                let operand = self.parse_expression_with_precedence(Precedence::Unary)?;
                Ok(ExpressionNode::unary_op(UnaryOperator::Minus, operand))
            }
            Some(Token::Plus) => {
                self.advance()?;
                // Unary plus is essentially a no-op, just parse the operand
                self.parse_expression_with_precedence(Precedence::Unary)
            }

            // Empty collections
            Some(Token::LeftBrace) => {
                self.advance()?;
                self.expect(Token::RightBrace)?;
                Ok(ExpressionNode::literal(LiteralValue::Null))
            }

            // Backtick identifiers
            Some(Token::Backtick) => {
                self.advance()?;
                let name = match self.current() {
                    Some(Token::Identifier(name)) => (*name).to_string(),
                    Some(Token::InternedIdentifier(name)) => name.as_ref().to_string(),
                    Some(Token::Where) => "where".to_string(),
                    Some(Token::Select) => "select".to_string(),
                    Some(Token::All) => "all".to_string(),
                    Some(Token::First) => "first".to_string(),
                    Some(Token::Last) => "last".to_string(),
                    Some(Token::Count) => "count".to_string(),
                    Some(Token::Empty) => "empty".to_string(),
                    Some(Token::Contains) => "contains".to_string(),
                    Some(Token::Tail) => "tail".to_string(),
                    Some(Token::True) => "true".to_string(),
                    Some(Token::False) => "false".to_string(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: std::borrow::Cow::Borrowed("Expected identifier after backtick"),
                            position: 0,
                        });
                    }
                };
                self.advance()?;
                self.expect(Token::Backtick)?;
                Ok(ExpressionNode::identifier(name))
            }

            // Built-in function keywords that can be standalone
            Some(Token::Count) => self.parse_builtin_function("count"),
            Some(Token::Where) => self.parse_builtin_function("where"),
            Some(Token::Select) => self.parse_builtin_function("select"),
            Some(Token::All) => self.parse_builtin_function("all"),
            Some(Token::First) => self.parse_builtin_function("first"),
            Some(Token::Last) => self.parse_builtin_function("last"),
            Some(Token::Tail) => self.parse_builtin_function("tail"),
            Some(Token::Empty) => self.parse_builtin_function("empty"),
            Some(Token::Contains) => self.parse_builtin_function("contains"),
            Some(Token::Take) => self.parse_builtin_function("take"),
            Some(Token::Skip) => self.parse_builtin_function("skip"),
            Some(Token::Distinct) => self.parse_builtin_function("distinct"),

            // Anonymous lambda: => expression
            Some(Token::Arrow) => {
                self.advance()?; // consume =>
                let body = self.parse_expression_with_precedence(Precedence::Implies)?;
                Ok(ExpressionNode::lambda_anonymous(body))
            }

            None => Err(ParseError::UnexpectedToken {
                token: std::borrow::Cow::Borrowed("Unexpected end of input"),
                position: 0,
            }),

            Some(token) => Err(ParseError::UnexpectedToken {
                token: format!("Unexpected token: {token:?}").into(),
                position: 0,
            }),
        }
    }

    /// Parse built-in function calls (count, where, select, etc.)
    #[inline]
    fn parse_builtin_function(&mut self, function_name: &str) -> ParseResult<ExpressionNode> {
        self.advance()?;

        // Check if it's a function call
        if let Some(Token::LeftParen) = self.current() {
            self.parse_function_call(function_name)
        } else {
            Ok(ExpressionNode::identifier(function_name))
        }
    }

    /// Parse function call with optimized argument parsing
    #[inline]
    fn parse_function_call(&mut self, name: &str) -> ParseResult<ExpressionNode> {
        self.expect(Token::LeftParen)?;

        let mut args = Vec::new();

        // Handle empty argument list
        if let Some(Token::RightParen) = self.current() {
            self.advance()?;
            return Ok(ExpressionNode::function_call(name, args));
        }

        // Parse argument list
        loop {
            args.push(self.parse_expression_with_precedence(Precedence::Implies)?);

            match self.current() {
                Some(Token::Comma) => {
                    self.advance()?;
                    continue;
                }
                Some(Token::RightParen) => {
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        token: std::borrow::Cow::Borrowed(
                            "Expected ',' or ')' in function arguments",
                        ),
                        position: 0,
                    });
                }
            }
        }

        Ok(ExpressionNode::function_call(name, args))
    }

    /// Parse postfix expressions (method calls, indexing, path navigation)
    /// This handles the highest precedence operations efficiently
    #[inline]
    fn parse_postfix(&mut self, mut left: ExpressionNode) -> ParseResult<ExpressionNode> {
        loop {
            match self.current() {
                Some(Token::Dot) => {
                    self.advance()?;
                    left = self.parse_path_or_method(left)?;
                }
                Some(Token::LeftBracket) => {
                    self.advance()?;
                    let index = self.parse_expression_with_precedence(Precedence::Implies)?;
                    self.expect(Token::RightBracket)?;
                    left = ExpressionNode::index(left, index);
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Parse path navigation or method call after dot
    #[inline]
    fn parse_path_or_method(&mut self, base: ExpressionNode) -> ParseResult<ExpressionNode> {
        let name = match self.current() {
            Some(Token::Identifier(name)) => (*name).to_string(),
            Some(Token::InternedIdentifier(arc_name)) => arc_name.as_ref().to_string(),
            Some(Token::Where) => "where".to_string(),
            Some(Token::Select) => "select".to_string(),
            Some(Token::All) => "all".to_string(),
            Some(Token::First) => "first".to_string(),
            Some(Token::Last) => "last".to_string(),
            Some(Token::Count) => "count".to_string(),
            Some(Token::Empty) => "empty".to_string(),
            Some(Token::Contains) => "contains".to_string(),
            Some(Token::Tail) => "tail".to_string(),
            Some(Token::Take) => "take".to_string(),
            Some(Token::Skip) => "skip".to_string(),
            Some(Token::Distinct) => "distinct".to_string(),
            Some(Token::Is) => "is".to_string(),
            Some(Token::Not) => "not".to_string(),
            Some(Token::OfType) => "ofType".to_string(),
            Some(Token::As) => "as".to_string(),
            Some(Token::Backtick) => {
                self.advance()?;
                let backtick_name = match self.current() {
                    Some(Token::Identifier(name)) => (*name).to_string(),
                    Some(Token::InternedIdentifier(arc_name)) => arc_name.as_ref().to_string(),
                    Some(Token::Where) => "where".to_string(),
                    Some(Token::Select) => "select".to_string(),
                    Some(Token::All) => "all".to_string(),
                    Some(Token::First) => "first".to_string(),
                    Some(Token::Last) => "last".to_string(),
                    Some(Token::Count) => "count".to_string(),
                    Some(Token::Empty) => "empty".to_string(),
                    Some(Token::Contains) => "contains".to_string(),
                    Some(Token::Tail) => "tail".to_string(),
                    Some(Token::True) => "true".to_string(),
                    Some(Token::False) => "false".to_string(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: std::borrow::Cow::Borrowed("Expected identifier after backtick"),
                            position: 0,
                        });
                    }
                };
                self.advance()?;
                self.expect(Token::Backtick)?;
                return self.parse_method_or_path(base, &backtick_name);
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: format!("Expected identifier after dot: {:?}", self.current()).into(),
                    position: 0,
                });
            }
        };

        self.advance()?;
        self.parse_method_or_path(base, &name)
    }

    /// Parse method call or path based on whether parentheses follow
    #[inline]
    fn parse_method_or_path(
        &mut self,
        base: ExpressionNode,
        name: &str,
    ) -> ParseResult<ExpressionNode> {
        if let Some(Token::LeftParen) = self.current() {
            // Method call
            self.advance()?;
            let mut args = Vec::new();

            if let Some(Token::RightParen) = self.current() {
                self.advance()?;
                return Ok(ExpressionNode::method_call(base, name, args));
            }

            loop {
                args.push(self.parse_expression_with_precedence(Precedence::Implies)?);

                match self.current() {
                    Some(Token::Comma) => {
                        self.advance()?;
                        continue;
                    }
                    Some(Token::RightParen) => {
                        self.advance()?;
                        break;
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: std::borrow::Cow::Borrowed(
                                "Expected ',' or ')' in method arguments",
                            ),
                            position: 0,
                        });
                    }
                }
            }

            Ok(ExpressionNode::method_call(base, name, args))
        } else {
            // Path navigation
            Ok(ExpressionNode::path(base, name))
        }
    }

    /// Core Pratt parsing algorithm with branch-prediction optimizations
    /// This is the hot path - optimized for maximum performance with aggressive inlining
    #[inline(always)]
    fn parse_expression_with_precedence(
        &mut self,
        min_precedence: Precedence,
    ) -> ParseResult<ExpressionNode> {
        // Parse left-hand side (primary + postfix)
        let mut left = self.parse_primary()?;
        left = self.parse_postfix(left)?;

        // Process binary operators with precedence climbing
        while let Some(current_token) = self.current() {
            // Fast precedence lookup with branch prediction hint
            let precedence = match get_precedence(current_token) {
                Some(prec) if prec as u8 >= min_precedence as u8 => prec,
                _ => break,
            };

            // Handle special cases efficiently
            match current_token {
                Token::Is => {
                    self.advance()?;

                    // Handle parenthesized type names like is(DateTime) or plain type names like is DateTime
                    let type_name = if let Some(Token::LeftParen) = self.current() {
                        // Handle is(Type) form
                        self.advance()?; // consume (
                        if let Some(token) = self.current() {
                            if let Some(first_part) = token.as_identifier() {
                                let mut type_name = first_part.to_string();
                                self.advance()?;

                                // Handle qualified names with dots (e.g., System.Boolean)
                                while let Some(Token::Dot) = self.current() {
                                    self.advance()?; // consume dot
                                    if let Some(token) = self.current() {
                                        if let Some(next_part) = token.as_identifier() {
                                            type_name.push('.');
                                            type_name.push_str(next_part);
                                            self.advance()?;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                self.expect(Token::RightParen)?; // consume )
                                type_name
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier in 'is(Type)', got: {:?}",
                                        self.current()
                                    )
                                    .into(),
                                    position: 0,
                                });
                            }
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected type name in parentheses after 'is' operator, got: {:?}",
                                    self.current()
                                ).into(),
                                position: 0,
                            });
                        }
                    } else if let Some(token) = self.current() {
                        if let Some(first_part) = token.as_identifier() {
                            // Handle is Type form
                            let mut type_name = first_part.to_string();
                            self.advance()?;

                            // Handle qualified names with dots (e.g., System.Boolean)
                            while let Some(Token::Dot) = self.current() {
                                self.advance()?; // consume dot
                                if let Some(token) = self.current() {
                                    if let Some(next_part) = token.as_identifier() {
                                        type_name.push('.');
                                        type_name.push_str(next_part);
                                        self.advance()?;
                                    } else {
                                        return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Expected identifier after '.' in qualified type name, got: {:?}",
                                            self.current()
                                        ).into(),
                                        position: 0,
                                    });
                                    }
                                } else {
                                    return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier after '.' in qualified type name, got: {:?}",
                                        self.current()
                                    ).into(),
                                    position: 0,
                                });
                                }
                            }

                            type_name
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected identifier after 'is', got: {:?}",
                                    self.current()
                                )
                                .into(),
                                position: 0,
                            });
                        }
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            token: format!(
                                "Expected type name after 'is' operator in type check expression, got: {:?}. Context: {}",
                                self.current(),
                                Self::precedence_context(precedence)
                            ).into(),
                            position: 0,
                        });
                    };

                    left = ExpressionNode::TypeCheck {
                        expression: Box::new(left),
                        type_name,
                    };
                    continue;
                }
                Token::As => {
                    self.advance()?;

                    // Handle parenthesized type names like as(Type) or plain type names like as Type
                    let type_name = if let Some(Token::LeftParen) = self.current() {
                        // Handle as(Type) form
                        self.advance()?; // consume (
                        if let Some(token) = self.current() {
                            if let Some(first_part) = token.as_identifier() {
                                let mut type_name = first_part.to_string();
                                self.advance()?;

                                // Handle qualified names with dots (e.g., System.Boolean)
                                while let Some(Token::Dot) = self.current() {
                                    self.advance()?; // consume dot
                                    if let Some(token) = self.current() {
                                        if let Some(next_part) = token.as_identifier() {
                                            type_name.push('.');
                                            type_name.push_str(next_part);
                                            self.advance()?;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                self.expect(Token::RightParen)?; // consume )
                                type_name
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier in 'as(Type)', got: {:?}",
                                        self.current()
                                    )
                                    .into(),
                                    position: 0,
                                });
                            }
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected type name in parentheses after 'as' operator, got: {:?}",
                                    self.current()
                                ).into(),
                                position: 0,
                            });
                        }
                    } else if let Some(token) = self.current() {
                        if let Some(first_part) = token.as_identifier() {
                            // Handle as Type form
                            let mut type_name = first_part.to_string();
                            self.advance()?;

                            // Handle qualified names with dots (e.g., System.Boolean)
                            while let Some(Token::Dot) = self.current() {
                                self.advance()?; // consume dot
                                if let Some(token) = self.current() {
                                    if let Some(next_part) = token.as_identifier() {
                                        type_name.push('.');
                                        type_name.push_str(next_part);
                                        self.advance()?;
                                    } else {
                                        return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Expected identifier after '.' in qualified type name, got: {:?}",
                                            self.current()
                                        ).into(),
                                        position: 0,
                                    });
                                    }
                                } else {
                                    return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier after '.' in qualified type name, got: {:?}",
                                        self.current()
                                    ).into(),
                                    position: 0,
                                });
                                }
                            }

                            type_name
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected identifier after 'as', got: {:?}",
                                    self.current()
                                )
                                .into(),
                                position: 0,
                            });
                        }
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            token: format!(
                                "Expected type name after 'as' operator in type cast expression, got: {:?}. Context: {}",
                                self.current(),
                                Self::precedence_context(precedence)
                            ).into(),
                            position: 0,
                        });
                    };

                    left = ExpressionNode::TypeCast {
                        expression: Box::new(left),
                        type_name,
                    };
                    continue;
                }
                _ => {}
            }

            // Get binary operator
            let op =
                token_to_binary_op(current_token).ok_or_else(|| ParseError::UnexpectedToken {
                    token: format!("Expected binary operator, got {current_token:?}").into(),
                    position: 0,
                })?;

            self.advance()?;

            // Calculate next minimum precedence (handles associativity)
            // For left associative operators, use next higher precedence level
            let next_min_precedence = if precedence.is_right_associative() {
                precedence
            } else {
                precedence.next_level()
            };

            // Parse right-hand side recursively
            let right = self.parse_expression_with_precedence(next_min_precedence)?;

            // Create binary operation node
            left = ExpressionNode::binary_op(op, left, right);
        }

        Ok(left)
    }

    /// Parse complete expression (public entry point)
    #[inline]
    pub fn parse_expression(&mut self) -> ParseResult<ExpressionNode> {
        self.parse_expression_with_precedence(Precedence::Implies)
    }

    /// Parse complete input
    #[inline]
    pub fn parse(&mut self) -> ParseResult<ExpressionNode> {
        let expr = self.parse_expression()?;

        // Ensure we consumed all input
        if self.current_token.is_some() {
            return Err(ParseError::UnexpectedToken {
                token: format!("Unexpected token: {:?}", self.current_token).into(),
                position: 0,
            });
        }

        Ok(expr)
    }

    /// Process escape sequences in string literals, including Unicode escapes
    fn process_string_escapes(input: &str) -> ParseResult<String> {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('\'') => result.push('\''),
                    Some('\"') => result.push('\"'),
                    Some('u') => {
                        // Unicode escape sequence \uXXXX
                        let mut hex_chars = String::new();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(hex_ch) if hex_ch.is_ascii_hexdigit() => {
                                    hex_chars.push(hex_ch);
                                }
                                _ => {
                                    return Err(ParseError::InvalidEscape {
                                        sequence: std::borrow::Cow::Borrowed("\\u"),
                                        position: 0,
                                    });
                                }
                            }
                        }

                        // Parse hex digits to Unicode code point
                        match u32::from_str_radix(&hex_chars, 16) {
                            Ok(code_point) => match char::from_u32(code_point) {
                                Some(unicode_char) => result.push(unicode_char),
                                None => {
                                    return Err(ParseError::InvalidEscape {
                                        sequence: format!("\\u{hex_chars}").into(),
                                        position: 0,
                                    });
                                }
                            },
                            Err(_) => {
                                return Err(ParseError::InvalidEscape {
                                    sequence: format!("\\u{hex_chars}").into(),
                                    position: 0,
                                });
                            }
                        }
                    }
                    Some(escaped_ch) => {
                        // Unknown escape sequence - treat literally for compatibility
                        result.push('\\');
                        result.push(escaped_ch);
                    }
                    None => {
                        return Err(ParseError::InvalidEscape {
                            sequence: std::borrow::Cow::Borrowed("\\"),
                            position: 0,
                        });
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

/// High-performance parsing function (public API)
#[inline]
pub fn parse_expression_pratt(input: &str) -> ParseResult<ExpressionNode> {
    let mut parser = PrattParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        assert!(Precedence::Multiplicative > Precedence::Additive);
        assert!(Precedence::Additive > Precedence::Equality);
        assert!(Precedence::Equality > Precedence::And);
        assert!(Precedence::And > Precedence::Or);
        assert!(Precedence::Or > Precedence::Implies);
    }

    #[test]
    fn test_basic_expressions() {
        let result = parse_expression_pratt("Patient").unwrap();
        assert!(matches!(result, ExpressionNode::Identifier(_)));

        let result = parse_expression_pratt("Patient.name").unwrap();
        assert!(matches!(result, ExpressionNode::Path { .. }));

        let result = parse_expression_pratt("2 + 3 * 4").unwrap();
        // Should parse as: 2 + (3 * 4) due to precedence
        if let ExpressionNode::BinaryOp(data) = result {
            assert_eq!(data.op, BinaryOperator::Add);
            assert!(matches!(
                data.left,
                ExpressionNode::Literal(LiteralValue::Integer(2))
            ));
            if let ExpressionNode::BinaryOp(inner_data) = &data.right {
                assert_eq!(inner_data.op, BinaryOperator::Multiply);
            } else {
                panic!("Expected multiplication on right side");
            }
        } else {
            panic!("Expected addition with multiplication on right");
        }
    }

    #[test]
    fn test_associativity() {
        let result = parse_expression_pratt("a implies b implies c").unwrap();
        // Should parse as: a implies (b implies c) due to right associativity
        if let ExpressionNode::BinaryOp(data) = result {
            assert_eq!(data.op, BinaryOperator::Implies);
            assert!(matches!(data.left, ExpressionNode::Identifier(_)));
            if let ExpressionNode::BinaryOp(inner_data) = &data.right {
                assert_eq!(inner_data.op, BinaryOperator::Implies);
            } else {
                panic!("Expected nested implies on right side");
            }
        } else {
            panic!("Expected right-associative implies");
        }
    }

    #[test]
    fn test_function_calls() {
        let result = parse_expression_pratt("count()").unwrap();
        assert!(matches!(result, ExpressionNode::FunctionCall { .. }));

        let result = parse_expression_pratt("Patient.name.where(use = 'official')").unwrap();
        assert!(matches!(result, ExpressionNode::MethodCall { .. }));
    }

    #[test]
    fn test_complex_expression() {
        let result =
            parse_expression_pratt("Patient.name.where(use = 'official').given.first()").unwrap();
        // Should parse as a chain of method calls and path navigations
        assert!(matches!(result, ExpressionNode::MethodCall { .. }));
    }
}
