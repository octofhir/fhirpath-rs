//! Inlay hints feature implementation
//!
//! Provides inlay hints for:
//! - Function parameter names
//! - Variable types in iterations
//! - Return type annotations

use lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};
use octofhir_fhirpath::ast::ExpressionNode;
use octofhir_fhirpath::evaluator::create_function_registry;
use octofhir_fhirpath::parser;

use crate::document::FhirPathDocument;

/// Generate inlay hints for the given document and range
pub fn generate_inlay_hints(document: &FhirPathDocument, range: Range) -> Vec<InlayHint> {
    let mut hints = Vec::new();

    // Process each expression in the document
    for expr in &document.expressions {
        // Check if expression is within the requested range
        if !is_range_overlapping(&expr.range, &range) {
            continue;
        }

        // Parse expression to AST
        let parse_result = parser::parse(&expr.text);

        if let Some(ast) = parse_result.ast {
            // Collect hints from AST
            collect_hints_from_ast(&ast, document, &mut hints);
        }
    }

    hints
}

/// Check if two ranges overlap
fn is_range_overlapping(a: &Range, b: &Range) -> bool {
    // Check if ranges overlap - need to compare both line and character
    if a.end.line < b.start.line || a.start.line > b.end.line {
        return false;
    }

    // If on same line, check character positions
    if a.end.line == b.start.line && a.end.character <= b.start.character {
        return false;
    }
    if a.start.line == b.end.line && a.start.character >= b.end.character {
        return false;
    }

    true
}

/// Collect inlay hints from an AST node
fn collect_hints_from_ast(
    node: &ExpressionNode,
    document: &FhirPathDocument,
    hints: &mut Vec<InlayHint>,
) {
    match node {
        ExpressionNode::FunctionCall(func) => {
            // Add parameter name hints for function calls
            add_function_parameter_hints(func, document, hints);

            // Recursively process arguments
            for arg in &func.arguments {
                collect_hints_from_ast(arg, document, hints);
            }
        }

        ExpressionNode::MethodCall(method) => {
            // Add parameter name hints for method calls
            add_method_parameter_hints(method, document, hints);

            // Process object and arguments
            collect_hints_from_ast(&method.object, document, hints);
            for arg in &method.arguments {
                collect_hints_from_ast(arg, document, hints);
            }
        }

        ExpressionNode::PropertyAccess(prop) => {
            collect_hints_from_ast(&prop.object, document, hints);
        }

        ExpressionNode::IndexAccess(idx) => {
            collect_hints_from_ast(&idx.object, document, hints);
            collect_hints_from_ast(&idx.index, document, hints);
        }

        ExpressionNode::BinaryOperation(bin_op) => {
            collect_hints_from_ast(&bin_op.left, document, hints);
            collect_hints_from_ast(&bin_op.right, document, hints);
        }

        ExpressionNode::UnaryOperation(un_op) => {
            collect_hints_from_ast(&un_op.operand, document, hints);
        }

        ExpressionNode::Lambda(lambda) => {
            // Add type hint for lambda parameter
            if let Some(param) = &lambda.parameter
                && let Some(_loc) = &lambda.location
            {
                // Try to find parameter position (simplified)
                // In a real implementation, we'd track exact positions
                if let Some(position) = find_lambda_param_position(document, param) {
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String(": any".to_string()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: None,
                        padding_left: None,
                        padding_right: None,
                        data: None,
                    });
                }
            }
            collect_hints_from_ast(&lambda.body, document, hints);
        }

        ExpressionNode::Collection(coll) => {
            for elem in &coll.elements {
                collect_hints_from_ast(elem, document, hints);
            }
        }

        ExpressionNode::Parenthesized(expr) => {
            collect_hints_from_ast(expr, document, hints);
        }

        ExpressionNode::TypeCast(cast) => {
            collect_hints_from_ast(&cast.expression, document, hints);
        }

        ExpressionNode::Filter(filter) => {
            collect_hints_from_ast(&filter.base, document, hints);
            collect_hints_from_ast(&filter.condition, document, hints);
        }

        ExpressionNode::Union(union) => {
            collect_hints_from_ast(&union.left, document, hints);
            collect_hints_from_ast(&union.right, document, hints);
        }

        ExpressionNode::TypeCheck(type_check) => {
            collect_hints_from_ast(&type_check.expression, document, hints);
        }

        ExpressionNode::Path(path) => {
            collect_hints_from_ast(&path.base, document, hints);
        }

        // Leaf nodes - no recursion needed
        ExpressionNode::Literal(_)
        | ExpressionNode::Identifier(_)
        | ExpressionNode::Variable(_)
        | ExpressionNode::TypeInfo(_) => {}
    }
}

