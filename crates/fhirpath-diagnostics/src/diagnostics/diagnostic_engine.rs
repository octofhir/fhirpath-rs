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

//! Comprehensive diagnostic engine for FHIRPath analysis

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use crate::location::SourceLocation;
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during diagnostic formatting
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Unknown formatter: {name}")]
    UnknownFormatter { name: String },
    #[error("Formatting failed: {message}")]
    FormattingFailed { message: String },
}

/// Trait for formatting diagnostics in different output styles
pub trait DiagnosticFormatter: Send + Sync {
    /// Format diagnostics into a string representation
    fn format(&self, diagnostics: &[Diagnostic], source: &str) -> Result<String, FormatError>;
    
    /// Get the name of this formatter
    fn name(&self) -> &str;
    
    /// Check if this formatter supports color output
    fn supports_color(&self) -> bool {
        false
    }
}

/// Error recovery engine for continuing analysis after errors
pub struct ErrorRecoveryEngine {
    max_error_cascade: usize,
    recovery_strategies: Vec<Box<dyn RecoveryStrategy>>,
}

/// Recovery strategy for handling specific types of errors
pub trait RecoveryStrategy: Send + Sync {
    /// Check if this strategy can recover from the given diagnostic
    fn can_recover(&self, diagnostic: &Diagnostic) -> bool;
    
    /// Attempt to recover from the diagnostic and continue analysis
    fn recover(&self, diagnostic: &Diagnostic) -> RecoveryResult;
}

/// Result of an error recovery attempt
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// Whether analysis should continue
    pub continue_analysis: bool,
    /// Additional diagnostics generated during recovery
    pub additional_diagnostics: Vec<Diagnostic>,
    /// A synthetic type to use for continuing analysis
    pub synthetic_type: Option<String>,
}

/// Suggestion engine for generating helpful error suggestions
pub struct SuggestionEngine {
    // Future expansion: will contain FHIR provider and function registry
    _placeholder: (),
}

impl SuggestionEngine {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
    
