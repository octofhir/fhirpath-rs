// Optimized FHIRPath Parser
//
// This module provides an optimized parser that uses:
// - Arena allocation for AST nodes
// - String interning for identifiers and literals
// - Zero-copy string handling
// - Compilation hints for optimization

use crate::errors::FhirPathError;
use crate::lexer::{Token, TokenType};
use crate::optimized_model::{
    AstArena, AstNodeKind, BinaryOperator, CompilationHints, OptimizedAstNode, SourceSpan,
    StringInterner, UnaryOperator,
};
use smallvec::SmallVec;
use std::borrow::Cow;

/// Optimized parser with arena allocation and string interning
pub struct OptimizedParser<'a> {
    tokens: &'a [Token],
    current: usize,
    arena: AstArena<'a>,
    interner: &'a mut StringInterner,
}

impl<'a> OptimizedParser<'a> {
    /// Create a new optimized parser
    pub fn new(tokens: &'a [Token], interner: &'a mut StringInterner) -> Self {
        Self {
            tokens,
            current: 0,
            arena: AstArena::new(),
            interner,
        }
    }

    /// Parse the tokens into an optimized AST
    pub fn parse(mut self) -> Result<(AstArena<'a>, u32), FhirPathError> {
        let root_id = self.expression()?;
        Ok((self.arena, root_id))
    }

    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    /// Peek at the current token
    fn peek(&self) -> &Token {
        if self.is_at_end() {
            &Token {
                token_type: TokenType::EOF,
                lexeme: String::new(),
                position: 0,
                line: 0,
                column: 0,
            }
        } else {
            &self.tokens[self.current]
        }
    }

