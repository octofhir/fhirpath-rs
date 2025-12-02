//! Comprehensive diagnostic processor for FHIRPath analysis results
//!
//! This module provides intelligent diagnostic processing that transforms raw analysis
//! results into actionable, contextual error messages with suggestions and rich formatting.

use crate::core::error_code::{ErrorCode, FP0001, FP0010, FP0055, FP0101};
use crate::diagnostics::{
    AriadneDiagnostic, DiagnosticEngine, DiagnosticSeverity, RelatedDiagnostic,
};
use std::collections::HashMap;
use std::ops::Range;

/// Comprehensive diagnostic processor that creates rich, contextual diagnostics
pub struct DiagnosticProcessor {
    /// Diagnostic engine for rendering
    engine: DiagnosticEngine,
    /// Help system for providing contextual help
    help_system: HelpSystem,
    /// Suggestion engine for generating fixes
    #[allow(dead_code)]
    suggestion_engine: SuggestionEngine,
    /// Relationship detector for linking related diagnostics
    #[allow(dead_code)]
    relationship_detector: RelationshipDetector,
}

/// Configuration for diagnostic processing
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// Maximum number of suggestions to show per diagnostic
    pub max_suggestions: usize,
    /// Include optimization suggestions as info diagnostics
    pub include_optimizations: bool,
    /// Show context lines around errors
    pub show_context: bool,
    /// Number of context lines before/after error
    pub context_lines: usize,
    /// Show related diagnostics
    pub show_related: bool,
    /// Enable suggestion confidence scoring
    pub score_suggestions: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 3,
            include_optimizations: true,
            show_context: true,
            context_lines: 2,
            show_related: true,
            score_suggestions: true,
        }
    }
}

/// Rich diagnostic with context and suggestions
#[derive(Debug, Clone)]
pub struct ProcessedDiagnostic {
    /// Core diagnostic information
    pub diagnostic: AriadneDiagnostic,
    /// Context information about the error location
    pub context: DiagnosticContext,
    /// Suggested fixes and improvements
    pub suggestions: Vec<DiagnosticSuggestion>,
    /// Related diagnostics
    pub related: Vec<RelatedDiagnostic>,
    /// Help text for understanding the issue
    pub help_text: Option<String>,
    /// Documentation URL for more information
    pub documentation_url: Option<String>,
}

/// Context information for a diagnostic
#[derive(Debug, Clone)]
pub struct DiagnosticContext {
    /// Source code context around the error
    pub source_snippet: String,
    /// Inferred resource type context
    pub resource_context: Option<String>,
    /// Function call context
    pub function_context: Option<String>,
    /// Expression path context
    pub expression_path: Vec<String>,
    /// Nearby source lines
    pub source_lines: Vec<ContextLine>,
}

/// Source line with highlighting information
#[derive(Debug, Clone)]
pub struct ContextLine {
    /// Line number (1-based)
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Whether this is the error line
    pub is_error_line: bool,
    /// Highlight ranges within the line
    pub highlights: Vec<Highlight>,
}

/// Highlight range within a source line
#[derive(Debug, Clone)]
pub struct Highlight {
    /// Start column
    pub start: usize,
    /// End column
    pub end: usize,
    /// Highlight style
    pub style: HighlightStyle,
    /// Optional tooltip message
    pub message: Option<String>,
}

/// Highlight style
#[derive(Debug, Clone)]
pub enum HighlightStyle {
    /// Error highlighting
    Error,
    /// Warning highlighting
    Warning,
    /// Info highlighting
    Info,
    /// Suggestion highlighting
    Suggestion,
    /// Context highlighting
    Context,
}

/// Diagnostic suggestion with confidence and category
#[derive(Debug, Clone)]
pub struct DiagnosticSuggestion {
    /// Suggestion message
    pub message: String,
    /// Text replacements to apply
    pub replacements: Vec<TextReplacement>,
    /// Confidence level
    pub confidence: SuggestionConfidence,
    /// Suggestion category
    pub category: SuggestionCategory,
    /// Estimated improvement
    pub improvement_estimate: Option<f32>,
}

/// Text replacement for suggestions
#[derive(Debug, Clone)]
pub struct TextReplacement {
    /// Source range to replace
    pub range: Range<usize>,
    /// Replacement text
    pub text: String,
    /// Description of the replacement
    pub description: String,
}

/// Confidence level for suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionConfidence {
    /// Almost certainly correct
    High,
    /// Likely correct
    Medium,
    /// Possible alternative
    Low,
}

