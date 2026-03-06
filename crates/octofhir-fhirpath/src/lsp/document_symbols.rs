//! Document symbol provider for FHIRPath expressions
//!
//! Provides an outline view of a FHIRPath expression tree.

use crate::ast::expression::ExpressionNode;
use crate::core::SourceLocation;
use crate::parser::parse_with_analysis;

use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// Provider for document symbols (outline) in FHIRPath expressions
pub struct DocumentSymbolProvider;

impl DocumentSymbolProvider {
    /// Generate document symbols for a FHIRPath expression
    #[allow(deprecated)] // DocumentSymbol::children is not deprecated, but some fields trigger this
    pub fn provide(expression: &str) -> Vec<DocumentSymbol> {
        let result = parse_with_analysis(expression);
        let Some(ast) = result.ast else {
            return Vec::new();
        };

        let mut symbols = Vec::new();
        Self::collect_symbols(&ast, expression, &mut symbols);
        symbols
    }

    #[allow(deprecated)]
    fn collect_symbols(node: &ExpressionNode, source: &str, symbols: &mut Vec<DocumentSymbol>) {
        match node {
            ExpressionNode::MethodCall(method) => {
                let range = Self::loc_to_range(method.location.as_ref(), source);
                let selection_range = if let Some(ref loc) = method.location {
                    // Select just the method name
                    Self::find_method_range(source, loc, &method.method).unwrap_or(range)
                } else {
                    range
                };

                let mut children = Vec::new();
                // Collect child symbols from object
                Self::collect_symbols(&method.object, source, &mut children);
                // Collect from arguments
                for arg in &method.arguments {
                    Self::collect_arg_symbols(arg, source, &mut children);
                }

                symbols.push(DocumentSymbol {
                    name: format!(".{}()", method.method),
                    detail: Some(format!("{} argument(s)", method.arguments.len())),
                    kind: SymbolKind::METHOD,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::PropertyAccess(prop) => {
                let range = Self::loc_to_range(prop.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&prop.object, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: format!(".{}", prop.property),
                    detail: Some("property".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::FunctionCall(func) => {
                let range = Self::loc_to_range(func.location.as_ref(), source);

                let mut children = Vec::new();
                for arg in &func.arguments {
                    Self::collect_arg_symbols(arg, source, &mut children);
                }

                symbols.push(DocumentSymbol {
                    name: format!("{}()", func.name),
                    detail: Some(format!("{} argument(s)", func.arguments.len())),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::BinaryOperation(binop) => {
                let range = Self::loc_to_range(binop.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&binop.left, source, &mut children);
                Self::collect_symbols(&binop.right, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: binop.operator.symbol().to_string(),
                    detail: Some("operator".to_string()),
                    kind: SymbolKind::OPERATOR,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::Identifier(id) => {
                let range = Self::loc_to_range(id.location.as_ref(), source);
                let kind = if id.name.starts_with(|c: char| c.is_uppercase()) {
                    SymbolKind::CLASS
                } else {
                    SymbolKind::VARIABLE
                };

                symbols.push(DocumentSymbol {
                    name: id.name.clone(),
                    detail: None,
                    kind,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            ExpressionNode::TypeCast(cast) => {
                let range = Self::loc_to_range(cast.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&cast.expression, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: format!("as {}", cast.target_type),
                    detail: Some("type cast".to_string()),
                    kind: SymbolKind::TYPE_PARAMETER,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::TypeCheck(check) => {
                let range = Self::loc_to_range(check.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&check.expression, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: format!("is {}", check.target_type),
                    detail: Some("type check".to_string()),
                    kind: SymbolKind::TYPE_PARAMETER,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            // For compound nodes, just recurse
            ExpressionNode::Parenthesized(inner) => {
                Self::collect_symbols(inner, source, symbols);
            }
            ExpressionNode::Union(union) => {
                let range = Self::loc_to_range(union.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&union.left, source, &mut children);
                Self::collect_symbols(&union.right, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: "|".to_string(),
                    detail: Some("union".to_string()),
                    kind: SymbolKind::OPERATOR,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            ExpressionNode::IndexAccess(idx) => {
                let range = Self::loc_to_range(idx.location.as_ref(), source);

                let mut children = Vec::new();
                Self::collect_symbols(&idx.object, source, &mut children);

                symbols.push(DocumentSymbol {
                    name: "[index]".to_string(),
                    detail: Some("index access".to_string()),
                    kind: SymbolKind::OPERATOR,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                });
            }
            // Leaf nodes that don't produce symbols
            ExpressionNode::Literal(_)
            | ExpressionNode::Variable(_)
            | ExpressionNode::Lambda(_)
            | ExpressionNode::Collection(_)
            | ExpressionNode::Filter(_)
            | ExpressionNode::Path(_)
            | ExpressionNode::TypeInfo(_)
            | ExpressionNode::UnaryOperation(_) => {}
        }
    }

    /// Collect symbols from function/method arguments
    #[allow(deprecated)]
    fn collect_arg_symbols(node: &ExpressionNode, source: &str, symbols: &mut Vec<DocumentSymbol>) {
        // Only promote complex sub-expressions as child symbols
        match node {
            ExpressionNode::MethodCall(_)
            | ExpressionNode::FunctionCall(_)
            | ExpressionNode::BinaryOperation(_)
            | ExpressionNode::PropertyAccess(_) => {
                Self::collect_symbols(node, source, symbols);
            }
            _ => {}
        }
    }

    /// Convert a SourceLocation to an LSP Range
    fn loc_to_range(loc: Option<&SourceLocation>, source: &str) -> Range {
        if let Some(loc) = loc {
            let start = Self::offset_to_position(source, loc.offset);
            let end = Self::offset_to_position(source, loc.offset + loc.length);
            Range { start, end }
        } else {
            Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            }
        }
    }

    /// Find the range for just the method name within a method call location
    fn find_method_range(source: &str, loc: &SourceLocation, method_name: &str) -> Option<Range> {
        let region = source.get(loc.offset..loc.offset + loc.length)?;
        let needle = format!(".{}", method_name);
        let pos = region.find(&needle)?;
        let name_start = loc.offset + pos + 1; // skip dot
        let start = Self::offset_to_position(source, name_start);
        let end = Self::offset_to_position(source, name_start + method_name.len());
        Some(Range { start, end })
    }

    /// Convert byte offset to LSP Position
    fn offset_to_position(source: &str, offset: usize) -> Position {
        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Position::new(line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_property_access() {
        let symbols = DocumentSymbolProvider::provide("Patient.name");
        assert!(!symbols.is_empty(), "Should produce document symbols");
        // Should have a property access symbol
        assert!(symbols.iter().any(|s| s.name.contains("name")));
    }

    #[test]
    fn test_method_call_symbols() {
        let symbols = DocumentSymbolProvider::provide("Patient.name.where(use = 'official')");
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name.contains("where")));
    }

    #[test]
    fn test_binary_operation_symbols() {
        let symbols = DocumentSymbolProvider::provide("age > 18");
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == ">"));
    }
}
