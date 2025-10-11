//! Directive parsing (@input, @input-file)

use anyhow::Result;
use lsp_types::Range;
use regex::Regex;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

/// Directive in a FHIRPath document
#[derive(Debug, Clone)]
pub struct Directive {
    /// Type of directive
    pub kind: DirectiveKind,
    /// Location in document
    pub range: Range,
    /// Directive content
    pub content: DirectiveContent,
}

/// Type of directive
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveKind {
    /// @input { ... } - Inline FHIR resource
    Input,
    /// @input-file path - External resource file
    InputFile,
}

/// Content of a directive
#[derive(Debug, Clone)]
pub enum DirectiveContent {
    /// Inline JSON resource
    InlineResource(JsonValue),
    /// File path (absolute or relative)
    FilePath {
        /// The path string from the directive
        path: String,
        /// Resolved absolute path (if file exists)
        resolved: Option<PathBuf>,
    },
}

/// Parsed expression in a document
#[derive(Debug, Clone)]
pub struct ParsedExpression {
    /// Expression text
    pub text: String,
    /// Location in document
    pub range: Range,
}

/// Parser for FHIRPath document directives and expressions
pub struct DirectiveParser {
    /// Regex for multiline comment blocks
    comment_regex: Regex,
    /// Regex for @input directive
    input_regex: Regex,
    /// Regex for @input-file directive
    input_file_regex: Regex,
}

impl DirectiveParser {
    /// Create a new directive parser
    pub fn new() -> Self {
        Self {
            // Match /** ... */ comments
            comment_regex: Regex::new(r"/\*\*[\s\S]*?\*/").unwrap(),
            // Match @input within comment
            input_regex: Regex::new(r"@input\s+(\{[\s\S]*?\})").unwrap(),
            // Match @input-file path/to/file.json
            input_file_regex: Regex::new(r"@input-file\s+([^\s\n]+)").unwrap(),
        }
    }

    /// Parse directives from document text
    pub fn parse_directives(
        &self,
        text: &str,
        workspace_root: Option<&Path>,
    ) -> Result<Vec<Directive>> {
        let mut directives = Vec::new();

        // Find all multiline comments
        for comment_match in self.comment_regex.find_iter(text) {
            let comment_text = comment_match.as_str();
            let comment_start = comment_match.start();

            // Clean comment text: remove /** */ and leading * on each line
            let cleaned_comment = self.clean_comment(comment_text);

            // Check for @input directive
            if let Some(input_match) = self.input_regex.captures(&cleaned_comment) {
                let json_str = input_match.get(1).unwrap().as_str();

                match serde_json::from_str::<JsonValue>(json_str) {
                    Ok(resource) => {
                        let range = self.offset_to_range(text, comment_start, comment_match.end());
                        directives.push(Directive {
                            kind: DirectiveKind::Input,
                            range,
                            content: DirectiveContent::InlineResource(resource),
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse @input JSON: {}", e);
                    }
                }
            }

            // Check for @input-file directive
            if let Some(file_match) = self.input_file_regex.captures(&cleaned_comment) {
                let path_str = file_match.get(1).unwrap().as_str().trim();

                let resolved =
                    workspace_root.and_then(|root| self.resolve_path(root, path_str).ok());

                let range = self.offset_to_range(text, comment_start, comment_match.end());
                directives.push(Directive {
                    kind: DirectiveKind::InputFile,
                    range,
                    content: DirectiveContent::FilePath {
                        path: path_str.to_string(),
                        resolved,
                    },
                });
            }
        }

        Ok(directives)
    }

    /// Parse expressions (separated by semicolon)
    pub fn parse_expressions(&self, text: &str) -> Vec<ParsedExpression> {
        let mut expressions = Vec::new();

        // Remove all comments first
        let text_without_comments = self.comment_regex.replace_all(text, "");

        // Split by semicolon
        let mut current_start = 0;
        for part in text_without_comments.split(';') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                let range = self.offset_to_range(
                    &text_without_comments,
                    current_start,
                    current_start + part.len(),
                );

                expressions.push(ParsedExpression {
                    text: trimmed.to_string(),
                    range,
                });
            }
            current_start += part.len() + 1; // +1 for semicolon
        }

        expressions
    }

    /// Clean comment text by removing comment markers
    fn clean_comment(&self, comment_text: &str) -> String {
        comment_text
            .lines()
            .map(|line| {
                // Remove leading whitespace and * markers
                line.trim_start()
                    .trim_start_matches('/')
                    .trim_start_matches('*')
                    .trim_start()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Resolve a path relative to workspace root
    fn resolve_path(&self, workspace_root: &Path, path_str: &str) -> Result<PathBuf> {
        let path = Path::new(path_str);

        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            workspace_root.join(path)
        };

        if resolved.exists() {
            Ok(resolved)
        } else {
            anyhow::bail!("File not found: {}", resolved.display())
        }
    }

    /// Convert byte offsets to LSP range
    fn offset_to_range(&self, text: &str, start: usize, end: usize) -> Range {
        let start_pos = self.offset_to_position(text, start);
        let end_pos = self.offset_to_position(text, end);
        Range::new(start_pos, end_pos)
    }

    /// Convert byte offset to LSP position
    fn offset_to_position(&self, text: &str, offset: usize) -> lsp_types::Position {
        let mut line = 0;
        let mut character = 0;

        for (i, ch) in text.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }
        }

        lsp_types::Position::new(line, character)
    }
}

impl Default for DirectiveParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_directive() {
        let parser = DirectiveParser::new();
        let text = r#"
/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example"
 * }
 */

Patient.name.family
"#;

        let directives = parser.parse_directives(text, None).unwrap();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, DirectiveKind::Input);
    }

    #[test]
    fn test_parse_input_file_directive() {
        let parser = DirectiveParser::new();
        let text = r#"
/** @input-file ./examples/patient.json */

Patient.birthDate
"#;

        let directives = parser.parse_directives(text, None).unwrap();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, DirectiveKind::InputFile);

        if let DirectiveContent::FilePath { path, .. } = &directives[0].content {
            assert_eq!(path, "./examples/patient.json");
        }
    }

    #[test]
    fn test_parse_expressions() {
        let parser = DirectiveParser::new();
        let text = "Patient.name.family; Patient.birthDate; Patient.gender";

        let expressions = parser.parse_expressions(text);
        assert_eq!(expressions.len(), 3);
        assert_eq!(expressions[0].text, "Patient.name.family");
        assert_eq!(expressions[1].text, "Patient.birthDate");
        assert_eq!(expressions[2].text, "Patient.gender");
    }
}
