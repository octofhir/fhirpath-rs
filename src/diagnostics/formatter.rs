//! Formatting diagnostics for different output formats

use super::diagnostic::Diagnostic;
#[cfg(feature = "terminal")]
use super::diagnostic::Severity;

/// Output format for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
    /// Compact single-line format
    Compact,
}

/// Formatter for diagnostics
pub struct DiagnosticFormatter {
    format: Format,
    show_code: bool,
    show_suggestions: bool,
    #[cfg(feature = "terminal")]
    use_color: bool,
}

impl DiagnosticFormatter {
    /// Create a new formatter
    pub fn new(format: Format) -> Self {
        Self {
            format,
            show_code: true,
            show_suggestions: true,
            #[cfg(feature = "terminal")]
            use_color: true,
        }
    }

    /// Set whether to show error codes
    pub fn with_code(mut self, show: bool) -> Self {
        self.show_code = show;
        self
    }

    /// Set whether to show suggestions
    pub fn with_suggestions(mut self, show: bool) -> Self {
        self.show_suggestions = show;
        self
    }

    /// Set whether to use color (terminal feature only)
    #[cfg(feature = "terminal")]
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    /// Format a diagnostic
    pub fn format(&self, diagnostic: &Diagnostic) -> String {
        match self.format {
            Format::Text => self.format_text(diagnostic),
            Format::Json => self.format_json(diagnostic),
            Format::Compact => self.format_compact(diagnostic),
        }
    }

