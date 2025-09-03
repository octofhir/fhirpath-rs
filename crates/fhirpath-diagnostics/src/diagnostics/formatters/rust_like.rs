// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Rust-like diagnostic formatter that produces compiler-style error messages

use crate::diagnostic::{Diagnostic, Severity, SuggestionType};
use crate::diagnostics::diagnostic_engine::{DiagnosticFormatter, FormatError};

/// Configuration for the Rust-like formatter
#[derive(Debug, Clone)]
pub struct RustLikeFormatterConfig {
    /// Whether to use colored output
    pub use_colors: bool,
    /// Whether to show source code snippets
    pub show_source: bool,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Maximum number of suggestions to show
    pub max_suggestions: usize,
    /// Maximum number of context lines to show
    pub max_context_lines: usize,
    /// Whether to show help URLs
    pub show_help_urls: bool,
    /// Whether to show related information
    pub show_related: bool,
}

impl Default for RustLikeFormatterConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            show_source: true,
            show_line_numbers: true,
            max_suggestions: 3,
            max_context_lines: 3,
            show_help_urls: true,
            show_related: true,
        }
    }
}

/// Rust-like diagnostic formatter that produces output similar to rustc
pub struct RustLikeDiagnosticFormatter {
    config: RustLikeFormatterConfig,
}

impl RustLikeDiagnosticFormatter {
    /// Create a new Rust-like formatter with default config
    pub fn new() -> Self {
        Self {
            config: RustLikeFormatterConfig::default(),
        }
    }
    
    /// Create a new Rust-like formatter with custom config
    pub fn with_config(config: RustLikeFormatterConfig) -> Self {
        Self { config }
    }
    
    /// Format a single diagnostic
    fn format_diagnostic(&self, diagnostic: &Diagnostic, source: &str) -> String {
        let mut output = String::new();
        
        // Main error line: error[E001]: message
        let severity_color = self.get_severity_color(diagnostic.severity);
        let severity_name = self.get_severity_name(diagnostic.severity);
        let error_code = diagnostic.code_string();
        
        if self.config.use_colors && cfg!(feature = "terminal") {
            #[cfg(feature = "terminal")]
            {
                use colored::Colorize;
                output.push_str(&format!(
                    "{severity}[{code}]: {message}\n",
                    severity = severity_name.color(severity_color).bold(),
                    code = error_code.bright_white().bold(),
                    message = diagnostic.message.bright_white().bold()
                ));
            }
            #[cfg(not(feature = "terminal"))]
            {
                output.push_str(&format!(
                    "{severity}[{error_code}]: {message}\n",
                    severity = severity_name,
                    message = diagnostic.message
                ));
            }
        } else {
            output.push_str(&format!(
                "{severity}[{error_code}]: {message}\n",
                severity = severity_name,
                message = diagnostic.message
            ));
        }
        
        // Location line: --> file:line:column
        let location_str = if let Some(file_path) = &diagnostic.location.file_path {
            format!(
                "{file}:{line}:{col}",
                file = file_path,
                line = diagnostic.location.span.start.line + 1,
                col = diagnostic.location.span.start.column + 1
            )
        } else {
            format!(
                "expression:{line}:{col}",
                line = diagnostic.location.span.start.line + 1,
                col = diagnostic.location.span.start.column + 1
            )
        };
        
        if self.config.use_colors && cfg!(feature = "terminal") {
            #[cfg(feature = "terminal")]
            {
                use colored::Colorize;
                output.push_str(&format!(
                    " {} {}\n",
                    "-->".bright_blue().bold(),
                    location_str.bright_white()
                ));
            }
            #[cfg(not(feature = "terminal"))]
            {
                output.push_str(&format!(" --> {}\n", location_str));
            }
        } else {
            output.push_str(&format!(" --> {}\n", location_str));
        }
        
        // Source code snippet
        if self.config.show_source {
            if let Some(source_text) = &diagnostic.location.source_text {
                output.push_str(&self.format_source_snippet(
                    source_text,
                    diagnostic,
                ));
            } else if !source.is_empty() {
                // Use provided source if diagnostic doesn't have its own
                output.push_str(&self.format_source_snippet(
                    source,
                    diagnostic,
                ));
            }
        }
        
        // Suggestions
        if !diagnostic.suggestions.is_empty() {
            output.push_str(&self.format_suggestions(diagnostic));
        }
        
        // Related information
        if self.config.show_related && !diagnostic.related.is_empty() {
            output.push_str(&self.format_related_information(diagnostic));
        }
        
        // Help URL
        if self.config.show_help_urls {
            if let Some(help_url) = self.get_help_url(diagnostic) {
                output.push_str(&format!("  = help: see {}\n", help_url));
            }
        }
        
        output
    }
    