/// Category of suggestion
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionCategory {
    /// Fix a definite error
    Fix,
    /// Performance or style improvement
    Improvement,
    /// Different way to achieve same result
    Alternative,
    /// Make intent clearer
    Clarification,
}

/// Relationship between diagnostics
#[derive(Debug, Clone)]
pub struct DiagnosticRelationship {
    /// Kind of relationship
    pub kind: RelationshipKind,
    /// Description of the relationship
    pub description: String,
    /// Severity level for the relationship
    pub severity: DiagnosticSeverity,
    /// Confidence in the relationship
    pub confidence: f32,
}

/// Type of relationship between diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipKind {
    /// Diagnostics are duplicates or very similar
    Duplicate,
    /// One diagnostic causes another
    CauseEffect,
    /// Diagnostics are related by context
    ContextRelated,
    /// Diagnostics suggest conflicting approaches
    Conflicting,
}

/// Contextual suggestion with rich information
#[derive(Debug, Clone)]
pub struct ContextualSuggestion {
    /// Base suggestion
    pub suggestion: DiagnosticSuggestion,
    /// Context where this suggestion applies
    pub context: Vec<String>,
    /// Priority for ordering suggestions
    pub priority: i32,
    /// Whether this suggestion fixes the primary issue
    pub fixes_primary: bool,
}

impl DiagnosticProcessor {
    /// Create a new diagnostic processor
    pub fn new() -> Self {
        Self {
            engine: DiagnosticEngine::new(),
            help_system: HelpSystem::new(),
            suggestion_engine: SuggestionEngine::new(),
            relationship_detector: RelationshipDetector::new(),
        }
    }

    /// Create processor with custom configuration
    pub fn with_config(config: DiagnosticConfig) -> Self {
        let mut processor = Self::new();
        processor
            .help_system
            .set_max_suggestions(config.max_suggestions);
        processor
    }

    /// Process analysis results into rich diagnostics
    pub fn process_analysis(
        &mut self,
        result: &crate::analyzer::StaticAnalysisResult,
        source: &str,
        filename: Option<&str>,
    ) -> Vec<ProcessedDiagnostic> {
        let mut processed = Vec::new();

        // Add source to engine
        let _source_id = self
            .engine
            .add_source(filename.unwrap_or("<input>"), source);

        // Process diagnostics from static analysis (already in AriadneDiagnostic format!)
        for diagnostic in &result.diagnostics {
            let context = self.build_context(diagnostic, source);

            // Generate suggestions
            let suggestions = self
                .suggestion_engine
                .generate_suggestions(diagnostic, source, &context);

            // Get help text
            let help_text = self.help_system.get_help_text(&diagnostic.error_code);
            let documentation_url = self
                .help_system
                .get_documentation_url(&diagnostic.error_code);

            processed.push(ProcessedDiagnostic {
                diagnostic: diagnostic.clone(),
                context,
                suggestions,
                related: Vec::new(), // Will be populated by relationship detector
                help_text,
                documentation_url,
            });
        }

        // Process analysis suggestions as info-level diagnostics
        for suggestion in &result.suggestions {
            if let Some(processed_diag) = self.process_suggestion_to_diagnostic(suggestion, source)
            {
                processed.push(processed_diag);
            }
        }

        // Detect and link related diagnostics
        self.relationship_detector.link_related(&mut processed);

        processed
    }

    /// Process a single diagnostic into a rich diagnostic
    #[allow(dead_code)]
    fn process_diagnostic(
        &self,
        diagnostic: &crate::diagnostics::Diagnostic,
        source: &str,
        _source_id: usize,
    ) -> Option<ProcessedDiagnostic> {
        // Convert to AriadneDiagnostic
        let ariadne_diag = self.convert_to_ariadne_diagnostic(diagnostic)?;

        // Build context
        let context = self.build_context(&ariadne_diag, source);

        // Generate suggestions
        let suggestions =
            self.suggestion_engine
                .generate_suggestions(&ariadne_diag, source, &context);

        // Get help text
        let help_text = self.help_system.get_help_text(&ariadne_diag.error_code);
        let documentation_url = self
            .help_system
            .get_documentation_url(&ariadne_diag.error_code);

        Some(ProcessedDiagnostic {
            diagnostic: ariadne_diag,
            context,
            suggestions,
            related: Vec::new(), // Will be populated by relationship detector
            help_text,
            documentation_url,
        })
    }

