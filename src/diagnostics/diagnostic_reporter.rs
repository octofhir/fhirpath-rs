//! Multiple diagnostic message reporting and analysis system
//!
//! This module provides comprehensive diagnostic reporting capabilities that can
//! collect, analyze, and present multiple diagnostic messages in a coherent way.

use super::{
    diagnostic::{DiagnosticCode, Severity},
    enhanced_diagnostic::EnhancedDiagnostic,
    location::SourceLocation,
};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// Comprehensive diagnostic report with multiple messages and analysis
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// All diagnostics in the report
    pub diagnostics: Vec<EnhancedDiagnostic>,
    /// Summary statistics
    pub summary: DiagnosticSummary,
    /// Grouped diagnostics by type
    pub grouped_diagnostics: GroupedDiagnostics,
    /// Analysis of diagnostic patterns
    pub analysis: DiagnosticAnalysis,
    /// Suggested workflow for fixing issues
    pub suggested_workflow: Vec<WorkflowStep>,
}

/// Summary statistics for diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticSummary {
    /// Total number of diagnostics
    pub total_count: usize,
    /// Count by severity
    pub error_count: usize,
    /// Number of warning diagnostics
    pub warning_count: usize,
    /// Number of info diagnostics
    pub info_count: usize,
    /// Number of hint diagnostics
    pub hint_count: usize,
    /// Most common diagnostic codes
    pub common_codes: Vec<(DiagnosticCode, usize)>,
    /// Overall severity (highest severity present)
    pub overall_severity: Severity,
}

/// Diagnostics grouped by various criteria
#[derive(Debug, Clone, Default)]
pub struct GroupedDiagnostics {
    /// Grouped by severity
    pub by_severity: BTreeMap<Severity, Vec<EnhancedDiagnostic>>,
    /// Grouped by diagnostic code
    pub by_code: HashMap<String, Vec<EnhancedDiagnostic>>,
    /// Grouped by location (line-based)
    pub by_location: BTreeMap<usize, Vec<EnhancedDiagnostic>>,
    /// Related diagnostics (errors that might be caused by the same issue)
    pub related_groups: Vec<Vec<EnhancedDiagnostic>>,
}

/// Analysis of diagnostic patterns
#[derive(Debug, Clone, Default)]
pub struct DiagnosticAnalysis {
    /// Detected error patterns
    pub error_patterns: Vec<ErrorPattern>,
    /// Root cause analysis
    pub root_causes: Vec<RootCause>,
    /// Cascading error detection
    pub cascading_errors: Vec<CascadingErrorGroup>,
    /// Confidence in the analysis (0.0 to 1.0)
    pub confidence: f32,
}

/// Detected error pattern
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    /// Pattern name
    pub name: String,
    /// Description of the pattern
    pub description: String,
    /// Diagnostics that match this pattern
    pub matching_diagnostics: Vec<usize>, // indices into diagnostics array
    /// Confidence in pattern detection (0.0 to 1.0)
    pub confidence: f32,
    /// Suggested fix for this pattern
    pub suggested_fix: String,
}

/// Root cause analysis result
#[derive(Debug, Clone)]
pub struct RootCause {
    /// Description of the root cause
    pub description: String,
    /// Primary diagnostic that indicates this root cause
    pub primary_diagnostic: usize, // index into diagnostics array
    /// Secondary diagnostics caused by this root cause
    pub secondary_diagnostics: Vec<usize>,
    /// Confidence in this root cause analysis (0.0 to 1.0)
    pub confidence: f32,
    /// Priority for fixing (1 = highest)
    pub fix_priority: u32,
}

/// Group of cascading errors
#[derive(Debug, Clone)]
pub struct CascadingErrorGroup {
    /// The initial error that caused others
    pub initial_error: usize, // index into diagnostics array
    /// Errors that cascaded from the initial error
    pub cascaded_errors: Vec<usize>,
    /// Description of the cascade
    pub cascade_description: String,
}

/// Workflow step for fixing issues
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Step number in the workflow
    pub step_number: usize,
    /// Title of the step
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Diagnostics addressed by this step
    pub addresses_diagnostics: Vec<usize>,
    /// Estimated effort (1-5 scale)
    pub effort: u32,
    /// Priority (1 = highest)
    pub priority: u32,
}

