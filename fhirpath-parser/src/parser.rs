//! FHIRPath expression parser implementation

use crate::error::{ParseError, ParseResult};
use crate::span::{Span, Spanned};
use crate::tokenizer::{Token, tokenize};
use crate::lexer::TokenStream;
use fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator, LiteralValue};
// Parser implementation - nom not needed for now as we use token streams

/// Parse a FHIRPath expression string
pub fn parse_expression(input: &str) -> ParseResult<ExpressionNode> {
    let tokens = tokenize(input)?;
    let mut stream = TokenStream::new(tokens);
    parse_expr(&mut stream)
}

/// Parse a complete expression
fn parse_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    parse_or_expr(stream)
}

/// Parse OR expression (lowest precedence)
fn parse_or_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_xor_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::Or => {
                stream.next();
                let right = parse_xor_expr(stream)?;
                left = ExpressionNode::binary_op(BinaryOperator::Or, left, right);
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse XOR expression
fn parse_xor_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_and_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::Xor => {
                stream.next();
                let right = parse_and_expr(stream)?;
                left = ExpressionNode::binary_op(BinaryOperator::Xor, left, right);
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse AND expression
fn parse_and_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_implies_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::And => {
                stream.next();
                let right = parse_implies_expr(stream)?;
                left = ExpressionNode::binary_op(BinaryOperator::And, left, right);
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse IMPLIES expression
fn parse_implies_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_union_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::Implies => {
                stream.next();
                let right = parse_union_expr(stream)?;
                left = ExpressionNode::binary_op(BinaryOperator::Implies, left, right);
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse UNION expression
fn parse_union_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_equality_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::Union => {
                stream.next();
                let right = parse_equality_expr(stream)?;
                left = ExpressionNode::Union {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse equality/equivalence expressions
fn parse_equality_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_relational_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        let op = match &token.value {
            Token::Equal => Some(BinaryOperator::Equal),
            Token::NotEqual => Some(BinaryOperator::NotEqual),
            Token::Equivalent => Some(BinaryOperator::Equivalent),
            Token::NotEquivalent => Some(BinaryOperator::NotEquivalent),
            _ => None,
        };
        
        if let Some(op) = op {
            stream.next();
            let right = parse_relational_expr(stream)?;
            left = ExpressionNode::binary_op(op, left, right);
        } else {
            break;
        }
    }
    
    Ok(left)
}

/// Parse relational expressions
fn parse_relational_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_membership_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        let op = match &token.value {
            Token::LessThan => Some(BinaryOperator::LessThan),
            Token::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
            Token::GreaterThan => Some(BinaryOperator::GreaterThan),
            Token::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
            _ => None,
        };
        
        if let Some(op) = op {
            stream.next();
            let right = parse_membership_expr(stream)?;
            left = ExpressionNode::binary_op(op, left, right);
        } else {
            break;
        }
    }
    
    Ok(left)
}

/// Parse membership expressions (in, contains)
fn parse_membership_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_type_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        let op = match &token.value {
            Token::In => Some(BinaryOperator::In),
            Token::Contains => Some(BinaryOperator::Contains),
            _ => None,
        };
        
        if let Some(op) = op {
            stream.next();
            let right = parse_type_expr(stream)?;
            left = ExpressionNode::binary_op(op, left, right);
        } else {
            break;
        }
    }
    
    Ok(left)
}

/// Parse type expressions (is, as)
fn parse_type_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_additive_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        match &token.value {
            Token::Is => {
                stream.next();
                let type_name = parse_type_specifier(stream)?;
                left = ExpressionNode::TypeCheck {
                    expression: Box::new(left),
                    type_name,
                };
            }
            Token::As => {
                stream.next();
                let type_name = parse_type_specifier(stream)?;
                left = ExpressionNode::TypeCast {
                    expression: Box::new(left),
                    type_name,
                };
            }
            _ => break,
        }
    }
    
    Ok(left)
}

/// Parse additive expressions
fn parse_additive_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_multiplicative_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        let op = match &token.value {
            Token::Plus => Some(BinaryOperator::Add),
            Token::Minus => Some(BinaryOperator::Subtract),
            Token::And => Some(BinaryOperator::Concatenate), // String concatenation uses &
            _ => None,
        };
        
        if let Some(op) = op {
            stream.next();
            let right = parse_multiplicative_expr(stream)?;
            left = ExpressionNode::binary_op(op, left, right);
        } else {
            break;
        }
    }
    
    Ok(left)
}

