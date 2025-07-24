# fhirpath-ast

Abstract Syntax Tree (AST) definitions for FHIRPath expressions.

## Overview

This crate provides the core AST types used to represent parsed FHIRPath expressions. It is designed to be lightweight with minimal dependencies, making it suitable for use in parsers, analyzers, and other tools that work with FHIRPath expressions.

## Features

- Complete AST representation for FHIRPath v2.0.0
- Visitor pattern for AST traversal
- Optional serde support for serialization
- Zero runtime dependencies (only serde when enabled)

## Usage

```rust
use fhirpath_ast::{ExpressionNode, BinaryOperator, LiteralValue};

// Create a simple expression: 42 + 8
let expr = ExpressionNode::binary_op(
    BinaryOperator::Add,
    ExpressionNode::literal(LiteralValue::Integer(42)),
    ExpressionNode::literal(LiteralValue::Integer(8))
);

// Use the visitor pattern to traverse the AST
use fhirpath_ast::Visitor;

struct ExpressionPrinter;

impl Visitor for ExpressionPrinter {
    type Result = String;
    
    fn visit_literal(&mut self, literal: &LiteralValue) -> Self::Result {
        match literal {
            LiteralValue::Integer(i) => i.to_string(),
            LiteralValue::String(s) => format!("'{}'", s),
            _ => "...".to_string(),
        }
    }
    
    fn visit_binary_op(
        &mut self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
    ) -> Self::Result {
        let left_str = self.visit_expression(left);
        let right_str = self.visit_expression(right);
        format!("({} {} {})", left_str, op.as_str(), right_str)
    }
    
    // ... implement other visitor methods
}
```

## AST Node Types

- **ExpressionNode**: The main AST node type
  - Literal values (boolean, integer, decimal, string, date/time, quantity)
  - Identifiers and variable references
  - Function calls
  - Binary and unary operations
  - Path navigation and indexing
  - Type checking and casting
  - Lambda expressions
  - Conditional expressions

- **BinaryOperator**: All binary operators (+, -, *, /, =, !=, etc.)
- **UnaryOperator**: All unary operators (not, -, +)
- **LiteralValue**: Literal value types

## License

This project is licensed under the Apache-2.0 license.