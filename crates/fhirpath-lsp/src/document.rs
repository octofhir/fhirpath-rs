//! Document management

use crate::directives::{Directive, DirectiveParser, ParsedExpression};
use lsp_types::{Position, Range, TextDocumentContentChangeEvent};
use url::Url;

/// FHIRPath document representation
#[derive(Debug, Clone)]
pub struct FhirPathDocument {
    /// Document URI
    pub uri: Url,
    /// Full document text
    pub text: String,
    /// Document version (from LSP)
    pub version: i32,
    /// Line start positions (for position ↔ offset conversion)
    line_starts: Vec<usize>,
    /// Parsed directives
    pub directives: Vec<Directive>,
    /// Parsed expressions
    pub expressions: Vec<ParsedExpression>,
}

impl FhirPathDocument {
    /// Create a new document
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        let line_starts = compute_line_starts(&text);
        let mut doc = Self {
            uri,
            text,
            version,
            line_starts,
            directives: Vec::new(),
            expressions: Vec::new(),
        };
        doc.reparse();
        doc
    }

    /// Reparse directives and expressions
    fn reparse(&mut self) {
        let parser = DirectiveParser::new();

        // Extract workspace root from URI
        let workspace_root = self
            .uri
            .to_file_path()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        // Parse directives
        self.directives = parser
            .parse_directives(&self.text, workspace_root.as_deref())
            .unwrap_or_default();

        // Parse expressions
        self.expressions = parser.parse_expressions(&self.text);
    }

    /// Apply an incremental text change
    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent, version: i32) {
        self.version = version;

        match change.range {
            Some(range) => {
                // Incremental change
                let start_offset = self.position_to_offset(range.start);
                let end_offset = self.position_to_offset(range.end);

                self.text
                    .replace_range(start_offset..end_offset, &change.text);
                self.line_starts = compute_line_starts(&self.text);
            }
            None => {
                // Full document sync
                self.text = change.text;
                self.line_starts = compute_line_starts(&self.text);
            }
        }

        // Reparse after change
        self.reparse();
    }

    /// Convert LSP position to byte offset
    pub fn position_to_offset(&self, position: Position) -> usize {
        let line = position.line as usize;
        let character = position.character as usize;

        if line >= self.line_starts.len() {
            return self.text.len();
        }

        let line_start = self.line_starts[line];
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text.len());

        let line_text = &self.text[line_start..line_end];
        let char_offset = line_text
            .char_indices()
            .nth(character)
            .map(|(offset, _)| offset)
            .unwrap_or(line_text.len());

        line_start + char_offset
    }

    /// Convert byte offset to LSP position
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self
            .line_starts
            .iter()
            .position(|&start| start > offset)
            .map(|line| line - 1)
            .unwrap_or(self.line_starts.len().saturating_sub(1));

        let line_start = self.line_starts[line];
        let line_text = &self.text[line_start..offset.min(self.text.len())];
        let character = line_text.chars().count();

        Position::new(line as u32, character as u32)
    }

    /// Get text in a range
    pub fn get_range_text(&self, range: Range) -> String {
        let start = self.position_to_offset(range.start);
        let end = self.position_to_offset(range.end);
        self.text[start..end].to_string()
    }
}

/// Compute line start positions for a text
fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_offset() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "line1\nline2\nline3".to_string(),
            1,
        );

        assert_eq!(doc.position_to_offset(Position::new(0, 0)), 0);
        assert_eq!(doc.position_to_offset(Position::new(0, 5)), 5);
        assert_eq!(doc.position_to_offset(Position::new(1, 0)), 6);
        assert_eq!(doc.position_to_offset(Position::new(1, 5)), 11);
    }

    #[test]
    fn test_offset_to_position() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "line1\nline2\nline3".to_string(),
            1,
        );

        assert_eq!(doc.offset_to_position(0), Position::new(0, 0));
        assert_eq!(doc.offset_to_position(5), Position::new(0, 5));
        assert_eq!(doc.offset_to_position(6), Position::new(1, 0));
        assert_eq!(doc.offset_to_position(11), Position::new(1, 5));
    }
}