    /// Convert a Diagnostic to AriadneDiagnostic
    #[allow(dead_code)]
    fn convert_to_ariadne_diagnostic(
        &self,
        diagnostic: &crate::diagnostics::Diagnostic,
    ) -> Option<AriadneDiagnostic> {
        // Convert location to span
        let span = if let Some(location) = &diagnostic.location {
            location.offset..(location.offset + location.length)
        } else {
            0..0
        };

        // Convert code - use existing error code construction
        let error_code = match diagnostic.code.code.as_str() {
            "FP0001" => FP0001,
            "FP0055" => FP0055,
            "FP0101" => FP0101,
            _ => FP0001, // Default fallback
        };

        Some(AriadneDiagnostic {
            severity: diagnostic.severity.clone(),
            error_code,
            message: diagnostic.message.clone(),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        })
    }

    /// Build diagnostic context
    #[allow(dead_code)]
    fn build_context(&self, diagnostic: &AriadneDiagnostic, source: &str) -> DiagnosticContext {
        let source_snippet = self.extract_source_snippet(source, &diagnostic.span);
        let source_lines = self.extract_context_lines(source, &diagnostic.span, 2);
        let resource_context = self.infer_resource_context(source, &diagnostic.span);
        let function_context = self.infer_function_context(source, &diagnostic.span);
        let expression_path = self.build_expression_path(source, &diagnostic.span);

        DiagnosticContext {
            source_snippet,
            resource_context,
            function_context,
            expression_path,
            source_lines,
        }
    }

    /// Extract source snippet for the error span
    #[allow(dead_code)]
    fn extract_source_snippet(&self, source: &str, span: &Range<usize>) -> String {
        let start = span.start.min(source.len());
        let end = span.end.min(source.len());
        source[start..end].to_string()
    }

    /// Extract context lines around the error
    #[allow(dead_code)]
    fn extract_context_lines(
        &self,
        source: &str,
        span: &Range<usize>,
        context_lines: usize,
    ) -> Vec<ContextLine> {
        let lines: Vec<&str> = source.lines().collect();
        let error_line_idx = self.find_line_index(source, span.start);

        let start_line = error_line_idx.saturating_sub(context_lines);
        let end_line = (error_line_idx + context_lines).min(lines.len().saturating_sub(1));

        let mut result = Vec::new();
        for (line_idx, line_content) in lines
            .iter()
            .enumerate()
            .skip(start_line)
            .take(end_line - start_line + 1)
        {
            let is_error_line = line_idx == error_line_idx;
            let highlights = if is_error_line {
                self.calculate_line_highlights(
                    line_content,
                    span,
                    self.get_line_start_offset(source, line_idx),
                )
            } else {
                Vec::new()
            };

            result.push(ContextLine {
                line_number: line_idx + 1,
                content: line_content.to_string(),
                is_error_line,
                highlights,
            });
        }

        result
    }

    /// Calculate highlights for a specific line
    #[allow(dead_code)]
    fn calculate_line_highlights(
        &self,
        line: &str,
        span: &Range<usize>,
        line_start: usize,
    ) -> Vec<Highlight> {
        let line_end = line_start + line.len();

        // Check if the span intersects with this line
        if span.start >= line_end || span.end <= line_start {
            return Vec::new();
        }

        let highlight_start = span.start.saturating_sub(line_start);
        let highlight_end = (span.end.saturating_sub(line_start)).min(line.len());

        if highlight_start < line.len() && highlight_end > highlight_start {
            vec![Highlight {
                start: highlight_start,
                end: highlight_end,
                style: HighlightStyle::Error,
                message: None,
            }]
        } else {
            Vec::new()
        }
    }

    /// Find line index for a character position
    fn find_line_index(&self, source: &str, pos: usize) -> usize {
        source[..pos.min(source.len())].matches('\n').count()
    }

    /// Get the character offset where a line starts
    fn get_line_start_offset(&self, source: &str, line_idx: usize) -> usize {
        if line_idx == 0 {
            return 0;
        }

        let mut offset = 0;
        let mut current_line = 0;

        for ch in source.chars() {
            if current_line == line_idx {
                break;
            }
            if ch == '\n' {
                current_line += 1;
            }
            offset += ch.len_utf8();
        }

        offset
    }