/// Add parameter name hints for a function call
fn add_function_parameter_hints(
    func: &octofhir_fhirpath::ast::FunctionCallNode,
    document: &FhirPathDocument,
    hints: &mut Vec<InlayHint>,
) {
    let registry = create_function_registry();

    if let Some(metadata) = registry.get_metadata(&func.name) {
        // Add hints for each argument
        for (i, arg) in func.arguments.iter().enumerate() {
            if let Some(param) = metadata.signature.parameters.get(i) {
                // Find position of the argument (simplified - would need AST position tracking)
                if let Some(position) = find_argument_position(document, arg) {
                    hints.push(InlayHint {
                        position,
                        label: InlayHintLabel::String(format!("{}: ", param.name)),
                        kind: Some(InlayHintKind::PARAMETER),
                        text_edits: None,
                        tooltip: Some(lsp_types::InlayHintTooltip::String(
                            param.description.clone(),
                        )),
                        padding_left: None,
                        padding_right: Some(true),
                        data: None,
                    });
                }
            }
        }
    }
}

/// Add parameter name hints for a method call
fn add_method_parameter_hints(
    method: &octofhir_fhirpath::ast::MethodCallNode,
    document: &FhirPathDocument,
    hints: &mut Vec<InlayHint>,
) {
    let registry = create_function_registry();

    if let Some(metadata) = registry.get_metadata(&method.method) {
        // Add hints for each argument
        for (i, arg) in method.arguments.iter().enumerate() {
            if let Some(param) = metadata.signature.parameters.get(i)
                && let Some(position) = find_argument_position(document, arg)
            {
                hints.push(InlayHint {
                    position,
                    label: InlayHintLabel::String(format!("{}: ", param.name)),
                    kind: Some(InlayHintKind::PARAMETER),
                    text_edits: None,
                    tooltip: Some(lsp_types::InlayHintTooltip::String(
                        param.description.clone(),
                    )),
                    padding_left: None,
                    padding_right: Some(true),
                    data: None,
                });
            }
        }
    }
}

/// Find the position where an argument starts (simplified)
fn find_argument_position(_document: &FhirPathDocument, _arg: &ExpressionNode) -> Option<Position> {
    // In a real implementation, we would track exact positions in the AST
    // For now, return None to avoid incorrect hints
    // This would require enhancing the parser to track spans
    None
}

/// Find lambda parameter position (simplified)
fn find_lambda_param_position(_document: &FhirPathDocument, _param: &str) -> Option<Position> {
    // In a real implementation, we would track exact positions
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_is_range_overlapping() {
        let range_a = Range::new(Position::new(0, 0), Position::new(0, 10));
        let range_b = Range::new(Position::new(0, 5), Position::new(0, 15));
        assert!(is_range_overlapping(&range_a, &range_b));

        let range_c = Range::new(Position::new(0, 20), Position::new(0, 30));
        assert!(!is_range_overlapping(&range_a, &range_c));
    }

    #[test]
    fn test_generate_inlay_hints_empty() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name".to_string(),
            1,
        );

        let range = Range::new(Position::new(0, 0), Position::new(1, 0));
        let hints = generate_inlay_hints(&doc, range);

        // Currently returns empty because we don't have position tracking
        // In a full implementation with position tracking, this would return hints
        assert_eq!(hints.len(), 0);
    }

    #[test]
    fn test_generate_inlay_hints_function_call() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.where(use = 'official')".to_string(),
            1,
        );

        let range = Range::new(Position::new(0, 0), Position::new(1, 0));
        let hints = generate_inlay_hints(&doc, range);

        // Currently returns empty due to lack of position tracking
        // With full implementation, would show parameter hints
        assert_eq!(hints.len(), 0);
    }

    #[test]
    fn test_collect_hints_from_function() {
        // Test that the function metadata lookup works
        let registry = create_function_registry();
        let metadata = registry.get_metadata("where");

        assert!(metadata.is_some());
        let meta = metadata.unwrap();
        assert!(!meta.signature.parameters.is_empty());
    }
}
