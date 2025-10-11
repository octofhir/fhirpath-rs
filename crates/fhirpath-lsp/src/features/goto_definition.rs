//! Go to definition feature implementation

use crate::directives::DirectiveContent;
use crate::document::FhirPathDocument;
use lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

/// Generate goto definition response for a position in the document
pub fn goto_definition(
    document: &FhirPathDocument,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    // Check if position is within an @input-file directive
    for directive in &document.directives {
        if position_in_range(position, directive.range)
            && let DirectiveContent::FilePath {
                resolved: Some(path),
                ..
            } = &directive.content
            && let Ok(uri) = Url::from_file_path(path)
        {
            // Return location pointing to the beginning of the file
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri,
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            }));
        }
    }

    None
}

/// Check if a position is within a range
fn position_in_range(pos: Position, range: Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }

    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }

    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_position_in_range() {
        let range = Range::new(Position::new(1, 5), Position::new(3, 10));

        // Position before range
        assert!(!position_in_range(Position::new(0, 0), range));
        assert!(!position_in_range(Position::new(1, 4), range));

        // Position within range
        assert!(position_in_range(Position::new(1, 5), range));
        assert!(position_in_range(Position::new(2, 0), range));
        assert!(position_in_range(Position::new(3, 10), range));

        // Position after range
        assert!(!position_in_range(Position::new(3, 11), range));
        assert!(!position_in_range(Position::new(4, 0), range));
    }

    #[test]
    fn test_goto_definition_with_input_file() {
        // Create a test document with @input-file directive
        let text = r#"/**
 * @input-file ./examples/patient.json
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let mut doc = FhirPathDocument::new(uri, text.to_string(), 1);

        // Manually set resolved path for testing
        if let Some(directive) = doc.directives.first_mut() {
            if let DirectiveContent::FilePath { resolved, .. } = &mut directive.content {
                *resolved = Some(PathBuf::from("/tmp/examples/patient.json"));
            }
        }

        // Position within the directive (line 1, column 5)
        let position = Position::new(1, 5);
        let result = goto_definition(&doc, position);

        assert!(result.is_some());
        if let Some(GotoDefinitionResponse::Scalar(location)) = result {
            assert_eq!(
                location.uri.to_string(),
                "file:///tmp/examples/patient.json"
            );
            assert_eq!(location.range.start, Position::new(0, 0));
        } else {
            panic!("Expected scalar location");
        }
    }

    #[test]
    fn test_goto_definition_with_unresolved_file() {
        // Create a test document with unresolved @input-file directive
        let text = r#"/**
 * @input-file ./nonexistent.json
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        // Position within the directive
        let position = Position::new(1, 5);
        let result = goto_definition(&doc, position);

        // Should return None since file is not resolved
        assert!(result.is_none());
    }

    #[test]
    fn test_goto_definition_outside_directive() {
        // Create a test document with @input-file directive
        let text = r#"/**
 * @input-file ./examples/patient.json
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let mut doc = FhirPathDocument::new(uri, text.to_string(), 1);

        // Manually set resolved path
        if let Some(directive) = doc.directives.first_mut() {
            if let DirectiveContent::FilePath { resolved, .. } = &mut directive.content {
                *resolved = Some(PathBuf::from("/tmp/examples/patient.json"));
            }
        }

        // Position outside the directive (line 4, in the expression)
        let position = Position::new(4, 5);
        let result = goto_definition(&doc, position);

        // Should return None since position is outside directive
        assert!(result.is_none());
    }

    #[test]
    fn test_goto_definition_with_inline_input() {
        // Create a test document with @input directive (not @input-file)
        let text = r#"/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example"
 * }
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        // Position within the directive
        let position = Position::new(2, 5);
        let result = goto_definition(&doc, position);

        // Should return None for inline @input (not a file reference)
        assert!(result.is_none());
    }
}