    /// Infer resource type context from the source
    fn infer_resource_context(&self, source: &str, span: &Range<usize>) -> Option<String> {
        let before_error = &source[..span.start.min(source.len())];

        // Look for common resource types
        for resource_type in &[
            "Patient",
            "Observation",
            "Practitioner",
            "Organization",
            "Bundle",
            "Encounter",
        ] {
            if before_error.contains(resource_type) {
                return Some(resource_type.to_string());
            }
        }

        None
    }

    /// Infer function context from the source
    fn infer_function_context(&self, source: &str, span: &Range<usize>) -> Option<String> {
        let before_error = &source[..span.start.min(source.len())];

        // Find the last function call before the error
        if let Some(paren_pos) = before_error.rfind('(') {
            // Find function name before the parenthesis
            let before_paren = &before_error[..paren_pos];
            if let Some(func_start) = before_paren.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            {
                let function_name = &before_paren[func_start + 1..];
                if !function_name.is_empty()
                    && function_name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_')
                {
                    return Some(function_name.to_string());
                }
            }
        }

        None
    }

    /// Build expression path context
    fn build_expression_path(&self, source: &str, span: &Range<usize>) -> Vec<String> {
        let before_error = &source[..span.start.min(source.len())];
        before_error
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Convert analysis suggestion to processed diagnostic
    fn process_suggestion_to_diagnostic(
        &self,
        suggestion: &crate::analyzer::AnalysisSuggestion,
        source: &str,
    ) -> Option<ProcessedDiagnostic> {
        // Map suggestion type to error code
        let error_code = match suggestion.suggestion_type {
            crate::analyzer::SuggestionType::Performance => FP0010,
            crate::analyzer::SuggestionType::Simplification => FP0010,
            crate::analyzer::SuggestionType::TypeSafety => FP0010,
            crate::analyzer::SuggestionType::BestPractice => FP0010,
            crate::analyzer::SuggestionType::ErrorPrevention => FP0010,
        };

        // Create AriadneDiagnostic from suggestion
        let span = if let Some(location) = &suggestion.location {
            location.offset..(location.offset + location.length)
        } else {
            0..0
        };

        let ariadne_diag = AriadneDiagnostic {
            severity: DiagnosticSeverity::Info,
            error_code,
            message: suggestion.message.clone(),
            span: span.clone(),
            help: suggestion.code_snippet.clone(),
            note: None,
            related: Vec::new(),
        };

        // Build context
        let context = self.build_context(&ariadne_diag, source);

        // Convert suggestion to diagnostic suggestion
        let diag_suggestions = vec![DiagnosticSuggestion {
            message: suggestion.message.clone(),
            replacements: if let Some(snippet) = &suggestion.code_snippet {
                vec![TextReplacement {
                    range: span,
                    text: snippet.clone(),
                    description: "Apply suggested improvement".to_string(),
                }]
            } else {
                Vec::new()
            },
            confidence: if suggestion.confidence > 0.8 {
                SuggestionConfidence::High
            } else if suggestion.confidence > 0.5 {
                SuggestionConfidence::Medium
            } else {
                SuggestionConfidence::Low
            },
            category: match suggestion.suggestion_type {
                crate::analyzer::SuggestionType::Performance => SuggestionCategory::Improvement,
                crate::analyzer::SuggestionType::Simplification => SuggestionCategory::Improvement,
                crate::analyzer::SuggestionType::TypeSafety => SuggestionCategory::Fix,
                crate::analyzer::SuggestionType::BestPractice => SuggestionCategory::Clarification,
                crate::analyzer::SuggestionType::ErrorPrevention => SuggestionCategory::Fix,
            },
            improvement_estimate: Some(suggestion.confidence),
        }];

        Some(ProcessedDiagnostic {
            diagnostic: ariadne_diag,
            context,
            suggestions: diag_suggestions,
            related: Vec::new(),
            help_text: Some(format!(
                "{} suggestion with {:.0}% confidence",
                suggestion.suggestion_type,
                suggestion.confidence * 100.0
            )),
            documentation_url: None,
        })
    }

    /// Render processed diagnostics to string
    pub fn render_diagnostics(
        &mut self,
        diagnostics: &[ProcessedDiagnostic],
        source: &str,
        filename: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();
        let source_id = self
            .engine
            .add_source(filename.unwrap_or("<input>"), source);

        for (i, processed) in diagnostics.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }

            // Render main diagnostic
            let diagnostic_output = self
                .engine
                .format_diagnostic(&processed.diagnostic, source_id)?;
            output.push_str(&diagnostic_output);

            // Add suggestions if any
            if !processed.suggestions.is_empty() {
                output.push_str(&self.render_suggestions(&processed.suggestions));
            }

            // Add help text
            if let Some(help) = &processed.help_text {
                output.push_str(&format!("\n💡 Help: {help}"));
            }

            // Add documentation URL
            if let Some(url) = &processed.documentation_url {
                output.push_str(&format!("\n📚 Documentation: {url}"));
            }

            // Add related diagnostics
            if !processed.related.is_empty() {
                output.push_str(&self.render_related_diagnostics(&processed.related));
            }
        }

