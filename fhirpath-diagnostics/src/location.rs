//! Source location tracking for diagnostics

use std::fmt;

/// A position in source text (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed, in UTF-8 bytes)
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a position from a byte offset in the source text
    pub fn from_offset(source: &str, offset: usize) -> Self {
        let mut line = 0;
        let mut column = 0;
        let mut current_offset = 0;

        for ch in source.chars() {
            if current_offset >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += ch.len_utf8();
            }

            current_offset += ch.len_utf8();
        }

        Self { line, column }
    }

    /// Convert to 1-indexed position for display
    pub fn to_display(&self) -> (usize, usize) {
        (self.line + 1, self.column + 1)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.to_display();
        write!(f, "{}:{}", line, col)
    }
}

/// A span in source text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Start position
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Span {
    /// Create a new span
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a span from byte offsets
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        Self {
            start: Position::from_offset(source, start_offset),
            end: Position::from_offset(source, end_offset),
        }
    }

    /// Check if this span contains a position
    pub fn contains(&self, pos: Position) -> bool {
        if pos.line < self.start.line || pos.line > self.end.line {
            return false;
        }

        if pos.line == self.start.line && pos.column < self.start.column {
            return false;
        }

        if pos.line == self.end.line && pos.column >= self.end.column {
            return false;
        }

        true
    }

    /// Merge two spans
    pub fn merge(&self, other: &Span) -> Span {
        let start = if self.start.line < other.start.line {
            self.start
        } else if self.start.line > other.start.line {
            other.start
        } else if self.start.column < other.start.column {
            self.start
        } else {
            other.start
        };

        let end = if self.end.line > other.end.line {
            self.end
        } else if self.end.line < other.end.line {
            other.end
        } else if self.end.column > other.end.column {
            self.end
        } else {
            other.end
        };

        Span { start, end }
    }

    /// Get the length of the span (only valid for single-line spans)
    pub fn len(&self) -> Option<usize> {
        if self.start.line == self.end.line {
            Some(self.end.column - self.start.column)
        } else {
            None
        }
    }

    /// Check if the span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line + 1,
                self.start.column + 1,
                self.end.column + 1
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// Source location information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceLocation {
    /// The span in the source text
    pub span: Span,
    /// The source text at this location (optional)
    pub source_text: Option<String>,
    /// File path (optional)
    pub file_path: Option<String>,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(span: Span) -> Self {
        Self {
            span,
            source_text: None,
            file_path: None,
        }
    }

    /// Create with source text
    pub fn with_source(span: Span, source_text: String) -> Self {
        Self {
            span,
            source_text: Some(source_text),
            file_path: None,
        }
    }

    /// Create with file path
    pub fn with_file(span: Span, file_path: String) -> Self {
        Self {
            span,
            source_text: None,
            file_path: Some(file_path),
        }
    }

    /// Create a complete source location
    pub fn complete(span: Span, source_text: String, file_path: String) -> Self {
        Self {
            span,
            source_text: Some(source_text),
            file_path: Some(file_path),
        }
    }

    /// Get the lines of source text that this location spans
    pub fn get_lines<'a>(&self, full_source: &'a str) -> Vec<&'a str> {
        let lines: Vec<&str> = full_source.lines().collect();
        let start_line = self.span.start.line;
        let end_line = self.span.end.line;

        if start_line >= lines.len() {
            return vec![];
        }

        let end_line = end_line.min(lines.len() - 1);
        lines[start_line..=end_line].to_vec()
    }

    /// Get a snippet of the source text with context
    pub fn get_snippet(&self, full_source: &str, context_lines: usize) -> String {
        let lines: Vec<&str> = full_source.lines().collect();
        let start_line = self.span.start.line.saturating_sub(context_lines);
        let end_line = (self.span.end.line + context_lines).min(lines.len() - 1);

        let mut result = String::new();
        for (i, line) in lines[start_line..=end_line].iter().enumerate() {
            let line_num = start_line + i;
            if line_num >= self.span.start.line && line_num <= self.span.end.line {
                result.push_str(&format!("{:4} | {}\n", line_num + 1, line));
            } else {
                result.push_str(&format!("{:4} | {}\n", line_num + 1, line));
            }
        }

        result
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.file_path {
            write!(f, "{}:{}", path, self.span)
        } else {
            write!(f, "{}", self.span)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_offset() {
        let source = "hello\nworld\ntest";

        assert_eq!(Position::from_offset(source, 0), Position::new(0, 0));
        assert_eq!(Position::from_offset(source, 5), Position::new(0, 5));
        assert_eq!(Position::from_offset(source, 6), Position::new(1, 0));
        assert_eq!(Position::from_offset(source, 12), Position::new(2, 0));
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(Position::new(1, 5), Position::new(1, 10));

        assert!(span.contains(Position::new(1, 5)));
        assert!(span.contains(Position::new(1, 7)));
        assert!(!span.contains(Position::new(1, 10))); // End is exclusive
        assert!(!span.contains(Position::new(0, 7)));
        assert!(!span.contains(Position::new(2, 7)));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(Position::new(1, 5), Position::new(1, 10));
        let span2 = Span::new(Position::new(1, 8), Position::new(2, 3));

        let merged = span1.merge(&span2);
        assert_eq!(merged.start, Position::new(1, 5));
        assert_eq!(merged.end, Position::new(2, 3));
    }

    #[test]
    fn test_source_location_snippet() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let span = Span::new(Position::new(2, 0), Position::new(2, 6));
        let location = SourceLocation::new(span);

        let snippet = location.get_snippet(source, 1);
        assert!(snippet.contains("line 2"));
        assert!(snippet.contains("line 3"));
        assert!(snippet.contains("line 4"));
    }
}