    /// Format source code snippet with highlighting
    fn format_source_snippet(&self, source_text: &str, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = source_text.lines().collect();
        
        let start_line = diagnostic.location.span.start.line;
        let end_line = diagnostic.location.span.end.line;
        let start_col = diagnostic.location.span.start.column;
        let end_col = diagnostic.location.span.end.column;
        
        // Calculate context range
        let context_start = start_line.saturating_sub(self.config.max_context_lines);
        let context_end = std::cmp::min(end_line + self.config.max_context_lines, lines.len());
        
        // Calculate line number width for alignment
        let line_num_width = if self.config.show_line_numbers {
            (context_end + 1).to_string().len()
        } else {
            0
        };
        
        // Empty line with gutter
        if self.config.show_line_numbers {
            if self.config.use_colors && cfg!(feature = "terminal") {
                #[cfg(feature = "terminal")]
                {
                    use colored::Colorize;
                    output.push_str(&format!(
                        "{:width$} {}\n",
                        "",
                        "|".bright_blue().bold(),
                        width = line_num_width
                    ));
                }
                #[cfg(not(feature = "terminal"))]
                {
                    output.push_str(&format!("{:width$} |\n", "", width = line_num_width));
                }
            } else {
                output.push_str(&format!("{:width$} |\n", "", width = line_num_width));
            }
        }
        
        // Show context and highlighted lines
        for line_idx in context_start..context_end {
            if line_idx >= lines.len() {
                break;
            }
            
            let line_content = lines[line_idx];
            let line_number = line_idx + 1;
            let is_error_line = line_idx >= start_line && line_idx <= end_line;
            
            // Format line number and gutter
            let gutter = if self.config.show_line_numbers {
                if self.config.use_colors && cfg!(feature = "terminal") {
                    #[cfg(feature = "terminal")]
                    {
                        use colored::Colorize;
                        format!(
                            "{:width$} {} ",
                            line_number.to_string().bright_blue().bold(),
                            "|".bright_blue().bold(),
                            width = line_num_width
                        )
                    }
                    #[cfg(not(feature = "terminal"))]
                    {
                        format!("{:width$} | ", line_number, width = line_num_width)
                    }
                } else {
                    format!("{:width$} | ", line_number, width = line_num_width)
                }
            } else {
                "".to_string()
            };
            
            output.push_str(&format!("{}{}\n", gutter, line_content));
            
            // Add underline for error line
            if is_error_line && line_idx == start_line {
                let underline_start = if line_idx == start_line { start_col } else { 0 };
                let underline_end = if line_idx == end_line {
                    std::cmp::min(end_col, line_content.len())
                } else {
                    line_content.len()
                };
                
                let padding = if self.config.show_line_numbers {
                    format!("{:width$} {} ", "", "|", width = line_num_width)
                } else {
                    "".to_string()
                };
                
                let prefix_spaces = " ".repeat(underline_start);
                let underline_chars = "^".repeat(underline_end.saturating_sub(underline_start).max(1));
                
                let underline = if self.config.use_colors && cfg!(feature = "terminal") {
                    #[cfg(feature = "terminal")]
                    {
                        use colored::Colorize;
                        match diagnostic.severity {
                            Severity::Error => underline_chars.bright_red().bold(),
                            Severity::Warning => underline_chars.bright_yellow().bold(),
                            Severity::Info => underline_chars.bright_blue().bold(),
                            Severity::Hint => underline_chars.bright_green().bold(),
                        }.to_string()
                    }
                    #[cfg(not(feature = "terminal"))]
                    {
                        underline_chars
                    }
                } else {
                    underline_chars
                };
                
                output.push_str(&format!("{}{}{}\n", padding, prefix_spaces, underline));
            }
        }
        
        output
    }
    
    /// Format suggestions
    fn format_suggestions(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();
        
        let suggestions_to_show = diagnostic.suggestions
            .iter()
            .take(self.config.max_suggestions);
        
        for suggestion in suggestions_to_show {
            let prefix = match suggestion.suggestion_type {
                SuggestionType::TypoFix => "help: ",
                SuggestionType::AlternativeFunction => "help: ",
                SuggestionType::AlternativeProperty => "help: ",
                SuggestionType::TypeConversion => "note: ",
                SuggestionType::SyntaxImprovement => "help: ",
                SuggestionType::PerformanceOptimization => "note: ",
                SuggestionType::General => "help: ",
            };
            
            if self.config.use_colors && cfg!(feature = "terminal") {
                #[cfg(feature = "terminal")]
                {
                    use colored::Colorize;
                    output.push_str(&format!(
                        "  {} {}\n",
                        "=".bright_blue().bold(),
                        format!("{}{}", prefix, suggestion.message).bright_white()
                    ));
                }
                #[cfg(not(feature = "terminal"))]
                {
                    output.push_str(&format!("  = {}{}\n", prefix, suggestion.message));
                }
            } else {
                output.push_str(&format!("  = {}{}\n", prefix, suggestion.message));
            }
            
            // Show replacement if available
            if let Some(replacement) = &suggestion.replacement {
                output.push_str(&format!(
                    "    {} {}\n",
                    "try:",
                    replacement.new_text
                ));
            }
        }
        
        output
    }
    