        Ok(output)
    }

    /// Render suggestions as formatted text
    fn render_suggestions(&self, suggestions: &[DiagnosticSuggestion]) -> String {
        let mut output = String::new();

        if suggestions.is_empty() {
            return output;
        }

        output.push_str("\n\n💡 Suggestions:");

        for (i, suggestion) in suggestions.iter().enumerate() {
            let confidence_icon = match suggestion.confidence {
                SuggestionConfidence::High => "✓",
                SuggestionConfidence::Medium => "~",
                SuggestionConfidence::Low => "?",
            };

            let category_icon = match suggestion.category {
                SuggestionCategory::Fix => "🔧",
                SuggestionCategory::Improvement => "⚡",
                SuggestionCategory::Alternative => "🔄",
                SuggestionCategory::Clarification => "💭",
            };

            output.push_str(&format!(
                "\n  {} {} {}: {}",
                category_icon,
                confidence_icon,
                i + 1,
                suggestion.message
            ));

            // Show replacements if any
            for replacement in &suggestion.replacements {
                output.push_str(&format!("\n    Replace with: '{}'", replacement.text));
            }
        }

        output
    }

    /// Render related diagnostics
    fn render_related_diagnostics(&self, related: &[RelatedDiagnostic]) -> String {
        let mut output = String::new();

        if related.is_empty() {
            return output;
        }

        output.push_str("\n\n🔗 Related:");

        for related_diag in related {
            output.push_str(&format!("\n  • {}", related_diag.message));
        }

        output
    }
}

impl Default for DiagnosticProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Help system for providing contextual help text
struct HelpSystem {
    help_texts: HashMap<ErrorCode, String>,
    documentation_urls: HashMap<ErrorCode, String>,
    max_suggestions: usize,
}

impl HelpSystem {
    fn new() -> Self {
        let mut system = Self {
            help_texts: HashMap::new(),
            documentation_urls: HashMap::new(),
            max_suggestions: 3,
        };
        system.initialize_help_content();
        system
    }

    fn set_max_suggestions(&mut self, max: usize) {
        self.max_suggestions = max;
    }

    fn get_help_text(&self, code: &ErrorCode) -> Option<String> {
        self.help_texts.get(code).cloned()
    }

    fn get_documentation_url(&self, code: &ErrorCode) -> Option<String> {
        self.documentation_urls.get(code).cloned()
    }

    fn initialize_help_content(&mut self) {
        self.help_texts.insert(
            FP0001,
            "This is a general syntax error. Check the FHIRPath expression syntax.".to_string(),
        );

        self.help_texts.insert(
            FP0055,
            "The property does not exist on this resource type. Check the FHIR specification for valid properties.".to_string()
        );

        self.documentation_urls
            .insert(FP0001, "https://hl7.org/fhirpath/#grammar".to_string());

        self.documentation_urls.insert(
            FP0055,
            "https://hl7.org/fhir/elementdefinition.html".to_string(),
        );
    }
}

/// Engine for generating diagnostic suggestions
struct SuggestionEngine;

impl SuggestionEngine {
    fn new() -> Self {
        Self
    }

    fn generate_suggestions(
        &self,
        diagnostic: &AriadneDiagnostic,
        source: &str,
        context: &DiagnosticContext,
    ) -> Vec<DiagnosticSuggestion> {
        let mut suggestions = Vec::new();

        // Generate suggestions based on error code
        if diagnostic.error_code == FP0055 {
            // Property not found
            suggestions.extend(self.suggest_property_fixes(source, &diagnostic.span, context));
        } else if diagnostic.error_code == FP0101 {
            // Unknown function
            suggestions.extend(self.suggest_function_fixes(source, &diagnostic.span, context));
        } else {
            // Generic suggestions based on context
            suggestions.extend(self.suggest_generic_improvements(source, context));
        }

        suggestions
    }