/// Diagnostic reporter that can collect and analyze multiple diagnostics
pub struct DiagnosticReporter {
    /// Collected diagnostics
    diagnostics: Vec<EnhancedDiagnostic>,
    /// Analysis configuration
    config: ReporterConfig,
}

/// Configuration for diagnostic reporting
#[derive(Debug, Clone)]
pub struct ReporterConfig {
    /// Maximum number of diagnostics to collect
    pub max_diagnostics: usize,
    /// Whether to perform pattern analysis
    pub enable_pattern_analysis: bool,
    /// Whether to perform root cause analysis
    pub enable_root_cause_analysis: bool,
    /// Whether to detect cascading errors
    pub enable_cascade_detection: bool,
    /// Whether to generate workflow suggestions
    pub enable_workflow_suggestions: bool,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            max_diagnostics: 100,
            enable_pattern_analysis: true,
            enable_root_cause_analysis: true,
            enable_cascade_detection: true,
            enable_workflow_suggestions: true,
        }
    }
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticReporter {
    /// Create a new diagnostic reporter
    pub fn new() -> Self {
        Self::with_config(ReporterConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ReporterConfig) -> Self {
        Self {
            diagnostics: Vec::new(),
            config,
        }
    }

    /// Add a diagnostic to the reporter
    pub fn add_diagnostic(&mut self, diagnostic: EnhancedDiagnostic) {
        if self.diagnostics.len() < self.config.max_diagnostics {
            self.diagnostics.push(diagnostic);
        }
    }

    /// Add multiple diagnostics
    pub fn add_diagnostics(&mut self, diagnostics: Vec<EnhancedDiagnostic>) {
        for diagnostic in diagnostics {
            self.add_diagnostic(diagnostic);
        }
    }

    /// Generate comprehensive report
    pub fn generate_report(&self) -> DiagnosticReport {
        let summary = self.generate_summary();
        let grouped = self.group_diagnostics();
        let analysis = self.analyze_diagnostics(&grouped);
        let workflow = self.generate_workflow(&analysis);

        DiagnosticReport {
            diagnostics: self.diagnostics.clone(),
            summary,
            grouped_diagnostics: grouped,
            analysis,
            suggested_workflow: workflow,
        }
    }

    /// Generate summary statistics
    fn generate_summary(&self) -> DiagnosticSummary {
        let mut summary = DiagnosticSummary::default();
        let mut code_counts: HashMap<String, usize> = HashMap::new();

        summary.total_count = self.diagnostics.len();

        for diagnostic in &self.diagnostics {
            match diagnostic.diagnostic.severity {
                Severity::Error => summary.error_count += 1,
                Severity::Warning => summary.warning_count += 1,
                Severity::Info => summary.info_count += 1,
                Severity::Hint => summary.hint_count += 1,
            }

            let code_str = diagnostic.diagnostic.code_string();
            *code_counts.entry(code_str).or_insert(0) += 1;

            if diagnostic.diagnostic.severity > summary.overall_severity {
                summary.overall_severity = diagnostic.diagnostic.severity;
            }
        }

        // Sort codes by frequency
        let mut code_vec: Vec<_> = code_counts.into_iter().collect();
        code_vec.sort_by(|a, b| b.1.cmp(&a.1));

        summary.common_codes = code_vec
            .into_iter()
            .take(5)
            .map(|(code, count)| (DiagnosticCode::Custom(code), count))
            .collect();

        summary
    }

    /// Group diagnostics by various criteria
    fn group_diagnostics(&self) -> GroupedDiagnostics {
        let mut grouped = GroupedDiagnostics::default();

        for diagnostic in self.diagnostics.iter() {
            // Group by severity
            grouped
                .by_severity
                .entry(diagnostic.diagnostic.severity)
                .or_insert_with(Vec::new)
                .push(diagnostic.clone());

            // Group by code
            let code_str = diagnostic.diagnostic.code_string();
            grouped
                .by_code
                .entry(code_str)
                .or_insert_with(Vec::new)
                .push(diagnostic.clone());

            // Group by location (simplified - by line)
            let line = self.extract_line_number(&diagnostic.diagnostic.location);
            grouped
                .by_location
                .entry(line)
                .or_insert_with(Vec::new)
                .push(diagnostic.clone());
        }

        // Find related diagnostics
        grouped.related_groups = self.find_related_diagnostics();

        grouped
    }

    /// Analyze diagnostic patterns
    fn analyze_diagnostics(&self, grouped: &GroupedDiagnostics) -> DiagnosticAnalysis {
        let mut analysis = DiagnosticAnalysis::default();

        if self.config.enable_pattern_analysis {
            analysis.error_patterns = self.detect_error_patterns(grouped);
        }

        if self.config.enable_root_cause_analysis {
            analysis.root_causes = self.analyze_root_causes(grouped);
        }

        if self.config.enable_cascade_detection {
            analysis.cascading_errors = self.detect_cascading_errors();
        }

        analysis.confidence = self.calculate_analysis_confidence(&analysis);

        analysis
    }

    /// Detect common error patterns
    fn detect_error_patterns(&self, grouped: &GroupedDiagnostics) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();

        // Pattern: Multiple undefined variables
        if let Some(undefined_vars) = grouped.by_code.get("E202") {
            if undefined_vars.len() > 1 {
                patterns.push(ErrorPattern {
                    name: "Multiple Undefined Variables".to_string(),
                    description: "Multiple variables are used without being defined".to_string(),
                    matching_diagnostics: self.find_diagnostic_indices(undefined_vars),
                    confidence: 0.9,
                    suggested_fix: "Define variables before use or check variable names for typos"
                        .to_string(),
                });
            }
        }

        // Pattern: Syntax error cascade
        let syntax_errors: Vec<_> = grouped
            .by_code
            .keys()
            .filter(|code| code.starts_with("E00"))
            .collect();

        if syntax_errors.len() > 2 {
            let matching_diagnostics = syntax_errors
                .iter()
                .filter_map(|code| grouped.by_code.get(code.as_str()))
                .flatten()
                .enumerate()
                .map(|(i, _)| i)
                .collect();

            patterns.push(ErrorPattern {
                name: "Syntax Error Cascade".to_string(),
                description: "Multiple syntax errors, likely caused by a single mistake"
                    .to_string(),
                matching_diagnostics,
                confidence: 0.8,
                suggested_fix: "Fix the first syntax error, others may resolve automatically"
                    .to_string(),
            });
        }

        // Pattern: Type mismatch chain
        if let Some(type_errors) = grouped.by_code.get("E100") {
            if type_errors.len() > 1 {
                patterns.push(ErrorPattern {
                    name: "Type Mismatch Chain".to_string(),
                    description: "Multiple type mismatches in the same expression".to_string(),
                    matching_diagnostics: self.find_diagnostic_indices(type_errors),
                    confidence: 0.7,
                    suggested_fix: "Check the types of variables and function parameters"
                        .to_string(),
                });
            }
        }

        patterns
    }

    /// Analyze root causes
    fn analyze_root_causes(&self, _grouped: &GroupedDiagnostics) -> Vec<RootCause> {
        let mut root_causes = Vec::new();

        // Look for primary errors that could cause secondary errors
        for (primary_idx, primary_diagnostic) in self.diagnostics.iter().enumerate() {
            let mut secondary_indices = Vec::new();

            match &primary_diagnostic.diagnostic.code {
                DiagnosticCode::UnclosedString => {
                    // Unclosed strings often cause multiple downstream errors
                    for (idx, diagnostic) in
                        self.diagnostics.iter().enumerate().skip(primary_idx + 1)
                    {
                        if matches!(
                            diagnostic.diagnostic.code,
                            DiagnosticCode::UnexpectedToken | DiagnosticCode::ExpectedToken(_)
                        ) {
                            secondary_indices.push(idx);
                        }
                    }

                    if !secondary_indices.is_empty() {
                        root_causes.push(RootCause {
                            description: "Unclosed string literal causing parser confusion"
                                .to_string(),
                            primary_diagnostic: primary_idx,
                            secondary_diagnostics: secondary_indices,
                            confidence: 0.9,
                            fix_priority: 1,
                        });
                    }
                }

                DiagnosticCode::ExpectedToken(token) if token == ")" => {
                    // Missing closing parentheses can cause multiple issues
                    for (idx, diagnostic) in
                        self.diagnostics.iter().enumerate().skip(primary_idx + 1)
                    {
                        if matches!(diagnostic.diagnostic.code, DiagnosticCode::UnexpectedToken) {
                            secondary_indices.push(idx);
                        }
                    }

                    if !secondary_indices.is_empty() {
                        root_causes.push(RootCause {
                            description: "Missing closing parenthesis causing parse errors"
                                .to_string(),
                            primary_diagnostic: primary_idx,
                            secondary_diagnostics: secondary_indices,
                            confidence: 0.8,
                            fix_priority: 1,
                        });
                    }
                }

                _ => {}
            }
        }

        root_causes
    }

    /// Detect cascading errors
    fn detect_cascading_errors(&self) -> Vec<CascadingErrorGroup> {
        let mut cascading_groups = Vec::new();

        for (initial_idx, initial_diagnostic) in self.diagnostics.iter().enumerate() {
            if initial_diagnostic.diagnostic.severity == Severity::Error {
                let mut cascaded_indices = Vec::new();

                // Look for errors that appear after this one and might be related
                for (idx, diagnostic) in self.diagnostics.iter().enumerate().skip(initial_idx + 1) {
                    if self.could_be_cascading_error(initial_diagnostic, diagnostic) {
                        cascaded_indices.push(idx);
                    }
                }

                if !cascaded_indices.is_empty() {
                    cascading_groups.push(CascadingErrorGroup {
                        initial_error: initial_idx,
                        cascaded_errors: cascaded_indices,
                        cascade_description: format!(
                            "Errors cascading from {} at position {}",
                            initial_diagnostic.diagnostic.code_string(),
                            self.extract_position(&initial_diagnostic.diagnostic.location)
                        ),
                    });
                }
            }
        }

        cascading_groups
    }

    /// Generate workflow suggestions
    fn generate_workflow(&self, analysis: &DiagnosticAnalysis) -> Vec<WorkflowStep> {
        let mut workflow = Vec::new();
        let mut step_number = 1;

        // Start with root causes (highest priority)
        for root_cause in &analysis.root_causes {
            workflow.push(WorkflowStep {
                step_number,
                title: "Address Root Cause".to_string(),
                description: root_cause.description.clone(),
                addresses_diagnostics: {
                    let mut indices = vec![root_cause.primary_diagnostic];
                    indices.extend(&root_cause.secondary_diagnostics);
                    indices
                },
                effort: 2,
                priority: root_cause.fix_priority,
            });
            step_number += 1;
        }

        // Then handle error patterns
        for pattern in &analysis.error_patterns {
            if pattern.confidence > 0.7 {
                workflow.push(WorkflowStep {
                    step_number,
                    title: format!("Fix {}", pattern.name),
                    description: pattern.suggested_fix.clone(),
                    addresses_diagnostics: pattern.matching_diagnostics.clone(),
                    effort: 3,
                    priority: 2,
                });
                step_number += 1;
            }
        }

        // Finally, remaining individual errors
        let addressed_indices: std::collections::HashSet<usize> = workflow
            .iter()
            .flat_map(|step| &step.addresses_diagnostics)
            .cloned()
            .collect();

        for (idx, diagnostic) in self.diagnostics.iter().enumerate() {
            if !addressed_indices.contains(&idx) && diagnostic.diagnostic.is_error() {
                workflow.push(WorkflowStep {
                    step_number,
                    title: "Fix Individual Error".to_string(),
                    description: diagnostic.diagnostic.message.clone(),
                    addresses_diagnostics: vec![idx],
                    effort: 1,
                    priority: 3,
                });
                step_number += 1;
            }
        }

        // Sort by priority
        workflow.sort_by_key(|step| step.priority);

        workflow
    }

    // Helper methods

    fn extract_line_number(&self, location: &SourceLocation) -> usize {
        location.span.start.line
    }

    fn extract_position(&self, location: &SourceLocation) -> usize {
        location.span.start.column
    }

    fn find_diagnostic_indices(&self, diagnostics: &[EnhancedDiagnostic]) -> Vec<usize> {
        diagnostics
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                self.diagnostics
                    .iter()
                    .position(|our_d| our_d == d)
                    .map(|_| i)
            })
            .collect()
    }

    fn find_related_diagnostics(&self) -> Vec<Vec<EnhancedDiagnostic>> {
        // Simple implementation - group diagnostics that are close together
        let mut related_groups = Vec::new();
        let mut current_group = Vec::new();
        let mut last_position = 0;

        for diagnostic in &self.diagnostics {
            let position = self.extract_position(&diagnostic.diagnostic.location);

            if position.saturating_sub(last_position) <= 10 && !current_group.is_empty() {
                current_group.push(diagnostic.clone());
            } else {
                if current_group.len() > 1 {
                    related_groups.push(current_group);
                }
                current_group = vec![diagnostic.clone()];
            }
            last_position = position;
        }

        if current_group.len() > 1 {
            related_groups.push(current_group);
        }

        related_groups
    }

    fn could_be_cascading_error(
        &self,
        initial: &EnhancedDiagnostic,
        potential_cascade: &EnhancedDiagnostic,
    ) -> bool {
        // Simple heuristic - errors close together are likely related
        let initial_pos = self.extract_position(&initial.diagnostic.location);
        let cascade_pos = self.extract_position(&potential_cascade.diagnostic.location);

        cascade_pos > initial_pos && cascade_pos - initial_pos <= 20
    }

    fn calculate_analysis_confidence(&self, analysis: &DiagnosticAnalysis) -> f32 {
        if self.diagnostics.is_empty() {
            return 1.0;
        }

        let pattern_confidence = if analysis.error_patterns.is_empty() {
            0.5
        } else {
            analysis
                .error_patterns
                .iter()
                .map(|p| p.confidence)
                .sum::<f32>()
                / analysis.error_patterns.len() as f32
        };

        let root_cause_confidence = if analysis.root_causes.is_empty() {
            0.5
        } else {
            analysis
                .root_causes
                .iter()
                .map(|r| r.confidence)
                .sum::<f32>()
                / analysis.root_causes.len() as f32
        };

        (pattern_confidence + root_cause_confidence) / 2.0
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Get the number of collected diagnostics
    pub fn diagnostic_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.diagnostic.is_error())
    }

    /// Get all error diagnostics
    pub fn errors(&self) -> Vec<&EnhancedDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.diagnostic.is_error())
            .collect()
    }

    /// Get all warning diagnostics
    pub fn warnings(&self) -> Vec<&EnhancedDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.diagnostic.is_warning())
            .collect()
    }
}