/// Parse multiplicative expressions
fn parse_multiplicative_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut left = parse_unary_expr(stream)?;
    
    while let Some(token) = stream.peek() {
        let op = match &token.value {
            Token::Multiply => Some(BinaryOperator::Multiply),
            Token::Divide => Some(BinaryOperator::Divide),
            Token::Div => Some(BinaryOperator::IntegerDivide),
            Token::Mod => Some(BinaryOperator::Modulo),
            _ => None,
        };
        
        if let Some(op) = op {
            stream.next();
            let right = parse_unary_expr(stream)?;
            left = ExpressionNode::binary_op(op, left, right);
        } else {
            break;
        }
    }
    
    Ok(left)
}

/// Parse unary expressions
fn parse_unary_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    if let Some(token) = stream.peek() {
        match &token.value {
            Token::Not => {
                stream.next();
                let operand = parse_unary_expr(stream)?;
                return Ok(ExpressionNode::unary_op(UnaryOperator::Not, operand));
            }
            Token::Plus => {
                stream.next();
                let operand = parse_unary_expr(stream)?;
                return Ok(ExpressionNode::unary_op(UnaryOperator::Plus, operand));
            }
            Token::Minus => {
                stream.next();
                let operand = parse_unary_expr(stream)?;
                return Ok(ExpressionNode::unary_op(UnaryOperator::Minus, operand));
            }
            _ => {}
        }
    }
    
    parse_postfix_expr(stream)
}

/// Parse postfix expressions (invocations, indexing, navigation)
fn parse_postfix_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    let mut expr = parse_primary_expr(stream)?;
    
    loop {
        if let Some(token) = stream.peek() {
            match &token.value {
                Token::Dot => {
                    stream.next();
                    let path = parse_identifier(stream)?;
                    expr = ExpressionNode::Path {
                        base: Box::new(expr),
                        path,
                    };
                }
                Token::LeftBracket => {
                    stream.next();
                    let index = parse_expr(stream)?;
                    stream.expect(Token::RightBracket)
                        .map_err(|e| ParseError::SyntaxError {
                            position: stream.position(),
                            message: e,
                        })?;
                    expr = ExpressionNode::Index {
                        base: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::LeftParen => {
                    // Function call
                    if let ExpressionNode::Identifier(name) = expr {
                        stream.next();
                        let args = parse_argument_list(stream)?;
                        stream.expect(Token::RightParen)
                            .map_err(|e| ParseError::SyntaxError {
                                position: stream.position(),
                                message: e,
                            })?;
                        expr = ExpressionNode::function_call(name, args);
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        } else {
            break;
        }
    }
    
    Ok(expr)
}

/// Parse primary expressions
fn parse_primary_expr(stream: &mut TokenStream) -> ParseResult<ExpressionNode> {
    if let Some(token) = stream.next() {
        match token.value {
            // Literals
            Token::Integer(n) => Ok(ExpressionNode::literal(LiteralValue::Integer(n))),
            Token::Decimal(d) => Ok(ExpressionNode::literal(LiteralValue::Decimal(d.to_string()))),
            Token::String(s) => Ok(ExpressionNode::literal(LiteralValue::String(s))),
            Token::True => Ok(ExpressionNode::literal(LiteralValue::Boolean(true))),
            Token::False => Ok(ExpressionNode::literal(LiteralValue::Boolean(false))),
            Token::Date(d) => Ok(ExpressionNode::literal(LiteralValue::Date(d.format("%Y-%m-%d").to_string()))),
            Token::DateTime(dt) => Ok(ExpressionNode::literal(LiteralValue::DateTime(dt.to_rfc3339()))),
            Token::Time(t) => Ok(ExpressionNode::literal(LiteralValue::Time(t.format("%H:%M:%S").to_string()))),
            Token::Empty => Ok(ExpressionNode::literal(LiteralValue::Null)),
            
            // Identifiers
            Token::Identifier(name) => Ok(ExpressionNode::identifier(name)),
            
            // Special variables
            Token::Dollar => {
                let var_name = parse_identifier(stream)?;
                Ok(ExpressionNode::identifier(format!("${}", var_name)))
            }
            
            // Parenthesized expression
            Token::LeftParen => {
                let expr = parse_expr(stream)?;
                stream.expect(Token::RightParen)
                    .map_err(|e| ParseError::SyntaxError {
                        position: stream.position(),
                        message: e,
                    })?;
                Ok(expr)
            }
            
            // Collection literal - not supported in AST literals, so we create a synthetic collection
            Token::LeftBrace => {
                let _elements = parse_collection_elements(stream)?;
                stream.expect(Token::RightBrace)
                    .map_err(|e| ParseError::SyntaxError {
                        position: stream.position(),
                        message: e,
                    })?;
                // For now, return an empty collection as this requires evaluation context
                Ok(ExpressionNode::literal(LiteralValue::Null))
            }
            
            _ => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", token.value),
                position: token.start,
            }),
        }
    } else {
        Err(ParseError::UnexpectedEof)
    }
}

/// Parse an identifier
fn parse_identifier(stream: &mut TokenStream) -> ParseResult<String> {
    if let Some(token) = stream.next() {
        match token.value {
            Token::Identifier(name) => Ok(name),
            _ => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", token.value),
                position: token.start,
            }),
        }
    } else {
        Err(ParseError::UnexpectedEof)
    }
}