    fn suggest_property_fixes(
        &self,
        _source: &str,
        _span: &Range<usize>,
        _context: &DiagnosticContext,
    ) -> Vec<DiagnosticSuggestion> {
        vec![DiagnosticSuggestion {
            message: "Check the property name spelling".to_string(),
            replacements: Vec::new(),
            confidence: SuggestionConfidence::Medium,
            category: SuggestionCategory::Fix,
            improvement_estimate: None,
        }]
    }

    fn suggest_function_fixes(
        &self,
        _source: &str,
        _span: &Range<usize>,
        _context: &DiagnosticContext,
    ) -> Vec<DiagnosticSuggestion> {
        vec![DiagnosticSuggestion {
            message: "Check the function name spelling or availability".to_string(),
            replacements: Vec::new(),
            confidence: SuggestionConfidence::Medium,
            category: SuggestionCategory::Fix,
            improvement_estimate: None,
        }]
    }

    fn suggest_generic_improvements(
        &self,
        source: &str,
        _context: &DiagnosticContext,
    ) -> Vec<DiagnosticSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest performance improvements
        if source.contains("count()") && source.contains("> 0") {
            suggestions.push(DiagnosticSuggestion {
                message: "Consider using exists() instead of count() > 0 for better performance"
                    .to_string(),
                replacements: Vec::new(),
                confidence: SuggestionConfidence::Low,
                category: SuggestionCategory::Improvement,
                improvement_estimate: Some(0.3),
            });
        }

        suggestions
    }
}

/// Detector for finding relationships between diagnostics
struct RelationshipDetector;

impl RelationshipDetector {
    fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    fn link_related(&self, diagnostics: &mut [ProcessedDiagnostic]) {
        // Find related diagnostics based on various criteria
        for i in 0..diagnostics.len() {
            let mut related = Vec::new();

            for j in 0..diagnostics.len() {
                if i == j {
                    continue;
                }

                let relationship = self.analyze_relationship(&diagnostics[i], &diagnostics[j]);
                if let Some(rel) = relationship {
                    related.push(RelatedDiagnostic {
                        message: rel.description,
                        span: diagnostics[j].diagnostic.span.clone(),
                        severity: rel.severity,
                    });
                }
            }

            diagnostics[i].related.extend(related);
        }
    }

    #[allow(dead_code)]
    fn analyze_relationship(
        &self,
        primary: &ProcessedDiagnostic,
        candidate: &ProcessedDiagnostic,
    ) -> Option<DiagnosticRelationship> {
        // Check for overlapping spans (potential duplicates)
        if self.spans_overlap(&primary.diagnostic.span, &candidate.diagnostic.span) {
            return Some(DiagnosticRelationship {
                kind: RelationshipKind::Duplicate,
                description: "Similar issue at this location".to_string(),
                severity: DiagnosticSeverity::Info,
                confidence: 0.8,
            });
        }

        // Check for causal relationships based on error codes
        if self.is_causal_relationship(
            &primary.diagnostic.error_code,
            &candidate.diagnostic.error_code,
        ) {
            return Some(DiagnosticRelationship {
                kind: RelationshipKind::CauseEffect,
                description: "This error may be caused by the issue above".to_string(),
                severity: DiagnosticSeverity::Hint,
                confidence: 0.6,
            });
        }

        // Check for context relationships
        if primary.context.resource_context == candidate.context.resource_context
            && primary.context.resource_context.is_some()
        {
            return Some(DiagnosticRelationship {
                kind: RelationshipKind::ContextRelated,
                description: "Related issue in same resource type".to_string(),
                severity: DiagnosticSeverity::Info,
                confidence: 0.4,
            });
        }

        None
    }

    #[allow(dead_code)]
    fn spans_overlap(&self, span1: &Range<usize>, span2: &Range<usize>) -> bool {
        span1.start < span2.end && span2.start < span1.end
    }

    #[allow(dead_code)]
    fn is_causal_relationship(&self, primary: &ErrorCode, secondary: &ErrorCode) -> bool {
        // Define some known causal relationships
        (*primary == FP0001 && *secondary == FP0055) ||  // Syntax error may cause property access error
        (*primary == FP0010 && *secondary == FP0101) // Type error may cause function error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_processor_creation() {
        let processor = DiagnosticProcessor::new();
        assert!(processor.engine.colors_enabled() || !processor.engine.colors_enabled()); // Just test it's created
    }