impl fmt::Display for DiagnosticReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Diagnostic Report")?;
        writeln!(f, "================")?;
        writeln!(f)?;

        // Summary
        writeln!(f, "Summary:")?;
        writeln!(f, "  Total diagnostics: {}", self.summary.total_count)?;
        writeln!(f, "  Errors: {}", self.summary.error_count)?;
        writeln!(f, "  Warnings: {}", self.summary.warning_count)?;
        writeln!(f, "  Info: {}", self.summary.info_count)?;
        writeln!(f, "  Hints: {}", self.summary.hint_count)?;
        writeln!(f, "  Overall severity: {}", self.summary.overall_severity)?;
        writeln!(f)?;

        // Error patterns
        if !self.analysis.error_patterns.is_empty() {
            writeln!(f, "Detected Patterns:")?;
            for pattern in &self.analysis.error_patterns {
                writeln!(
                    f,
                    "  • {} (confidence: {:.0}%)",
                    pattern.name,
                    pattern.confidence * 100.0
                )?;
                writeln!(f, "    {}", pattern.description)?;
                writeln!(f, "    Fix: {}", pattern.suggested_fix)?;
            }
            writeln!(f)?;
        }

        // Root causes
        if !self.analysis.root_causes.is_empty() {
            writeln!(f, "Root Causes:")?;
            for root_cause in &self.analysis.root_causes {
                writeln!(
                    f,
                    "  • {} (confidence: {:.0}%)",
                    root_cause.description,
                    root_cause.confidence * 100.0
                )?;
            }
            writeln!(f)?;
        }

        // Workflow
        if !self.suggested_workflow.is_empty() {
            writeln!(f, "Suggested Fix Workflow:")?;
            for step in &self.suggested_workflow {
                writeln!(
                    f,
                    "  {}. {} (effort: {}/5)",
                    step.step_number, step.title, step.effort
                )?;
                writeln!(f, "     {}", step.description)?;
            }
            writeln!(f)?;
        }

        // Individual diagnostics
        writeln!(f, "All Diagnostics:")?;
        for (i, diagnostic) in self.diagnostics.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, diagnostic)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Diagnostic;
    use crate::diagnostics::location::{Position, Span};

    fn create_test_diagnostic(
        severity: Severity,
        code: DiagnosticCode,
        message: &str,
    ) -> EnhancedDiagnostic {
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("test".to_string()),
            file_path: None,
        };

        let diagnostic = Diagnostic::new(severity, code, message.to_string(), location);
        EnhancedDiagnostic::from_diagnostic(diagnostic)
    }

    #[test]
    fn test_diagnostic_reporter_creation() {
        let reporter = DiagnosticReporter::new();
        assert_eq!(reporter.diagnostic_count(), 0);
        assert!(!reporter.has_errors());
    }

    #[test]
    fn test_adding_diagnostics() {
        let mut reporter = DiagnosticReporter::new();

        let diagnostic = create_test_diagnostic(
            Severity::Error,
            DiagnosticCode::UnknownFunction,
            "Unknown function 'test'",
        );

        reporter.add_diagnostic(diagnostic);
        assert_eq!(reporter.diagnostic_count(), 1);
        assert!(reporter.has_errors());
    }

    #[test]
    fn test_report_generation() {
        let mut reporter = DiagnosticReporter::new();

        // Add various diagnostics
        reporter.add_diagnostic(create_test_diagnostic(
            Severity::Error,
            DiagnosticCode::UnknownFunction,
            "Unknown function 'test'",
        ));
        reporter.add_diagnostic(create_test_diagnostic(
            Severity::Warning,
            DiagnosticCode::TypeMismatch {
                expected: "string".to_string(),
                actual: "integer".to_string(),
            },
            "Type mismatch",
        ));
        reporter.add_diagnostic(create_test_diagnostic(
            Severity::Error,
            DiagnosticCode::UndefinedVariable,
            "Undefined variable '$test'",
        ));

        let report = reporter.generate_report();

        assert_eq!(report.summary.total_count, 3);
        assert_eq!(report.summary.error_count, 2);
        assert_eq!(report.summary.warning_count, 1);
        assert_eq!(report.summary.overall_severity, Severity::Error);

        assert!(!report.grouped_diagnostics.by_severity.is_empty());
        assert!(!report.suggested_workflow.is_empty());
    }

    #[test]
    fn test_pattern_detection() {
        let mut reporter = DiagnosticReporter::new();

        // Add multiple undefined variable errors (should trigger pattern)
        for i in 0..3 {
            reporter.add_diagnostic(create_test_diagnostic(
                Severity::Error,
                DiagnosticCode::UndefinedVariable,
                &format!("Undefined variable '$var{i}'"),
            ));
        }

        let report = reporter.generate_report();

        // Should detect the multiple undefined variables pattern
        assert!(!report.analysis.error_patterns.is_empty());
        let pattern = &report.analysis.error_patterns[0];
        assert!(pattern.name.contains("Undefined Variables"));
    }

    #[test]
    fn test_root_cause_analysis() {
        let mut reporter = DiagnosticReporter::new();

        // Add an unclosed string followed by unexpected token errors
        reporter.add_diagnostic(create_test_diagnostic(
            Severity::Error,
            DiagnosticCode::UnclosedString,
            "Unclosed string literal",
        ));
        reporter.add_diagnostic(create_test_diagnostic(
            Severity::Error,
            DiagnosticCode::UnexpectedToken,
            "Unexpected token",
        ));

        let report = reporter.generate_report();

        // Should identify root cause
        assert!(!report.analysis.root_causes.is_empty());
        let root_cause = &report.analysis.root_causes[0];
        assert!(root_cause.description.contains("Unclosed string"));
    }
}