    /// Format multiple diagnostics
    pub fn format_all(&self, diagnostics: &[Diagnostic]) -> String {
        match self.format {
            Format::Text => diagnostics
                .iter()
                .map(|d| self.format_text(d))
                .collect::<Vec<_>>()
                .join("\n\n"),
            Format::Json => {
                #[cfg(feature = "serde")]
                {
                    let json_diagnostics: Vec<_> = diagnostics
                        .iter()
                        .map(|d| serde_json::to_value(d).unwrap())
                        .collect();
                    serde_json::to_string_pretty(&json_diagnostics).unwrap()
                }
                #[cfg(not(feature = "serde"))]
                {
                    format!("{diagnostics:?}")
                }
            }
            Format::Compact => diagnostics
                .iter()
                .map(|d| self.format_compact(d))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    fn format_text(&self, diagnostic: &Diagnostic) -> String {
        let mut result = String::new();

        // Header line
        #[cfg(feature = "terminal")]
        if self.use_color {
            use colored::Colorize;
            let severity_str = match diagnostic.severity {
                Severity::Error => "error".red().bold(),
                Severity::Warning => "warning".yellow().bold(),
                Severity::Info => "info".blue().bold(),
                Severity::Hint => "hint".green().bold(),
            };

            result.push_str(&format!("{}: {}", severity_str, diagnostic.message.bold()));
        } else {
            result.push_str(&format!("{}: {}", diagnostic.severity, diagnostic.message));
        }

        #[cfg(not(feature = "terminal"))]
        result.push_str(&format!("{}: {}", diagnostic.severity, diagnostic.message));

        if self.show_code {
            result.push_str(&format!(" [{}]", diagnostic.code_string()));
        }

        result.push('\n');

        // Location
        result.push_str(&format!(" --> {}\n", diagnostic.location));

        // Source text if available
        if let Some(source) = &diagnostic.location.source_text {
            let lines: Vec<&str> = source.lines().collect();
            let start_line = diagnostic.location.span.start.line;
            let end_line = diagnostic.location.span.end.line;

            // Show source lines
            for line_idx in start_line..=end_line {
                if line_idx < lines.len() {
                    result.push_str(&format!("{:4} | {}\n", line_idx + 1, lines[line_idx]));

                    // Underline the problematic part
                    if line_idx == start_line {
                        let start_col = diagnostic.location.span.start.column;
                        let end_col = if line_idx == end_line {
                            diagnostic.location.span.end.column
                        } else {
                            lines[line_idx].len()
                        };

                        result.push_str("     | ");
                        result.push_str(&" ".repeat(start_col));

                        #[cfg(feature = "terminal")]
                        if self.use_color {
                            use colored::Colorize;
                            let underline = "^".repeat(end_col - start_col);
                            result.push_str(&match diagnostic.severity {
                                Severity::Error => underline.red().to_string(),
                                Severity::Warning => underline.yellow().to_string(),
                                Severity::Info => underline.blue().to_string(),
                                Severity::Hint => underline.green().to_string(),
                            });
                        } else {
                            result.push_str(&"^".repeat(end_col - start_col));
                        }

                        #[cfg(not(feature = "terminal"))]
                        result.push_str(&"^".repeat(end_col - start_col));

                        result.push('\n');
                    }
                }
            }
        }

        // Suggestions
        if self.show_suggestions && !diagnostic.suggestions.is_empty() {
            result.push_str("\nsuggestions:\n");
            for suggestion in &diagnostic.suggestions {
                result.push_str(&format!("  - {}", suggestion.message));
                if let Some(replacement) = &suggestion.replacement {
                    result.push_str(&format!(" (replace with '{replacement}')"));
                }
                result.push('\n');
            }
        }

        // Related information
        if !diagnostic.related.is_empty() {
            result.push_str("\nrelated:\n");
            for related in &diagnostic.related {
                result.push_str(&format!(
                    "  - {} at {}\n",
                    related.message, related.location
                ));
            }
        }

        result
    }

    fn format_json(&self, diagnostic: &Diagnostic) -> String {
        #[cfg(feature = "serde")]
        {
            serde_json::to_string_pretty(diagnostic).unwrap()
        }

        #[cfg(not(feature = "serde"))]
        {
            format!("{diagnostic:?}")
        }
    }

    fn format_compact(&self, diagnostic: &Diagnostic) -> String {
        let code = if self.show_code {
            format!("[{}] ", diagnostic.code_string())
        } else {
            String::new()
        };

        format!(
            "{}: {}: {}{}",
            diagnostic.location, diagnostic.severity, code, diagnostic.message
        )
    }
}

impl Default for DiagnosticFormatter {
    fn default() -> Self {
        Self::new(Format::Text)
    }
}

/// Extension trait for formatting diagnostics
pub trait DiagnosticFormat {
    /// Format as human-readable text
    fn to_text(&self) -> String;

    /// Format as JSON
    fn to_json(&self) -> String;

    /// Format as compact single line
    fn to_compact(&self) -> String;
}

impl DiagnosticFormat for Diagnostic {
    fn to_text(&self) -> String {
        DiagnosticFormatter::new(Format::Text).format(self)
    }

    fn to_json(&self) -> String {
        DiagnosticFormatter::new(Format::Json).format(self)
    }

    fn to_compact(&self) -> String {
        DiagnosticFormatter::new(Format::Compact).format(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::builder::DiagnosticBuilder;
    use crate::diagnostics::location::{Position, Span};

    #[test]
    fn test_text_format() {
        let diagnostic = DiagnosticBuilder::unknown_function("foo")
            .with_span(Span::new(Position::new(0, 10), Position::new(0, 13)))
            .with_source_text("let x = foo()")
            .suggest("Did you mean 'for'?", Some("for".to_string()))
            .build();

        let formatter = DiagnosticFormatter::new(Format::Text).with_code(false);
        let output = formatter.format(&diagnostic);

        assert!(output.contains("error: Unknown function 'foo'"));
        assert!(output.contains("let x = foo()"));
        assert!(output.contains("^^^"));
        assert!(output.contains("Did you mean 'for'?"));
    }

    #[test]
    fn test_compact_format() {
        let diagnostic = DiagnosticBuilder::unknown_function("foo")
            .with_span(Span::new(Position::new(5, 10), Position::new(5, 13)))
            .build();

        let formatter = DiagnosticFormatter::new(Format::Compact);
        let output = formatter.format(&diagnostic);

        assert!(output.contains("6:11-14"));
        assert!(output.contains("error"));
        assert!(output.contains("Unknown function 'foo'"));
    }

    #[test]
    fn test_multiple_diagnostics() {
        let diagnostics = vec![
            DiagnosticBuilder::unknown_function("foo").build(),
            DiagnosticBuilder::type_mismatch("String", "Integer").build(),
        ];

        let formatter = DiagnosticFormatter::new(Format::Compact);
        let output = formatter.format_all(&diagnostics);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