    /// Format related information
    fn format_related_information(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();
        
        for related in &diagnostic.related {
            if self.config.use_colors && cfg!(feature = "terminal") {
                #[cfg(feature = "terminal")]
                {
                    use colored::Colorize;
                    output.push_str(&format!(
                        "  {} {}\n",
                        "=".bright_blue().bold(),
                        format!("note: {} at {}", related.message, related.location).bright_white()
                    ));
                }
                #[cfg(not(feature = "terminal"))]
                {
                    output.push_str(&format!("  = note: {} at {}\n", related.message, related.location));
                }
            } else {
                output.push_str(&format!("  = note: {} at {}\n", related.message, related.location));
            }
        }
        
        output
    }
    
    /// Get severity color for terminal output
    #[cfg(feature = "terminal")]
    fn get_severity_color(&self, severity: Severity) -> colored::Color {
        match severity {
            Severity::Error => colored::Color::BrightRed,
            Severity::Warning => colored::Color::BrightYellow,
            Severity::Info => colored::Color::BrightBlue,
            Severity::Hint => colored::Color::BrightGreen,
        }
    }
    
    #[cfg(not(feature = "terminal"))]
    fn get_severity_color(&self, _severity: Severity) -> &'static str {
        ""
    }
    
    /// Get severity name
    fn get_severity_name(&self, severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }
    
    /// Get help URL for a diagnostic
    fn get_help_url(&self, diagnostic: &Diagnostic) -> Option<String> {
        // TODO: Integrate with structured error codes when available
        Some(format!(
            "https://docs.octofhir.com/fhirpath/errors/{}",
            diagnostic.code_string()
        ))
    }
}

impl Default for RustLikeDiagnosticFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticFormatter for RustLikeDiagnosticFormatter {
    fn format(&self, diagnostics: &[Diagnostic], source: &str) -> Result<String, FormatError> {
        let mut output = String::new();
        
        for (i, diagnostic) in diagnostics.iter().enumerate() {
            if i > 0 {
                output.push('\n'); // Add spacing between diagnostics
            }
            output.push_str(&self.format_diagnostic(diagnostic, source));
        }
        
        Ok(output)
    }
    
    fn name(&self) -> &str {
        "rust-like"
    }
    
    fn supports_color(&self) -> bool {
        self.config.use_colors && cfg!(feature = "terminal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{DiagnosticCode, Suggestion, SuggestionType, TextEdit};
    use crate::location::{Position, Span, SourceLocation};
    
    #[test]
    fn test_rust_like_formatter_basic() {
        let formatter = RustLikeDiagnosticFormatter::new();
        
        let location = SourceLocation {
            span: Span::new(Position::new(0, 8), Position::new(0, 19)),
            source_text: Some("Patient.invalidProp.name".to_string()),
            file_path: Some("test.fhirpath".to_string()),
        };
        
        let diagnostic = Diagnostic::new(
            DiagnosticCode::PropertyNotFound,
            Severity::Error,
            "property 'invalidProp' does not exist on type 'Patient'".to_string(),
            location,
        );
        
        let result = formatter.format(&[diagnostic], "").unwrap();
        
        assert!(result.contains("error[E201]:"));
        assert!(result.contains("property 'invalidProp' does not exist"));
        assert!(result.contains("test.fhirpath:1:9"));
        assert!(result.contains("Patient.invalidProp.name"));
        assert!(result.contains("^^^^^^^^^^^"));
    }
    
    #[test]
    fn test_rust_like_formatter_with_suggestions() {
        let formatter = RustLikeDiagnosticFormatter::new();
        
        let location = SourceLocation {
            span: Span::new(Position::new(0, 8), Position::new(0, 19)),
            source_text: Some("Patient.invalidProp.name".to_string()),
            file_path: None,
        };
        
        let suggestion = Suggestion::typo_fix(
            "invalidProp",
            "identifier",
            location.clone(),
            0.85,
        );
        
        let diagnostic = Diagnostic::new(
            DiagnosticCode::PropertyNotFound,
            Severity::Error,
            "property 'invalidProp' does not exist on type 'Patient'".to_string(),
            location,
        ).with_suggestion(suggestion);
        
        let result = formatter.format(&[diagnostic], "").unwrap();
        
        assert!(result.contains("help: Did you mean 'identifier'?"));
        assert!(result.contains("try: identifier"));
    }
    
    #[test]
    fn test_formatter_config() {
        let config = RustLikeFormatterConfig {
            use_colors: false,
            show_line_numbers: false,
            max_suggestions: 1,
            ..Default::default()
        };
        
        let formatter = RustLikeDiagnosticFormatter::with_config(config);
        
        assert_eq!(formatter.name(), "rust-like");
        assert!(!formatter.supports_color());
    }
}