// FHIRPath Parser
//
// This module implements the parser for FHIRPath expressions.

use crate::errors::FhirPathError;
use crate::lexer::{Token, TokenType};

/// AST node types for FHIRPath expressions
#[derive(Debug, Clone)]
pub enum AstNode {
    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral { value: f64, is_decimal: bool },
    BooleanLiteral(bool),
    DateTimeLiteral(String),
    QuantityLiteral {
        value: f64,
        unit: Option<String>,
    },
    Variable(String),

    // Path navigation
    Path(Box<AstNode>, Box<AstNode>),

    // Function calls
    FunctionCall {
        name: String,
        arguments: Vec<AstNode>,
    },

    // Operators
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    UnaryOp {
        op: UnaryOperator,
        operand: Box<AstNode>,
    },

    // Indexing
    Indexer {
        collection: Box<AstNode>,
        index: Box<AstNode>,
    },
}

/// Binary operators in FHIRPath
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    Equivalent,
    NotEquivalent,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Implies,
    In,
    Contains,
    Is,
    As,
    Union,
    Concatenation,
}

/// Unary operators in FHIRPath
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Positive,
    Negate,
    Not,
}

/// Parser for FHIRPath expressions
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parses a FHIRPath expression
    pub fn parse(&mut self) -> Result<AstNode, FhirPathError> {
        self.expression()
    }

    /// Checks if we've reached the end of the token stream
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token_type == TokenType::EOF
    }

    /// Returns the current token without advancing
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Returns the previous token
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Advances to the next token and returns the current one
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Checks if the current token matches the given type
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    /// Consumes the current token if it matches the given type
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consumes the current token if it matches any of the given types
    fn match_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.match_token(*token_type) {
                return true;
            }
        }
        false
    }

    /// Consumes the current token if it matches the given type, otherwise throws an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, FhirPathError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(FhirPathError::ParserError(format!(
                "{} at token {:?}",
                message,
                self.peek()
            )))
        }
    }

    /// Parses an expression
    fn expression(&mut self) -> Result<AstNode, FhirPathError> {
        self.logical_implies()
    }

    /// Parses a logical IMPLIES expression
    fn logical_implies(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.logical_or()?;

        while self.match_token(TokenType::Implies) {
            let right = self.logical_or()?;
            expr = AstNode::BinaryOp {
                op: BinaryOperator::Implies,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a logical OR expression
    fn logical_or(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.logical_and()?;

        while self.match_any(&[TokenType::Or, TokenType::Xor]) {
            let operator = match self.previous().token_type {
                TokenType::Or => BinaryOperator::Or,
                TokenType::Xor => BinaryOperator::Xor,
                _ => unreachable!(),
            };
            let right = self.logical_and()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a logical AND expression
    fn logical_and(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.membership()?;

        while self.match_token(TokenType::And) {
            let right = self.membership()?;
            expr = AstNode::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a membership expression (in, contains)
    fn membership(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.equality()?;

        while self.match_any(&[TokenType::In, TokenType::Contains]) {
            let operator = match self.previous().token_type {
                TokenType::In => BinaryOperator::In,
                TokenType::Contains => BinaryOperator::Contains,
                _ => unreachable!(),
            };
            let right = self.equality()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses an equality expression
    fn equality(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.inequality()?;

        while self.match_any(&[
            TokenType::Equal,
            TokenType::NotEqual,
            TokenType::Equivalent,
            TokenType::NotEquivalent,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Equal => BinaryOperator::Equals,
                TokenType::NotEqual => BinaryOperator::NotEquals,
                TokenType::Equivalent => BinaryOperator::Equivalent,
                TokenType::NotEquivalent => BinaryOperator::NotEquivalent,
                _ => unreachable!(),
            };
            let right = self.inequality()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses an inequality expression
    fn inequality(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.union()?;

        while self.match_any(&[
            TokenType::LessThan,
            TokenType::LessOrEqual,
            TokenType::GreaterThan,
            TokenType::GreaterOrEqual,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::LessThan => BinaryOperator::LessThan,
                TokenType::LessOrEqual => BinaryOperator::LessOrEqual,
                TokenType::GreaterThan => BinaryOperator::GreaterThan,
                TokenType::GreaterOrEqual => BinaryOperator::GreaterOrEqual,
                _ => unreachable!(),
            };
            let right = self.union()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a union expression
    fn union(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.type_expression()?;

        while self.match_token(TokenType::Pipe) {
            let right = self.type_expression()?;
            expr = AstNode::BinaryOp {
                op: BinaryOperator::Union,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a type expression (is, as)
    fn type_expression(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.additive()?;

        while self.check(TokenType::Is) || self.check(TokenType::As) {
            // Look ahead to see if this is a method call (followed by '(') or a binary operator
            if self.check(TokenType::Is)
                && self.current + 1 < self.tokens.len()
                && self.tokens[self.current + 1].token_type == TokenType::LeftParen
            {
                // This is a method call like .is(DateTime), not a binary operator
                // Let path() handle it instead
                break;
            }

            if self.check(TokenType::As)
                && self.current + 1 < self.tokens.len()
                && self.tokens[self.current + 1].token_type == TokenType::LeftParen
            {
                // This is a method call like .as(Type), not a binary operator
                // Let path() handle it instead
                break;
            }

            // This is a binary operator, consume it
            self.advance();
            let operator = match self.previous().token_type {
                TokenType::Is => BinaryOperator::Is,
                TokenType::As => BinaryOperator::As,
                _ => unreachable!(),
            };
            let right = self.qualified_identifier()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a qualified identifier (identifier ('.' identifier)*)
    fn qualified_identifier(&mut self) -> Result<AstNode, FhirPathError> {
        if !self.check(TokenType::Identifier)
            && !self.check(TokenType::DelimitedIdentifier)
            && !self.match_any(&[
                TokenType::Is,
                TokenType::As,
                TokenType::Contains,
                TokenType::In,
            ])
        {
            return Err(FhirPathError::ParserError(
                "Expected identifier for qualified identifier".to_string(),
            ));
        }

        let mut qualified_name = String::new();

        // Handle first identifier (can be regular identifier, delimited identifier, or keyword)
        if self.match_token(TokenType::Identifier) {
            qualified_name.push_str(&self.previous().lexeme);
        } else if self.match_token(TokenType::DelimitedIdentifier) {
            qualified_name.push_str(&self.previous().lexeme);
        } else if self.match_any(&[
            TokenType::Is,
            TokenType::As,
            TokenType::Contains,
            TokenType::In,
        ]) {
            qualified_name.push_str(&self.previous().lexeme);
        }

        // Handle additional dot-separated identifiers
        while self.match_token(TokenType::Dot) {
            qualified_name.push('.');

            if self.match_token(TokenType::Identifier) {
                qualified_name.push_str(&self.previous().lexeme);
            } else if self.match_token(TokenType::DelimitedIdentifier) {
                qualified_name.push_str(&self.previous().lexeme);
            } else if self.match_any(&[
                TokenType::Is,
                TokenType::As,
                TokenType::Contains,
                TokenType::In,
            ]) {
                qualified_name.push_str(&self.previous().lexeme);
            } else {
                return Err(FhirPathError::ParserError(
                    "Expected identifier after '.' in qualified identifier".to_string(),
                ));
            }
        }

        Ok(AstNode::Identifier(qualified_name))
    }

    /// Parses an additive expression (addition, subtraction, concatenation)
    fn additive(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.multiplicative()?;

        while self.match_any(&[TokenType::Plus, TokenType::Minus, TokenType::Ampersand]) {
            let operator = match self.previous().token_type {
                TokenType::Plus => BinaryOperator::Addition,
                TokenType::Minus => BinaryOperator::Subtraction,
                TokenType::Ampersand => BinaryOperator::Concatenation,
                _ => unreachable!(),
            };
            let right = self.multiplicative()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a multiplicative expression (multiplication, division, div, mod)
    fn multiplicative(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.unary()?;

        while self.match_any(&[
            TokenType::Multiply,
            TokenType::Divide,
            TokenType::Div,
            TokenType::Mod,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Multiply => BinaryOperator::Multiplication,
                TokenType::Divide => BinaryOperator::Division,
                TokenType::Div => BinaryOperator::Div,
                TokenType::Mod => BinaryOperator::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = AstNode::BinaryOp {
                op: operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parses a unary expression
    fn unary(&mut self) -> Result<AstNode, FhirPathError> {
        if self.match_token(TokenType::Plus) {
            let right = self.unary()?;
            Ok(AstNode::UnaryOp {
                op: UnaryOperator::Positive,
                operand: Box::new(right),
            })
        } else if self.match_token(TokenType::Minus) {
            let right = self.unary()?;
            Ok(AstNode::UnaryOp {
                op: UnaryOperator::Negate,
                operand: Box::new(right),
            })
        } else if self.check(TokenType::Identifier) && self.peek().lexeme == "not" {
            self.advance(); // consume 'not'
            let right = self.unary()?;
            Ok(AstNode::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(right),
            })
        } else {
            self.path()
        }
    }

    /// Parses a path expression
    fn path(&mut self) -> Result<AstNode, FhirPathError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(TokenType::Dot) {
                // Path navigation
                let right = self.primary()?;
                expr = AstNode::Path(Box::new(expr), Box::new(right));
            } else if self.match_token(TokenType::LeftBracket) {
                // Indexer
                let index = self.expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after index")?;
                expr = AstNode::Indexer {
                    collection: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parses a primary expression
    fn primary(&mut self) -> Result<AstNode, FhirPathError> {
        if self.match_token(TokenType::Identifier) {
            let name = self.previous().lexeme.clone();

            // Check if this is a function call
            if self.match_token(TokenType::LeftParen) {
                let mut arguments = Vec::new();

                // Parse arguments
                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    TokenType::RightParen,
                    "Expected ')' after function arguments",
                )?;

                Ok(AstNode::FunctionCall { name, arguments })
            } else {
                Ok(AstNode::Identifier(name))
            }
        } else if self.match_any(&[
            TokenType::Is,
            TokenType::As,
            TokenType::Contains,
            TokenType::In,
        ]) {
            // Handle 'is', 'as', 'contains', 'in' as function names when they appear in function call contexts
            let name = self.previous().lexeme.clone();

            // Check if this is a function call
            if self.match_token(TokenType::LeftParen) {
                let mut arguments = Vec::new();

                // Parse arguments
                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    TokenType::RightParen,
                    "Expected ')' after function arguments",
                )?;

                Ok(AstNode::FunctionCall { name, arguments })
            } else {
                Ok(AstNode::Identifier(name))
            }
        } else if self.match_token(TokenType::DelimitedIdentifier) {
            // Handle delimited identifiers like `identifier`
            let name = self.previous().lexeme.clone();
            Ok(AstNode::Identifier(name))
        } else if self.match_token(TokenType::StringLiteral) {
            Ok(AstNode::StringLiteral(self.previous().lexeme.clone()))
        } else if self.match_token(TokenType::NumberLiteral) {
            let lexeme = &self.previous().lexeme;
            let value = lexeme
                .parse::<f64>()
                .map_err(|e| FhirPathError::ParserError(format!("Invalid number: {}", e)))?;

            // Check if this is followed by a unit (quantity literal)
            if self.check(TokenType::Identifier) || self.check(TokenType::StringLiteral) {
                let unit = if self.match_token(TokenType::Identifier) {
                    Some(self.previous().lexeme.clone())
                } else if self.match_token(TokenType::StringLiteral) {
                    Some(self.previous().lexeme.clone())
                } else {
                    None
                };

                Ok(AstNode::QuantityLiteral { value, unit })
            } else {
                // Regular number literal without unit
                // Check if the original lexeme contained a decimal point
                let is_decimal = lexeme.contains('.');
                Ok(AstNode::NumberLiteral { value, is_decimal })
            }
        } else if self.match_token(TokenType::BooleanLiteral) {
            let value = match self.previous().lexeme.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(FhirPathError::ParserError(
                        "Invalid boolean literal".to_string(),
                    ));
                }
            };
            Ok(AstNode::BooleanLiteral(value))
        } else if self.match_token(TokenType::DateTimeLiteral) {
            // Handle DateTime literals generated by lexer
            Ok(AstNode::DateTimeLiteral(self.previous().lexeme.clone()))
        } else if self.match_token(TokenType::TimeLiteral) {
            // Handle Time literals generated by lexer
            Ok(AstNode::DateTimeLiteral(self.previous().lexeme.clone()))
        } else if self.match_token(TokenType::DateLiteral) {
            // Handle Date literals generated by lexer
            Ok(AstNode::DateTimeLiteral(self.previous().lexeme.clone()))
        } else if self.match_token(TokenType::LeftBrace) {
            // Handle empty collections {}
            self.consume(TokenType::RightBrace, "Expected '}' after empty collection")?;
            Ok(AstNode::Identifier("{}".to_string())) // Represent empty collection as special identifier
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            Ok(expr)
        } else if self.match_token(TokenType::Dollar) {
            // Context variable or special invocation - expect identifier after $
            if self.match_token(TokenType::Identifier) {
                let identifier = self.previous().lexeme.clone();
                match identifier.as_str() {
                    "this" => Ok(AstNode::Identifier("$this".to_string())),
                    "index" => Ok(AstNode::Identifier("$index".to_string())),
                    "total" => Ok(AstNode::Identifier("$total".to_string())),
                    _ => {
                        // Regular context variable
                        let var_name = format!("${}", identifier);
                        Ok(AstNode::Identifier(var_name))
                    }
                }
            } else {
                Err(FhirPathError::ParserError(
                    "Expected variable name after $".to_string(),
                ))
            }
        } else if self.match_token(TokenType::Percent) {
            // Variable reference - expect identifier or delimited identifier after %
            if self.match_token(TokenType::Identifier) {
                let var_name = self.previous().lexeme.clone();
                Ok(AstNode::Variable(var_name))
            } else if self.match_token(TokenType::DelimitedIdentifier) {
                let var_name = self.previous().lexeme.clone();
                Ok(AstNode::Variable(var_name))
            } else {
                Err(FhirPathError::ParserError(
                    "Expected variable name after %".to_string(),
                ))
            }
        } else {
            Err(FhirPathError::ParserError(format!(
                "Expected expression, got {:?}",
                self.peek()
            )))
        }
    }
}

/// Parses a FHIRPath expression from tokens
pub fn parse(tokens: &[Token]) -> Result<AstNode, FhirPathError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
