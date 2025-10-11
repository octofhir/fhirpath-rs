//! Code actions feature implementation
//!
//! Provides quick fixes and refactoring suggestions:
//! - Function name corrections (did you mean...)
//! - Simplification suggestions
//! - Extract to variable
//! - Convert to method chain

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionResponse, Diagnostic, Range,
    TextEdit, WorkspaceEdit,
};
use std::collections::HashMap;

use crate::document::FhirPathDocument;
use octofhir_fhirpath::evaluator::create_function_registry;

/// Generate code actions for the given document, range, and diagnostics
pub fn generate_code_actions(
    document: &FhirPathDocument,
    range: Range,
    diagnostics: &[Diagnostic],
) -> CodeActionResponse {
    let mut actions = Vec::new();

    // Generate quick fixes for diagnostics
    for diagnostic in diagnostics {
        if let Some(fixes) = generate_quick_fixes(document, diagnostic) {
            actions.extend(fixes);
        }
    }

    // Generate refactoring actions for the selected range
    actions.extend(generate_refactoring_actions(document, range));

    CodeActionResponse::from(actions)
}

/// Generate quick fixes for a diagnostic
fn generate_quick_fixes(
    document: &FhirPathDocument,
    diagnostic: &Diagnostic,
) -> Option<Vec<CodeActionOrCommand>> {
    let mut fixes = Vec::new();

    // Check for "function not found" errors
    if (diagnostic.message.contains("not found") || diagnostic.message.contains("unknown"))
        && let Some(function_name) = extract_function_name(&diagnostic.message)
    {
        // Suggest similar function names
        if let Some(suggestions) = suggest_similar_functions(&function_name) {
            for suggestion in suggestions {
                fixes.push(create_replace_action(
                    &format!("Did you mean '{}'?", suggestion),
                    document,
                    diagnostic.range,
                    &suggestion,
                ));
            }
        }
    }

    // Check for syntax errors
    if diagnostic.message.contains("syntax error") || diagnostic.message.contains("expected") {
        // Suggest common fixes
        if diagnostic.message.contains("expected ')'") {
            fixes.push(create_insert_action(
                "Add closing parenthesis",
                document,
                diagnostic.range.end,
                ")",
            ));
        } else if diagnostic.message.contains("expected '('") {
            fixes.push(create_insert_action(
                "Add opening parenthesis",
                document,
                diagnostic.range.start,
                "(",
            ));
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

/// Generate refactoring actions for a range
fn generate_refactoring_actions(
    document: &FhirPathDocument,
    range: Range,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Get text in range
    let text = document.get_range_text(range);

    if text.is_empty() {
        return actions;
    }

    // Suggest simplifications
    if text.contains(".count() > 0") {
        actions.push(create_replace_action(
            "Simplify to .exists()",
            document,
            range,
            &text.replace(".count() > 0", ".exists()"),
        ));
    }

    if text.contains(".count() = 0") {
        actions.push(create_replace_action(
            "Simplify to .empty()",
            document,
            range,
            &text.replace(".count() = 0", ".empty()"),
        ));
    }

    if text.contains(".where(") && text.contains(").first()") {
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Consider using single() if expecting one result".to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }));
    }

    actions
}

/// Extract function name from error message
fn extract_function_name(message: &str) -> Option<String> {
    // Try to extract function name from common error patterns
    if let Some(start) = message.find('\'')
        && let Some(end) = message[start + 1..].find('\'')
    {
        return Some(message[start + 1..start + 1 + end].to_string());
    }
    None
}

/// Suggest similar function names using Levenshtein distance
fn suggest_similar_functions(name: &str) -> Option<Vec<String>> {
    let registry = create_function_registry();
    let all_functions = registry.list_functions();

    let mut suggestions: Vec<(String, usize)> = all_functions
        .iter()
        .map(|func_name| {
            let distance = levenshtein_distance(name, func_name);
            ((*func_name).clone(), distance)
        })
        .filter(|(_, distance)| *distance <= 3) // Only suggest if distance is small
        .collect();

    suggestions.sort_by_key(|(_, distance)| *distance);

    let results: Vec<String> = suggestions
        .into_iter()
        .take(3) // Top 3 suggestions
        .map(|(name, _)| name)
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Create a code action that replaces text in a range
fn create_replace_action(
    title: &str,
    document: &FhirPathDocument,
    range: Range,
    new_text: &str,
) -> CodeActionOrCommand {
    let mut changes = HashMap::new();
    changes.insert(
        document.uri.clone(),
        vec![TextEdit {
            range,
            new_text: new_text.to_string(),
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

/// Create a code action that inserts text at a position
fn create_insert_action(
    title: &str,
    document: &FhirPathDocument,
    position: lsp_types::Position,
    text: &str,
) -> CodeActionOrCommand {
    let mut changes = HashMap::new();
    changes.insert(
        document.uri.clone(),
        vec![TextEdit {
            range: Range::new(position, position),
            new_text: text.to_string(),
        }],
    );

    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("where", "where"), 0);
        assert_eq!(levenshtein_distance("where", "were"), 1);
        assert_eq!(levenshtein_distance("where", "here"), 1);
        // "select" -> "exists" requires more operations (delete s,e,l,e,c,t and insert e,x,i,s,t,s)
        assert_eq!(levenshtein_distance("select", "exists"), 5);
    }

    #[test]
    fn test_suggest_similar_functions() {
        let suggestions = suggest_similar_functions("wher");
        assert!(suggestions.is_some());

        let funcs = suggestions.unwrap();
        assert!(funcs.contains(&"where".to_string()));
    }

    #[test]
    fn test_extract_function_name() {
        let message = "Function 'counnt' not found";
        let name = extract_function_name(message);
        assert_eq!(name, Some("counnt".to_string()));
    }

    #[test]
    fn test_generate_refactoring_actions_exists() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.count() > 0".to_string(),
            1,
        );

        let range = Range::new(
            lsp_types::Position::new(0, 0),
            lsp_types::Position::new(0, 24),
        );

        let actions = generate_refactoring_actions(&doc, range);
        assert!(!actions.is_empty());

        // Check that it suggests .exists()
        let has_exists = actions.iter().any(|action| {
            if let CodeActionOrCommand::CodeAction(ca) = action {
                ca.title.contains("exists")
            } else {
                false
            }
        });
        assert!(has_exists);
    }

    #[test]
    fn test_generate_refactoring_actions_empty() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.count() = 0".to_string(),
            1,
        );

        let range = Range::new(
            lsp_types::Position::new(0, 0),
            lsp_types::Position::new(0, 24),
        );

        let actions = generate_refactoring_actions(&doc, range);
        assert!(!actions.is_empty());

        // Check that it suggests .empty()
        let has_empty = actions.iter().any(|action| {
            if let CodeActionOrCommand::CodeAction(ca) = action {
                ca.title.contains("empty")
            } else {
                false
            }
        });
        assert!(has_empty);
    }

    #[test]
    fn test_generate_quick_fixes() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.wher()".to_string(),
            1,
        );

        let diagnostic = Diagnostic {
            range: Range::new(
                lsp_types::Position::new(0, 13),
                lsp_types::Position::new(0, 17),
            ),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("fhirpath".to_string()),
            message: "Function 'wher' not found".to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        let fixes = generate_quick_fixes(&doc, &diagnostic);
        assert!(fixes.is_some());

        let actions = fixes.unwrap();
        assert!(!actions.is_empty());
    }

    #[test]
    fn test_generate_code_actions() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.count() > 0".to_string(),
            1,
        );

        let range = Range::new(
            lsp_types::Position::new(0, 0),
            lsp_types::Position::new(0, 24),
        );

        let response = generate_code_actions(&doc, range, &[]);

        // CodeActionResponse is Vec<CodeActionOrCommand>
        assert!(!response.is_empty());
    }
}