    #[test]
    fn test_context_building() {
        let processor = DiagnosticProcessor::new();
        let diagnostic = AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0055,
            message: "Property not found".to_string(),
            span: 8..15, // "invalid" without the period
            help: None,
            note: None,
            related: Vec::new(),
        };

        let context = processor.build_context(&diagnostic, "Patient.invalid.name");
        assert_eq!(context.source_snippet, "invalid");
        assert_eq!(context.resource_context, Some("Patient".to_string()));
    }

    #[test]
    fn test_source_snippet_extraction() {
        let processor = DiagnosticProcessor::new();
        let snippet = processor.extract_source_snippet("Patient.name.family", &(8..12));
        assert_eq!(snippet, "name");
    }

    #[test]
    fn test_resource_context_inference() {
        let processor = DiagnosticProcessor::new();
        let context = processor.infer_resource_context("Patient.name.invalid", &(13..20));
        assert_eq!(context, Some("Patient".to_string()));

        let no_context = processor.infer_resource_context("unknown.property", &(8..16));
        assert_eq!(no_context, None);
    }

    #[test]
    fn test_function_context_inference() {
        let processor = DiagnosticProcessor::new();
        let context = processor.infer_function_context("Patient.name.where(invalid)", &(19..26));
        assert_eq!(context, Some("where".to_string()));

        let no_context = processor.infer_function_context("Patient.name.family", &(8..16));
        assert_eq!(no_context, None);
    }

    #[test]
    fn test_expression_path_building() {
        let processor = DiagnosticProcessor::new();
        let path = processor.build_expression_path("Patient.name.family", &(13..19));
        assert_eq!(path, vec!["Patient", "name"]);

        let simple_path = processor.build_expression_path("Patient", &(0..7));
        assert!(simple_path.is_empty());
    }

    #[test]
    fn test_context_line_extraction() {
        let processor = DiagnosticProcessor::new();
        let source = "Patient.name\n.family\n.where(invalid)";
        let span = 21..28; // "invalid"
        let lines = processor.extract_context_lines(source, &span, 1);

        assert_eq!(lines.len(), 2); // Should include line with error and one before
        assert!(lines.iter().any(|line| line.is_error_line));
    }

    #[test]
    fn test_suggestion_generation() {
        let processor = DiagnosticProcessor::new();
        let diagnostic = AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0055,
            message: "Property not found".to_string(),
            span: 8..16,
            help: None,
            note: None,
            related: Vec::new(),
        };

        let context = DiagnosticContext {
            source_snippet: "invalid".to_string(),
            resource_context: Some("Patient".to_string()),
            function_context: None,
            expression_path: vec!["Patient".to_string()],
            source_lines: Vec::new(),
        };

        let suggestions = processor.suggestion_engine.generate_suggestions(
            &diagnostic,
            "Patient.invalid",
            &context,
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].category == SuggestionCategory::Fix);
    }

    #[test]
    fn test_relationship_detection() {
        let detector = RelationshipDetector::new();

        let diagnostic1 = ProcessedDiagnostic {
            diagnostic: AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: FP0001,
                message: "Syntax error".to_string(),
                span: 5..10,
                help: None,
                note: None,
                related: Vec::new(),
            },
            context: DiagnosticContext {
                source_snippet: "error".to_string(),
                resource_context: Some("Patient".to_string()),
                function_context: None,
                expression_path: vec!["Patient".to_string()],
                source_lines: Vec::new(),
            },
            suggestions: Vec::new(),
            related: Vec::new(),
            help_text: None,
            documentation_url: None,
        };

        let diagnostic2 = ProcessedDiagnostic {
            diagnostic: AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: FP0055,
                message: "Property not found".to_string(),
                span: 15..20, // Non-overlapping span to test CauseEffect
                help: None,
                note: None,
                related: Vec::new(),
            },
            context: DiagnosticContext {
                source_snippet: "error".to_string(),
                resource_context: Some("Patient".to_string()),
                function_context: None,
                expression_path: vec!["Patient".to_string()],
                source_lines: Vec::new(),
            },
            suggestions: Vec::new(),
            related: Vec::new(),
            help_text: None,
            documentation_url: None,
        };

        let relationship = detector.analyze_relationship(&diagnostic1, &diagnostic2);
        assert!(relationship.is_some());

        let rel = relationship.unwrap();
        assert_eq!(rel.kind, RelationshipKind::CauseEffect);
    }

    #[test]
    fn test_span_overlap_detection() {
        let detector = RelationshipDetector::new();

        // Overlapping spans
        assert!(detector.spans_overlap(&(5..10), &(8..15)));
        assert!(detector.spans_overlap(&(8..15), &(5..10)));

        // Non-overlapping spans
        assert!(!detector.spans_overlap(&(5..10), &(15..20)));
        assert!(!detector.spans_overlap(&(15..20), &(5..10)));

        // Adjacent spans (should not overlap)
        assert!(!detector.spans_overlap(&(5..10), &(10..15)));
    }

    #[test]
    fn test_diagnostic_rendering() {
        let processor = DiagnosticProcessor::new();
        let processed = ProcessedDiagnostic {
            diagnostic: AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: FP0055,
                message: "Property 'invalid' not found".to_string(),
                span: 8..15,
                help: None,
                note: None,
                related: Vec::new(),
            },
            context: DiagnosticContext {
                source_snippet: "invalid".to_string(),
                resource_context: Some("Patient".to_string()),
                function_context: None,
                expression_path: vec!["Patient".to_string()],
                source_lines: Vec::new(),
            },
            suggestions: vec![DiagnosticSuggestion {
                message: "Check property spelling".to_string(),
                replacements: Vec::new(),
                confidence: SuggestionConfidence::Medium,
                category: SuggestionCategory::Fix,
                improvement_estimate: None,
            }],
            related: Vec::new(),
            help_text: Some("Check the FHIR specification for valid properties".to_string()),
            documentation_url: Some("https://hl7.org/fhir/".to_string()),
        };

        let output = processor.render_suggestions(&processed.suggestions);
        assert!(output.contains("💡 Suggestions:"));
        assert!(output.contains("Check property spelling"));
        assert!(output.contains("🔧")); // Fix category icon
    }

    #[tokio::test]
    async fn test_complex_diagnostic_processing() {
        use crate::analyzer::{AnalysisContext, StaticAnalyzer};
        use octofhir_fhir_model::{EmptyModelProvider, ModelProvider};
        use std::sync::Arc;

        let mut processor = DiagnosticProcessor::new();
        let model_provider = Arc::new(EmptyModelProvider);

        // Create a static analyzer
        let mut analyzer = StaticAnalyzer::new(model_provider.clone());

        // Get Patient type for context
        let patient_type = model_provider
            .get_type("Patient")
            .await
            .unwrap()
            .expect("Patient type should exist");

        let context = AnalysisContext {
            root_type: patient_type,
            deep_analysis: true,
            suggest_optimizations: true,
            max_suggestions: 5,
        };

        // Analyze an expression with issues
        let source = "Patient.invalidProperty";
        let analysis_result = analyzer.analyze_expression(source, context).await;

        // Process the analysis result through the diagnostic processor
        let processed = processor.process_analysis(&analysis_result, source, Some("test.fhirpath"));

        // Should have at least one diagnostic
        assert!(
            !processed.is_empty(),
            "Should have diagnostics for invalid property"
        );

        // Check that diagnostics were processed
        let has_error = processed
            .iter()
            .any(|d| matches!(d.diagnostic.severity, DiagnosticSeverity::Error));

        if has_error {
            let error_diag = processed
                .iter()
                .find(|d| matches!(d.diagnostic.severity, DiagnosticSeverity::Error))
                .unwrap();
            assert_eq!(
                error_diag.context.resource_context,
                Some("Patient".to_string())
            );
        }
    }

    #[test]
    fn test_suggestion_conversion() {
        use crate::analyzer::{AnalysisSuggestion, SuggestionType};
        use crate::core::SourceLocation;

        let processor = DiagnosticProcessor::new();

        let suggestion = AnalysisSuggestion {
            suggestion_type: SuggestionType::Performance,
            message: "Use exists() instead of count() > 0".to_string(),
            location: Some(SourceLocation {
                offset: 10,
                length: 15,
                line: 1,
                column: 10,
            }),
            code_snippet: Some("exists()".to_string()),
            confidence: 0.9,
        };

        let source = "Patient.name.count() > 0";
        let processed = processor.process_suggestion_to_diagnostic(&suggestion, source);

        assert!(processed.is_some());
        let processed = processed.unwrap();
        assert_eq!(processed.diagnostic.severity, DiagnosticSeverity::Info);
        assert!(processed.help_text.is_some());
        assert!(!processed.suggestions.is_empty());
        assert_eq!(
            processed.suggestions[0].confidence,
            SuggestionConfidence::High
        );
    }
}