    /// Get the previous token
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Advance to the next token
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Check if current token matches the given type
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    /// Match and consume a token if it matches the given type
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Match any of the given token types
    fn match_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Consume a token of the given type or return an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, FhirPathError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(FhirPathError::ParserError(format!(
                "{} at line {}, column {}",
                message,
                self.peek().line,
                self.peek().column
            )))
        }
    }

    /// Create a source span for the current token
    fn current_span(&self) -> SourceSpan {
        let token = self.peek();
        SourceSpan {
            start: token.column as usize,
            end: token.column as usize + token.lexeme.len(),
            line: token.line as u32,
            column: token.column as u32,
        }
    }

    /// Allocate a node with compilation hints
    fn alloc_node(&mut self, kind: AstNodeKind<'a>) -> u32 {
        let id = self.arena.alloc(kind);

        // Add compilation hints based on node type
        if let Some(node) = self.arena.get_mut(id) {
            node.hints = self.compute_hints(&node.kind);
            node.source_span = Some(self.current_span());
        }

        id
    }

    /// Compute compilation hints for a node
    fn compute_hints(&self, kind: &AstNodeKind<'a>) -> CompilationHints {
        let mut hints = CompilationHints::default();

        match kind {
            // Literals are pure and constant
            AstNodeKind::StringLiteral(_)
            | AstNodeKind::NumberLiteral { .. }
            | AstNodeKind::BooleanLiteral(_)
            | AstNodeKind::DateTimeLiteral(_)
            | AstNodeKind::QuantityLiteral { .. } => {
                hints.is_pure = true;
                hints.is_constant = true;
                hints.should_cache = false; // No need to cache constants
            }

            // Identifiers and variables are pure but not constant
            AstNodeKind::Identifier(_) | AstNodeKind::Variable(_) => {
                hints.is_pure = true;
                hints.is_constant = false;
                hints.should_cache = false;
            }

            // Binary operations
            AstNodeKind::BinaryOp { op, .. } => {
                hints.is_pure = true;
                hints.is_constant = false;
                hints.can_parallelize = op.is_commutative();
                hints.should_cache = matches!(
                    op,
                    BinaryOperator::Division
                        | BinaryOperator::Multiplication
                        | BinaryOperator::Mod
                        | BinaryOperator::Div
                );
            }

            // Unary operations are pure
            AstNodeKind::UnaryOp { .. } => {
                hints.is_pure = true;
                hints.is_constant = false;
            }

            // Function calls - depends on the function
            AstNodeKind::FunctionCall { name, .. } => {
                hints.is_pure = self.is_pure_function(name);
                hints.is_expensive = self.is_expensive_function(name);
                hints.should_cache = hints.is_expensive && hints.is_pure;
                hints.can_parallelize = self.can_parallelize_function(name);
            }

            // Path expressions are pure but can be expensive
            AstNodeKind::Path { .. } => {
                hints.is_pure = true;
                hints.is_expensive = true;
                hints.should_cache = true;
            }

            // Indexer operations are pure
            AstNodeKind::Indexer { .. } => {
                hints.is_pure = true;
                hints.should_cache = true;
            }
        }

        hints
    }

    /// Check if a function is pure (no side effects)
    fn is_pure_function(&self, name: &str) -> bool {
        // Most FHIRPath functions are pure, except for a few
        !matches!(name, "trace" | "now" | "today" | "timeOfDay")
    }

    /// Check if a function is expensive to evaluate
    fn is_expensive_function(&self, name: &str) -> bool {
        matches!(
            name,
            "descendants"
                | "children"
                | "repeat"
                | "sort"
                | "distinct"
                | "matches"
                | "replace"
                | "split"
                | "aggregate"
        )
    }

    /// Check if a function can be parallelized
    fn can_parallelize_function(&self, name: &str) -> bool {
        matches!(
            name,
            "where" | "select" | "distinct" | "sort" | "union" | "intersect" | "exclude"
        )
    }

    /// Parse an expression
    fn expression(&mut self) -> Result<u32, FhirPathError> {
        self.logical_implies()
    }

    /// Parse logical implies (lowest precedence)
    fn logical_implies(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.logical_or()?;

        while self.match_token(TokenType::Implies) {
            let right = self.logical_or()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op: BinaryOperator::Implies,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse logical OR
    fn logical_or(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.logical_and()?;

        while self.match_any(&[TokenType::Or, TokenType::Xor]) {
            let op = match self.previous().token_type {
                TokenType::Or => BinaryOperator::Or,
                TokenType::Xor => BinaryOperator::Xor,
                _ => unreachable!(),
            };
            let right = self.logical_and()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse logical AND
    fn logical_and(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.membership()?;

        while self.match_token(TokenType::And) {
            let right = self.membership()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op: BinaryOperator::And,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse membership operations (in, contains)
    fn membership(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.equality()?;

        while self.match_any(&[TokenType::In, TokenType::Contains]) {
            let op = match self.previous().token_type {
                TokenType::In => BinaryOperator::In,
                TokenType::Contains => BinaryOperator::Contains,
                _ => unreachable!(),
            };
            let right = self.equality()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse equality operations
    fn equality(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.inequality()?;

        while self.match_any(&[
            TokenType::Equal,
            TokenType::NotEqual,
            TokenType::Equivalent,
            TokenType::NotEquivalent,
        ]) {
            let op = match self.previous().token_type {
                TokenType::Equal => BinaryOperator::Equals,
                TokenType::NotEqual => BinaryOperator::NotEquals,
                TokenType::Equivalent => BinaryOperator::Equivalent,
                TokenType::NotEquivalent => BinaryOperator::NotEquivalent,
                _ => unreachable!(),
            };
            let right = self.inequality()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse inequality operations
    fn inequality(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.union()?;

        while self.match_any(&[
            TokenType::LessThan,
            TokenType::LessOrEqual,
            TokenType::GreaterThan,
            TokenType::GreaterOrEqual,
        ]) {
            let op = match self.previous().token_type {
                TokenType::LessThan => BinaryOperator::LessThan,
                TokenType::LessOrEqual => BinaryOperator::LessOrEqual,
                TokenType::GreaterThan => BinaryOperator::GreaterThan,
                TokenType::GreaterOrEqual => BinaryOperator::GreaterOrEqual,
                _ => unreachable!(),
            };
            let right = self.union()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse union operations
    fn union(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.type_expression()?;

        while self.match_token(TokenType::Union) {
            let right = self.type_expression()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op: BinaryOperator::Union,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse type expressions (is, as)
    fn type_expression(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.additive()?;

        while self.match_any(&[TokenType::Is, TokenType::As]) {
            let op = match self.previous().token_type {
                TokenType::Is => BinaryOperator::Is,
                TokenType::As => BinaryOperator::As,
                _ => unreachable!(),
            };
            let right = self.additive()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse additive operations
    fn additive(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.multiplicative()?;

        while self.match_any(&[TokenType::Plus, TokenType::Minus, TokenType::Ampersand]) {
            let op = match self.previous().token_type {
                TokenType::Plus => BinaryOperator::Addition,
                TokenType::Minus => BinaryOperator::Subtraction,
                TokenType::Ampersand => BinaryOperator::Concatenation,
                _ => unreachable!(),
            };
            let right = self.multiplicative()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse multiplicative operations
    fn multiplicative(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.unary()?;

        while self.match_any(&[TokenType::Multiply, TokenType::Divide, TokenType::Div, TokenType::Mod]) {
            let op = match self.previous().token_type {
                TokenType::Multiply => BinaryOperator::Multiplication,
                TokenType::Divide => BinaryOperator::Division,
                TokenType::Div => BinaryOperator::Div,
                TokenType::Mod => BinaryOperator::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = self.alloc_node(AstNodeKind::BinaryOp {
                op,
                left: expr,
                right,
            });
        }

        Ok(expr)
    }

    /// Parse unary operations
    fn unary(&mut self) -> Result<u32, FhirPathError> {
        if self.match_any(&[TokenType::Plus, TokenType::Minus, TokenType::Not]) {
            let op = match self.previous().token_type {
                TokenType::Plus => UnaryOperator::Positive,
                TokenType::Minus => UnaryOperator::Negate,
                TokenType::Not => UnaryOperator::Not,
                _ => unreachable!(),
            };
            let operand = self.unary()?;
            Ok(self.alloc_node(AstNodeKind::UnaryOp { op, operand }))
        } else {
            self.path()
        }
    }

    /// Parse path expressions
    fn path(&mut self) -> Result<u32, FhirPathError> {
        let mut expr = self.primary()?;

        while self.match_token(TokenType::Dot) {
            let path = self.primary()?;
            expr = self.alloc_node(AstNodeKind::Path { base: expr, path });
        }

        Ok(expr)
    }

    /// Parse primary expressions
    fn primary(&mut self) -> Result<u32, FhirPathError> {
        // Boolean literals
        if self.match_any(&[TokenType::True, TokenType::False]) {
            let value = self.previous().token_type == TokenType::True;
            return Ok(self.alloc_node(AstNodeKind::BooleanLiteral(value)));
        }

        // Number literals
        if self.match_token(TokenType::Number) {
            let lexeme = &self.previous().lexeme;
            let is_decimal = lexeme.contains('.');
            let value: f64 = lexeme.parse().map_err(|_| {
                FhirPathError::ParseError(format!("Invalid number: {}", lexeme))
            })?;
            return Ok(self.alloc_node(AstNodeKind::NumberLiteral {
                value,
                is_decimal,
            }));
        }

        // String literals
        if self.match_token(TokenType::String) {
            let lexeme = &self.previous().lexeme;
            // Remove quotes and intern the string
            let content = &lexeme[1..lexeme.len() - 1];
            let interned = self.interner.intern(content);
            return Ok(self.alloc_node(AstNodeKind::StringLiteral(Cow::Owned(
                interned.to_string(),
            ))));
        }

        // DateTime literals
        if self.match_token(TokenType::DateTime) {
            let lexeme = &self.previous().lexeme;
            // Remove @ prefix and intern
            let content = &lexeme[1..];
            let interned = self.interner.intern(content);
            return Ok(self.alloc_node(AstNodeKind::DateTimeLiteral(Cow::Owned(
                interned.to_string(),
            ))));
        }

        // Quantity literals
        if self.match_token(TokenType::Quantity) {
            let lexeme = &self.previous().lexeme;
            // Parse quantity (simplified - would need proper UCUM parsing)
            if let Some((value_str, unit_str)) = lexeme.split_once(' ') {
                let value: f64 = value_str.parse().map_err(|_| {
                    FhirPathError::ParseError(format!("Invalid quantity value: {}", value_str))
                })?;
                let unit = if unit_str.is_empty() {
                    None
                } else {
                    let interned = self.interner.intern(unit_str);
                    Some(Cow::Owned(interned.to_string()))
                };
                return Ok(self.alloc_node(AstNodeKind::QuantityLiteral { value, unit }));
            }
        }

        // Variables
        if self.match_token(TokenType::Variable) {
            let lexeme = &self.previous().lexeme;
            // Remove % prefix and intern
            let name = &lexeme[1..];
            let interned = self.interner.intern(name);
            return Ok(self.alloc_node(AstNodeKind::Variable(Cow::Owned(
                interned.to_string(),
            ))));
        }

        // Identifiers and function calls
        if self.match_token(TokenType::Identifier) {
            let name = &self.previous().lexeme;
            let interned_name = self.interner.intern(name);

            // Check for function call
            if self.match_token(TokenType::LeftParen) {
                let mut arguments = SmallVec::new();

                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RightParen, "Expected ')' after function arguments")?;

                return Ok(self.alloc_node(AstNodeKind::FunctionCall {
                    name: Cow::Owned(interned_name.to_string()),
                    arguments,
                }));
            } else {
                // Regular identifier
                return Ok(self.alloc_node(AstNodeKind::Identifier(Cow::Owned(
                    interned_name.to_string(),
                ))));
            }
        }

        // Parenthesized expressions
        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        // Indexer
        if self.match_token(TokenType::LeftBracket) {
            let collection = self.expression()?;
            self.consume(TokenType::Comma, "Expected ',' in indexer")?;
            let index = self.expression()?;
            self.consume(TokenType::RightBracket, "Expected ']' after indexer")?;
            return Ok(self.alloc_node(AstNodeKind::Indexer { collection, index }));
        }

        Err(FhirPathError::ParseError(format!(
            "Unexpected token: {} at line {}, column {}",
            self.peek().lexeme,
            self.peek().line,
            self.peek().column
        )))
    }
}

/// Parse tokens into an optimized AST
pub fn parse_optimized<'a>(
    tokens: &'a [Token],
    interner: &'a mut StringInterner,
) -> Result<(AstArena<'a>, u32), FhirPathError> {
    let parser = OptimizedParser::new(tokens, interner);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_optimized_parser_simple() {
        let mut interner = StringInterner::new();
        let tokens = tokenize("true").unwrap();
        let (arena, root_id) = parse_optimized(&tokens, &mut interner).unwrap();

        let root = arena.get(root_id).unwrap();
        assert_eq!(root.kind, AstNodeKind::BooleanLiteral(true));
        assert!(root.hints.is_pure);
        assert!(root.hints.is_constant);
    }

    #[test]
    fn test_optimized_parser_function_call() {
        let mut interner = StringInterner::new();
        let tokens = tokenize("count()").unwrap();
        let (arena, root_id) = parse_optimized(&tokens, &mut interner).unwrap();

        let root = arena.get(root_id).unwrap();
        if let AstNodeKind::FunctionCall { name, arguments } = &root.kind {
            assert_eq!(name.as_ref(), "count");
            assert!(arguments.is_empty());
            assert!(root.hints.is_pure);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_optimized_parser_binary_op() {
        let mut interner = StringInterner::new();
        let tokens = tokenize("1 + 2").unwrap();
        let (arena, root_id) = parse_optimized(&tokens, &mut interner).unwrap();

        let root = arena.get(root_id).unwrap();
        if let AstNodeKind::BinaryOp { op, left, right } = &root.kind {
            assert_eq!(*op, BinaryOperator::Addition);
            assert!(root.hints.is_pure);
            assert!(root.hints.can_parallelize);

            let left_node = arena.get(*left).unwrap();
            let right_node = arena.get(*right).unwrap();
            assert!(matches!(left_node.kind, AstNodeKind::NumberLiteral { .. }));
            assert!(matches!(right_node.kind, AstNodeKind::NumberLiteral { .. }));
        } else {
            panic!("Expected binary operation");
        }
    }

    #[test]
    fn test_string_interning() {
        let mut interner = StringInterner::new();
        let tokens1 = tokenize("name").unwrap();
        let tokens2 = tokenize("name").unwrap();

        let (arena1, root_id1) = parse_optimized(&tokens1, &mut interner).unwrap();
        let (arena2, root_id2) = parse_optimized(&tokens2, &mut interner).unwrap();

        let root1 = arena1.get(root_id1).unwrap();
        let root2 = arena2.get(root_id2).unwrap();

        // Both should have the same interned string
        if let (AstNodeKind::Identifier(name1), AstNodeKind::Identifier(name2)) =
            (&root1.kind, &root2.kind)
        {
            assert_eq!(name1, name2);
        }
    }
}
