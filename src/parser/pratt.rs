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
use crate::ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};

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
    /// Equality operators (=, !=, ~, !~)
    Equality = 5,
    /// Inequality operators (<, >, <=, >=)
    Inequality = 6,
    /// Union operator (|)
    Union = 7,
    /// Type operators (is, as)
    Type = 8,
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
            Precedence::Membership => Precedence::Equality,
            Precedence::Equality => Precedence::Inequality,
            Precedence::Inequality => Precedence::Union,
            Precedence::Union => Precedence::Type,
            Precedence::Type => Precedence::Additive,
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

/// Fast precedence lookup using direct pattern matching
///
/// This approach is actually faster than hash table lookup for small sets
/// and has better cache locality and branch prediction.
///
/// ## Adding New Operators
///
/// To add a new binary operator:
/// 1. Add the token variant to `Token` enum in `tokenizer.rs`
/// 2. Add the operator to the appropriate precedence level in this function
/// 3. Handle the operator in the main parsing loop if it needs special treatment
///
/// Example: Adding a new equality operator `===`:
/// ```text
/// Token::StrictEqual => Some(Precedence::Equality),
/// ```
///
/// The operators are ordered by frequency for branch prediction optimization.
#[inline(always)]
fn get_precedence<'input>(token: &Token<'input>) -> Option<Precedence> {
    match token {
        // Most common operators first for branch prediction optimization

        // Invocation operators (most frequent in FHIRPath)
        Token::Dot => Some(Precedence::Invocation),
        Token::LeftBracket => Some(Precedence::Invocation),

        // Equality operators (very common)
        Token::Equal => Some(Precedence::Equality),
        Token::NotEqual => Some(Precedence::Equality),
        Token::Equivalent => Some(Precedence::Equality),
        Token::NotEquivalent => Some(Precedence::Equality),

        // Additive operators (common)
        Token::Plus => Some(Precedence::Additive),
        Token::Minus => Some(Precedence::Additive),
        Token::Ampersand => Some(Precedence::Additive),

        // Logical operators
        Token::And => Some(Precedence::And),
        Token::Or => Some(Precedence::Or),
        Token::Xor => Some(Precedence::Or),

        // Multiplicative operators
        Token::Multiply => Some(Precedence::Multiplicative),
        Token::Divide => Some(Precedence::Multiplicative),
        Token::Div => Some(Precedence::Multiplicative),
        Token::Mod => Some(Precedence::Multiplicative),

        // Inequality operators
        Token::LessThan => Some(Precedence::Inequality),
        Token::LessThanOrEqual => Some(Precedence::Inequality),
        Token::GreaterThan => Some(Precedence::Inequality),
        Token::GreaterThanOrEqual => Some(Precedence::Inequality),

        // Membership operators
        Token::In => Some(Precedence::Membership),
        Token::Contains => Some(Precedence::Membership),

        // Type operators
        Token::Is => Some(Precedence::Type),
        Token::As => Some(Precedence::Type),

        // Union operator
        Token::Union => Some(Precedence::Union),

        // Implies (least common)
        Token::Implies => Some(Precedence::Implies),

        // Non-operator tokens
        _ => None,
    }
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
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(&expected) => {
                self.advance()
            }
            Some(token) => Err(ParseError::UnexpectedToken {
                token: format!(
                    "Expected {expected:?}, found {token:?}. Context: parsing expression"
                ),
                position: 0,
            }),
            None => Err(ParseError::UnexpectedToken {
                token: "Unexpected end of input while parsing expression".to_string(),
                position: 0,
            }),
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
            // Most common case: identifiers
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
                            token: "Empty parentheses are not valid in FHIRPath".to_string(),
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
                                    token: "Expected parameter name in lambda parameter list"
                                        .to_string(),
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
                                        ),
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
                        token: "Expected variable name after '$'".to_string(),
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

                        while let Some(token) = self.current() {
                            match token {
                                Token::Backtick => {
                                    // Found closing backtick
                                    self.advance()?; // consume closing backtick
                                    break;
                                }
                                Token::Identifier(name) => {
                                    var_name_parts.push(*name);
                                    self.advance()?;
                                }
                                Token::Minus => {
                                    var_name_parts.push("-");
                                    self.advance()?;
                                }
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Unexpected token in backtick variable name: {token:?}"
                                        ),
                                        position: 0,
                                    });
                                }
                            }
                        }

                        if var_name_parts.is_empty() {
                            return Err(ParseError::UnexpectedToken {
                                token: "Empty variable name in backticks".to_string(),
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
                        token: "Expected variable name after '%'".to_string(),
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
                    Some(Token::Identifier(name)) => *name,
                    Some(Token::Where) => "where",
                    Some(Token::Select) => "select",
                    Some(Token::All) => "all",
                    Some(Token::First) => "first",
                    Some(Token::Last) => "last",
                    Some(Token::Count) => "count",
                    Some(Token::Empty) => "empty",
                    Some(Token::Tail) => "tail",
                    Some(Token::True) => "true",
                    Some(Token::False) => "false",
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: "Expected identifier after backtick".to_string(),
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
                token: "Unexpected end of input".to_string(),
                position: 0,
            }),

            Some(token) => Err(ParseError::UnexpectedToken {
                token: format!("Unexpected token: {token:?}"),
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
                        token: "Expected ',' or ')' in function arguments".to_string(),
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
            Some(Token::Identifier(name)) => *name,
            Some(Token::Where) => "where",
            Some(Token::Select) => "select",
            Some(Token::All) => "all",
            Some(Token::First) => "first",
            Some(Token::Last) => "last",
            Some(Token::Count) => "count",
            Some(Token::Empty) => "empty",
            Some(Token::Tail) => "tail",
            Some(Token::Take) => "take",
            Some(Token::Skip) => "skip",
            Some(Token::Distinct) => "distinct",
            Some(Token::Is) => "is",
            Some(Token::Contains) => "contains",
            Some(Token::Not) => "not",
            Some(Token::OfType) => "ofType",
            Some(Token::As) => "as",
            Some(Token::Backtick) => {
                self.advance()?;
                let backtick_name = match self.current() {
                    Some(Token::Identifier(name)) => *name,
                    Some(Token::Where) => "where",
                    Some(Token::Select) => "select",
                    Some(Token::All) => "all",
                    Some(Token::First) => "first",
                    Some(Token::Last) => "last",
                    Some(Token::Count) => "count",
                    Some(Token::Empty) => "empty",
                    Some(Token::Tail) => "tail",
                    Some(Token::True) => "true",
                    Some(Token::False) => "false",
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: "Expected identifier after backtick".to_string(),
                            position: 0,
                        });
                    }
                };
                self.advance()?;
                self.expect(Token::Backtick)?;
                return self.parse_method_or_path(base, backtick_name);
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: format!("Expected identifier after dot: {:?}", self.current()),
                    position: 0,
                });
            }
        };

        self.advance()?;
        self.parse_method_or_path(base, name)
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
                            token: "Expected ',' or ')' in method arguments".to_string(),
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
                        if let Some(Token::Identifier(first_part)) = self.current() {
                            let mut type_name = first_part.to_string();
                            self.advance()?;

                            // Handle qualified names with dots (e.g., System.Boolean)
                            while let Some(Token::Dot) = self.current() {
                                self.advance()?; // consume dot
                                if let Some(Token::Identifier(next_part)) = self.current() {
                                    type_name.push('.');
                                    type_name.push_str(next_part);
                                    self.advance()?;
                                } else {
                                    return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Expected identifier after '.' in qualified type name, got: {:?}",
                                            self.current()
                                        ),
                                        position: 0,
                                    });
                                }
                            }

                            self.expect(Token::RightParen)?; // consume )
                            type_name
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected type name in parentheses after 'is' operator, got: {:?}",
                                    self.current()
                                ),
                                position: 0,
                            });
                        }
                    } else if let Some(Token::Identifier(first_part)) = self.current() {
                        // Handle is Type form
                        let mut type_name = first_part.to_string();
                        self.advance()?;

                        // Handle qualified names with dots (e.g., System.Boolean)
                        while let Some(Token::Dot) = self.current() {
                            self.advance()?; // consume dot
                            if let Some(Token::Identifier(next_part)) = self.current() {
                                type_name.push('.');
                                type_name.push_str(next_part);
                                self.advance()?;
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier after '.' in qualified type name, got: {:?}",
                                        self.current()
                                    ),
                                    position: 0,
                                });
                            }
                        }

                        type_name
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            token: format!(
                                "Expected type name after 'is' operator in type check expression, got: {:?}. Context: {}",
                                self.current(),
                                Self::precedence_context(precedence)
                            ),
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
                        if let Some(Token::Identifier(first_part)) = self.current() {
                            let mut type_name = first_part.to_string();
                            self.advance()?;

                            // Handle qualified names with dots (e.g., System.Boolean)
                            while let Some(Token::Dot) = self.current() {
                                self.advance()?; // consume dot
                                if let Some(Token::Identifier(next_part)) = self.current() {
                                    type_name.push('.');
                                    type_name.push_str(next_part);
                                    self.advance()?;
                                } else {
                                    return Err(ParseError::UnexpectedToken {
                                        token: format!(
                                            "Expected identifier after '.' in qualified type name, got: {:?}",
                                            self.current()
                                        ),
                                        position: 0,
                                    });
                                }
                            }

                            self.expect(Token::RightParen)?; // consume )
                            type_name
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                token: format!(
                                    "Expected type name in parentheses after 'as' operator, got: {:?}",
                                    self.current()
                                ),
                                position: 0,
                            });
                        }
                    } else if let Some(Token::Identifier(first_part)) = self.current() {
                        // Handle as Type form
                        let mut type_name = first_part.to_string();
                        self.advance()?;

                        // Handle qualified names with dots (e.g., System.Boolean)
                        while let Some(Token::Dot) = self.current() {
                            self.advance()?; // consume dot
                            if let Some(Token::Identifier(next_part)) = self.current() {
                                type_name.push('.');
                                type_name.push_str(next_part);
                                self.advance()?;
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: format!(
                                        "Expected identifier after '.' in qualified type name, got: {:?}",
                                        self.current()
                                    ),
                                    position: 0,
                                });
                            }
                        }

                        type_name
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            token: format!(
                                "Expected type name after 'as' operator in type cast expression, got: {:?}. Context: {}",
                                self.current(),
                                Self::precedence_context(precedence)
                            ),
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
                    token: format!("Expected binary operator, got {current_token:?}"),
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
                token: format!("Unexpected token: {:?}", self.current_token),
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
                                        sequence: "\\u".to_string(),
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
                                        sequence: format!("\\u{hex_chars}"),
                                        position: 0,
                                    });
                                }
                            },
                            Err(_) => {
                                return Err(ParseError::InvalidEscape {
                                    sequence: format!("\\u{hex_chars}"),
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
                            sequence: "\\".to_string(),
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
        if let ExpressionNode::BinaryOp {
            op: BinaryOperator::Add,
            left,
            right,
        } = result
        {
            assert!(matches!(
                *left,
                ExpressionNode::Literal(LiteralValue::Integer(2))
            ));
            assert!(matches!(
                *right,
                ExpressionNode::BinaryOp {
                    op: BinaryOperator::Multiply,
                    ..
                }
            ));
        } else {
            panic!("Expected addition with multiplication on right");
        }
    }

    #[test]
    fn test_associativity() {
        let result = parse_expression_pratt("a implies b implies c").unwrap();
        // Should parse as: a implies (b implies c) due to right associativity
        if let ExpressionNode::BinaryOp {
            op: BinaryOperator::Implies,
            left,
            right,
        } = result
        {
            assert!(matches!(*left, ExpressionNode::Identifier(_)));
            assert!(matches!(
                *right,
                ExpressionNode::BinaryOp {
                    op: BinaryOperator::Implies,
                    ..
                }
            ));
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