/// Parse a type specifier
fn parse_type_specifier(stream: &mut TokenStream) -> ParseResult<String> {
    parse_identifier(stream)
}

/// Parse argument list for function calls
fn parse_argument_list(stream: &mut TokenStream) -> ParseResult<Vec<ExpressionNode>> {
    let mut args = Vec::new();
    
    // Check for empty argument list
    if let Some(token) = stream.peek() {
        if token.value == Token::RightParen {
            return Ok(args);
        }
    }
    
    // Parse first argument
    args.push(parse_expr(stream)?);
    
    // Parse remaining arguments
    while let Some(token) = stream.peek() {
        if token.value == Token::Comma {
            stream.next();
            args.push(parse_expr(stream)?);
        } else {
            break;
        }
    }
    
    Ok(args)
}

/// Parse collection elements
fn parse_collection_elements(stream: &mut TokenStream) -> ParseResult<Vec<ExpressionNode>> {
    let mut elements = Vec::new();
    
    // Check for empty collection
    if let Some(token) = stream.peek() {
        if token.value == Token::RightBrace {
            return Ok(elements);
        }
    }
    
    // Parse first element
    elements.push(parse_expr(stream)?);
    
    // Parse remaining elements
    while let Some(token) = stream.peek() {
        if token.value == Token::Comma {
            stream.next();
            elements.push(parse_expr(stream)?);
        } else {
            break;
        }
    }
    
    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_identifier() {
        let expr = parse_expression("name").unwrap();
        match expr {
            ExpressionNode::Identifier(name) => assert_eq!(name, "name"),
            _ => panic!("Expected identifier"),
        }
    }
    
    #[test]
    fn test_parse_path_navigation() {
        let expr = parse_expression("name.given").unwrap();
        match expr {
            ExpressionNode::Path { base, path } => {
                match *base {
                    ExpressionNode::Identifier(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected identifier base"),
                }
                assert_eq!(path, "given");
            }
            _ => panic!("Expected path expression"),
        }
    }
    
    #[test]
    fn test_parse_function_call() {
        let expr = parse_expression("count()").unwrap();
        match expr {
            ExpressionNode::FunctionCall { name, args } => {
                assert_eq!(name, "count");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected function call"),
        }
    }
    
    #[test]
    fn test_parse_binary_operation() {
        let expr = parse_expression("age > 18").unwrap();
        match expr {
            ExpressionNode::BinaryOp { op, left, right } => {
                assert_eq!(op, BinaryOperator::GreaterThan);
                match *left {
                    ExpressionNode::Identifier(name) => assert_eq!(name, "age"),
                    _ => panic!("Expected identifier on left"),
                }
                match *right {
                    ExpressionNode::Literal(FhirPathValue::Integer(n)) => assert_eq!(n, 18),
                    _ => panic!("Expected integer literal on right"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }
}