    /// Generate suggestions for a diagnostic
    pub fn generate_suggestions(&self, diagnostic: &Diagnostic) -> Vec<crate::diagnostic::Suggestion> {
        // Basic suggestion generation - will be enhanced in later tasks
        match &diagnostic.code {
            DiagnosticCode::UnknownFunction => vec![],
            DiagnosticCode::PropertyNotFound => vec![],
            _ => vec![],
        }
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryEngine {
    pub fn new() -> Self {
        Self {
            max_error_cascade: 10,
            recovery_strategies: Vec::new(),
        }
    }
    
    pub fn with_max_cascade(mut self, max: usize) -> Self {
        self.max_error_cascade = max;
        self
    }
    
    pub fn add_strategy(mut self, strategy: Box<dyn RecoveryStrategy>) -> Self {
        self.recovery_strategies.push(strategy);
        self
    }
    
    /// Attempt to recover from a diagnostic error
    pub fn recover(&self, diagnostic: &Diagnostic) -> Option<RecoveryResult> {
        for strategy in &self.recovery_strategies {
            if strategy.can_recover(diagnostic) {
                return Some(strategy.recover(diagnostic));
            }
        }
        None
    }
}

impl Default for ErrorRecoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Main diagnostic engine that collects, manages, and formats diagnostics
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
    error_recovery: ErrorRecoveryEngine,
    suggestion_engine: SuggestionEngine,
    formatters: HashMap<String, Box<dyn DiagnosticFormatter>>,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_recovery: ErrorRecoveryEngine::new(),
            suggestion_engine: SuggestionEngine::new(),
            formatters: HashMap::new(),
        }
    }
    
    /// Register a diagnostic formatter
    pub fn register_formatter(&mut self, formatter: Box<dyn DiagnosticFormatter>) {
        let name = formatter.name().to_string();
        self.formatters.insert(name, formatter);
    }
    
    /// Add a diagnostic to the collection
    pub fn add_diagnostic(&mut self, mut diagnostic: Diagnostic) {
        // Generate suggestions for the diagnostic
        let suggestions = self.suggestion_engine.generate_suggestions(&diagnostic);
        diagnostic.suggestions.extend(suggestions);
        
        // Attempt error recovery if this is an error
        if diagnostic.is_error() {
            if let Some(recovery) = self.error_recovery.recover(&diagnostic) {
                // Add any additional diagnostics from recovery
                for additional in recovery.additional_diagnostics {
                    self.diagnostics.push(additional);
                }
            }
        }
        
        self.diagnostics.push(diagnostic);
        
        // Sort diagnostics by location and severity
        self.sort_diagnostics();
    }
    
    /// Add an error diagnostic
    pub fn add_error(&mut self, code: DiagnosticCode, message: String, location: SourceLocation) {
        let diagnostic = Diagnostic::new(code, Severity::Error, message, location);
        self.add_diagnostic(diagnostic);
    }
    
    /// Add a warning diagnostic
    pub fn add_warning(&mut self, code: DiagnosticCode, message: String, location: SourceLocation) {
        let diagnostic = Diagnostic::new(code, Severity::Warning, message, location);
        self.add_diagnostic(diagnostic);
    }
    
    /// Add an info diagnostic
    pub fn add_info(&mut self, code: DiagnosticCode, message: String, location: SourceLocation) {
        let diagnostic = Diagnostic::new(code, Severity::Info, message, location);
        self.add_diagnostic(diagnostic);
    }
    
    /// Add a hint diagnostic
    pub fn add_hint(&mut self, code: DiagnosticCode, message: String, location: SourceLocation) {
        let diagnostic = Diagnostic::new(code, Severity::Hint, message, location);
        self.add_diagnostic(diagnostic);
    }
    
    /// Format all diagnostics using the specified formatter
    pub fn format_diagnostics(&self, formatter_name: &str, source: &str) -> Result<String, FormatError> {
        let formatter = self.formatters.get(formatter_name)
            .ok_or_else(|| FormatError::UnknownFormatter { 
                name: formatter_name.to_string() 
            })?;
        
        formatter.format(&self.diagnostics, source)
    }
    
    /// Get all diagnostics
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Check if there are any error-level diagnostics
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }
    
    /// Check if there are any warning-level diagnostics
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_warning())
    }
    
    /// Get count of diagnostics by severity
    pub fn count_by_severity(&self) -> DiagnosticCounts {
        let mut counts = DiagnosticCounts::default();
        for diagnostic in &self.diagnostics {
            match diagnostic.severity {
                Severity::Error => counts.errors += 1,
                Severity::Warning => counts.warnings += 1,
                Severity::Info => counts.infos += 1,
                Severity::Hint => counts.hints += 1,
            }
        }
        counts
    }
    
    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
    
    /// Filter diagnostics by severity
    pub fn filter_by_severity(&self, min_severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity >= min_severity)
            .collect()
    }
    
    /// Get diagnostics grouped by location
    pub fn group_by_location(&self) -> HashMap<SourceLocation, Vec<&Diagnostic>> {
        let mut groups = HashMap::new();
        for diagnostic in &self.diagnostics {
            groups.entry(diagnostic.location.clone())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        groups
    }
    
    /// Sort diagnostics by location and severity
    fn sort_diagnostics(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            // First by location
            match a.location.span.start.cmp(&b.location.span.start) {
                std::cmp::Ordering::Equal => {
                    // Then by severity (errors first)
                    b.severity.cmp(&a.severity)
                }
                other => other,
            }
        });
    }
    
    /// Deduplicate similar diagnostics
    pub fn deduplicate(&mut self) {
        self.diagnostics.dedup_by(|a, b| {
            a.code == b.code && 
            a.location == b.location && 
            a.message == b.message
        });
    }
    
    /// Set the error recovery engine
    pub fn set_error_recovery(&mut self, recovery: ErrorRecoveryEngine) {
        self.error_recovery = recovery;
    }
    
    /// Set the suggestion engine
    pub fn set_suggestion_engine(&mut self, engine: SuggestionEngine) {
        self.suggestion_engine = engine;
    }
    
    /// Get list of available formatter names
    pub fn available_formatters(&self) -> Vec<&str> {
        self.formatters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Counts of diagnostics by severity
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCounts {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub hints: usize,
}

impl DiagnosticCounts {
    pub fn total(&self) -> usize {
        self.errors + self.warnings + self.infos + self.hints
    }
    
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

impl fmt::Display for DiagnosticCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "no diagnostics")
        } else {
            let mut parts = Vec::new();
            if self.errors > 0 {
                parts.push(format!("{} error{}", self.errors, if self.errors == 1 { "" } else { "s" }));
            }
            if self.warnings > 0 {
                parts.push(format!("{} warning{}", self.warnings, if self.warnings == 1 { "" } else { "s" }));
            }
            if self.infos > 0 {
                parts.push(format!("{} info{}", self.infos, if self.infos == 1 { "" } else { "s" }));
            }
            if self.hints > 0 {
                parts.push(format!("{} hint{}", self.hints, if self.hints == 1 { "" } else { "s" }));
            }
            write!(f, "{}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::{Position, Span};
    
    #[test]
    fn test_diagnostic_engine_creation() {
        let engine = DiagnosticEngine::new();
        assert!(engine.get_diagnostics().is_empty());
        assert!(!engine.has_errors());
        assert!(!engine.has_warnings());
    }
    
    #[test]
    fn test_add_diagnostics() {
        let mut engine = DiagnosticEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 3)),
            source_text: Some("foo".to_string()),
            file_path: None,
        };
        
        engine.add_error(
            DiagnosticCode::UnknownFunction,
            "Unknown function 'foo'".to_string(),
            location.clone(),
        );
        
        engine.add_warning(
            DiagnosticCode::PropertyNotFound,
            "Property might not exist".to_string(),
            location,
        );
        
        assert_eq!(engine.get_diagnostics().len(), 2);
        assert!(engine.has_errors());
        assert!(engine.has_warnings());
        
        let counts = engine.count_by_severity();
        assert_eq!(counts.errors, 1);
        assert_eq!(counts.warnings, 1);
        assert_eq!(counts.total(), 2);
    }
    
    #[test]
    fn test_filter_by_severity() {
        let mut engine = DiagnosticEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 3)),
            source_text: Some("foo".to_string()),
            file_path: None,
        };
        
        engine.add_hint(DiagnosticCode::Custom("H001".to_string()), "Hint".to_string(), location.clone());
        engine.add_info(DiagnosticCode::Custom("I001".to_string()), "Info".to_string(), location.clone());
        engine.add_warning(DiagnosticCode::PropertyNotFound, "Warning".to_string(), location.clone());
        engine.add_error(DiagnosticCode::UnknownFunction, "Error".to_string(), location);
        
        let errors_only = engine.filter_by_severity(Severity::Error);
        assert_eq!(errors_only.len(), 1);
        
        let warnings_and_above = engine.filter_by_severity(Severity::Warning);
        assert_eq!(warnings_and_above.len(), 2);
    }
    
    #[test]
    fn test_clear_diagnostics() {
        let mut engine = DiagnosticEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 3)),
            source_text: Some("foo".to_string()),
            file_path: None,
        };
        
        engine.add_error(
            DiagnosticCode::UnknownFunction,
            "Error".to_string(),
            location,
        );
        
        assert_eq!(engine.get_diagnostics().len(), 1);
        
        engine.clear();
        assert!(engine.get_diagnostics().is_empty());
        assert!(!engine.has_errors());
    }
    
    #[test]
    fn test_diagnostic_counts_display() {
        let counts = DiagnosticCounts {
            errors: 2,
            warnings: 1,
            infos: 0,
            hints: 3,
        };
        
        let display = counts.to_string();
        assert!(display.contains("2 errors"));
        assert!(display.contains("1 warning"));
        assert!(display.contains("3 hints"));
        assert!(!display.contains("info"));
    }
    
    #[test]
    fn test_deduplicate() {
        let mut engine = DiagnosticEngine::new();
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 3)),
            source_text: Some("foo".to_string()),
            file_path: None,
        };
        
        // Add the same diagnostic twice
        engine.add_error(
            DiagnosticCode::UnknownFunction,
            "Unknown function 'foo'".to_string(),
            location.clone(),
        );
        engine.add_error(
            DiagnosticCode::UnknownFunction,
            "Unknown function 'foo'".to_string(),
            location,
        );
        
        assert_eq!(engine.get_diagnostics().len(), 2);
        
        engine.deduplicate();
        assert_eq!(engine.get_diagnostics().len(), 1);
    }
}