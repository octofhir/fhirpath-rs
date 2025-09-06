//! Optimization report generator for FHIRPath expressions
//!
//! This module provides comprehensive reporting of optimization analysis results,
//! including performance scores, suggestions, complexity issues, and pattern matches.

use crate::analyzer::OptimizationKind;
use crate::analyzer::optimization_detector::{
    ComplexityIssueType, DepthAnalysis, FunctionCallStats, IssueSeverity,
    OptimizationAnalysisResult, PatternType,
};
use crate::core::SourceLocation;
use std::fmt::Write;

/// Configuration for optimization report generation
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Include detailed pattern match information
    pub include_pattern_details: bool,
    /// Include function call statistics
    pub include_function_stats: bool,
    /// Include depth analysis
    pub include_depth_analysis: bool,
    /// Maximum number of suggestions to show
    pub max_suggestions: Option<usize>,
    /// Show only high-impact suggestions
    pub high_impact_only: bool,
    /// Include source code snippets
    pub include_source_snippets: bool,
    /// Color output for terminal display
    pub colored_output: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_pattern_details: true,
            include_function_stats: true,
            include_depth_analysis: true,
            max_suggestions: None,
            high_impact_only: false,
            include_source_snippets: false,
            colored_output: true,
        }
    }
}

/// Optimization report generator
pub struct OptimizationReporter {
    config: ReportConfig,
}

impl OptimizationReporter {
    /// Create a new optimization reporter with default configuration
    pub fn new() -> Self {
        Self {
            config: ReportConfig::default(),
        }
    }

    /// Create a new optimization reporter with custom configuration
    pub fn with_config(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate a comprehensive optimization report
    pub fn generate_report(
        &self,
        result: &OptimizationAnalysisResult,
        source_code: Option<&str>,
        filename: Option<&str>,
    ) -> String {
        let mut report = String::new();

        // Header section
        self.write_header(&mut report, filename);

        // Performance overview
        self.write_performance_overview(&mut report, result);

        // Optimization suggestions
        self.write_optimization_suggestions(&mut report, result, source_code);

        // Complexity issues
        self.write_complexity_issues(&mut report, result);

        // Pattern matches (if enabled)
        if self.config.include_pattern_details {
            self.write_pattern_matches(&mut report, result);
        }

        // Function call statistics (if enabled)
        if self.config.include_function_stats {
            self.write_function_stats(&mut report, &result.function_call_stats);
        }

        // Depth analysis (if enabled)
        if self.config.include_depth_analysis {
            self.write_depth_analysis(&mut report, &result.depth_analysis);
        }

        // Summary and recommendations
        self.write_summary(&mut report, result);

        report
    }

    /// Generate a compact summary report
    pub fn generate_summary(&self, result: &OptimizationAnalysisResult) -> String {
        let mut summary = String::new();

        let score_icon = self.get_performance_icon(result.performance_score);
        let _ = writeln!(
            summary,
            "{} Performance Score: {:.1}/1.0",
            score_icon, result.performance_score
        );

        let impact_suggestions = result
            .suggestions
            .iter()
            .filter(|s| s.estimated_improvement >= 0.2)
            .count();

        if impact_suggestions > 0 {
            let _ = writeln!(
                summary,
                "⚠️  {} high-impact optimization opportunities",
                impact_suggestions
            );
        }

        if !result.complexity_issues.is_empty() {
            let critical_issues = result
                .complexity_issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
                .count();
            if critical_issues > 0 {
                let _ = writeln!(
                    summary,
                    "🔥 {} critical performance issues",
                    critical_issues
                );
            }
        }

        summary
    }

    /// Generate JSON report for programmatic consumption
    pub fn generate_json_report(&self, result: &OptimizationAnalysisResult) -> serde_json::Value {
        serde_json::json!({
            "performance_score": result.performance_score,
            "optimization_suggestions": result.suggestions.len(),
            "complexity_issues": result.complexity_issues.len(),
            "pattern_matches": result.pattern_matches.len(),
            "high_impact_suggestions": result.suggestions.iter()
                .filter(|s| s.estimated_improvement >= 0.2)
                .count(),
            "critical_issues": result.complexity_issues.iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
                .count(),
            "suggestions": result.suggestions.iter().map(|s| {
                serde_json::json!({
                    "kind": format!("{:?}", s.kind),
                    "message": s.message,
                    "estimated_improvement": s.estimated_improvement,
                    "location": s.location.as_ref().map(|l| serde_json::json!({
                        "start": l.offset,
                        "end": l.offset + l.length
                    }))
                })
            }).collect::<Vec<_>>(),
            "complexity_issues": result.complexity_issues.iter().map(|i| {
                serde_json::json!({
                    "issue_type": format!("{:?}", i.issue_type),
                    "severity": format!("{:?}", i.severity),
                    "description": i.description,
                    "suggested_fix": i.suggested_fix,
                    "performance_impact": i.performance_impact
                })
            }).collect::<Vec<_>>(),
            "function_stats": {
                "total_calls": result.function_call_stats.total_calls,
                "expensive_calls": result.function_call_stats.expensive_calls,
                "cacheable_calls": result.function_call_stats.cacheable_calls,
                "frequent_functions": result.function_call_stats.frequent_functions
            },
            "depth_analysis": {
                "max_property_depth": result.depth_analysis.max_property_depth,
                "max_expression_depth": result.depth_analysis.max_expression_depth,
                "deep_expressions": result.depth_analysis.deep_expressions
            }
        })
    }

    fn write_header(&self, report: &mut String, filename: Option<&str>) {
        let _ = writeln!(
            report,
            "╔══════════════════════════════════════════════════════════════╗"
        );
        let _ = writeln!(
            report,
            "║                  FHIRPath Optimization Analysis             ║"
        );
        let _ = writeln!(
            report,
            "╚══════════════════════════════════════════════════════════════╝"
        );
        let _ = writeln!(report);

        if let Some(name) = filename {
            let _ = writeln!(report, "📁 File: {}", name);
            let _ = writeln!(report);
        }
    }

    fn write_performance_overview(&self, report: &mut String, result: &OptimizationAnalysisResult) {
        let _ = writeln!(report, "📊 PERFORMANCE OVERVIEW");
        let _ = writeln!(report, "{}", "═".repeat(50));

        let score_icon = self.get_performance_icon(result.performance_score);
        let score_desc = self.get_performance_description(result.performance_score);

        let _ = writeln!(
            report,
            "{} Performance Score: {:.1}/1.0 ({})",
            score_icon, result.performance_score, score_desc
        );

        let _ = writeln!(
            report,
            "🔧 Optimization Opportunities: {}",
            result.suggestions.len()
        );
        let _ = writeln!(
            report,
            "⚠️  Complexity Issues: {}",
            result.complexity_issues.len()
        );
        let _ = writeln!(
            report,
            "✨ Pattern Matches: {}",
            result.pattern_matches.len()
        );

        // High-impact suggestions count
        let high_impact = result
            .suggestions
            .iter()
            .filter(|s| s.estimated_improvement >= 0.3)
            .count();
        if high_impact > 0 {
            let _ = writeln!(report, "🔥 High-Impact Suggestions: {}", high_impact);
        }

        let _ = writeln!(report);
    }

    fn write_optimization_suggestions(
        &self,
        report: &mut String,
        result: &OptimizationAnalysisResult,
        source_code: Option<&str>,
    ) {
        if result.suggestions.is_empty() {
            let _ = writeln!(report, "✅ OPTIMIZATION SUGGESTIONS");
            let _ = writeln!(report, "{}", "═".repeat(50));
            let _ = writeln!(
                report,
                "No optimization opportunities found! Your expression is well-optimized."
            );
            let _ = writeln!(report);
            return;
        }

        let _ = writeln!(report, "💡 OPTIMIZATION SUGGESTIONS");
        let _ = writeln!(report, "{}", "═".repeat(50));

        // Filter and sort suggestions
        let mut suggestions = result.suggestions.clone();

        if self.config.high_impact_only {
            suggestions.retain(|s| s.estimated_improvement >= 0.2);
        }

        if let Some(max) = self.config.max_suggestions {
            suggestions.truncate(max);
        }

        // Sort by estimated improvement (descending)
        suggestions.sort_by(|a, b| {
            b.estimated_improvement
                .partial_cmp(&a.estimated_improvement)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, suggestion) in suggestions.iter().enumerate() {
            let impact_icon = self.get_impact_icon(suggestion.estimated_improvement);
            let impact_level = self.get_impact_level(suggestion.estimated_improvement);
            let kind_icon = self.get_optimization_kind_icon(&suggestion.kind);

            let _ = writeln!(
                report,
                "{}. {} [{}] {:?}",
                i + 1,
                kind_icon,
                impact_level,
                suggestion.kind
            );
            let _ = writeln!(report, "   {}", suggestion.message);

            if suggestion.estimated_improvement > 0.0 {
                let _ = writeln!(
                    report,
                    "   {} Estimated improvement: {:.0}%",
                    impact_icon,
                    suggestion.estimated_improvement * 100.0
                );
            }

            if let Some(location) = &suggestion.location {
                let _ = writeln!(
                    report,
                    "   📍 Location: {}..{}",
                    location.offset,
                    location.offset + location.length
                );

                // Include source code snippet if available
                if self.config.include_source_snippets {
                    if let Some(source) = source_code {
                        if let Some(snippet) = self.extract_source_snippet(source, location) {
                            let _ = writeln!(report, "   📝 Source: {}", snippet);
                        }
                    }
                }
            }

            let _ = writeln!(report);
        }
    }

    fn write_complexity_issues(&self, report: &mut String, result: &OptimizationAnalysisResult) {
        if result.complexity_issues.is_empty() {
            return;
        }

        let _ = writeln!(report, "⚡ COMPLEXITY ISSUES");
        let _ = writeln!(report, "{}", "═".repeat(50));

        for issue in &result.complexity_issues {
            let severity_icon = self.get_severity_icon(&issue.severity);
            let issue_icon = self.get_complexity_issue_icon(&issue.issue_type);

            let _ = writeln!(
                report,
                "{} {} {:?}: {}",
                severity_icon, issue_icon, issue.issue_type, issue.description
            );

            if let Some(fix) = &issue.suggested_fix {
                let _ = writeln!(report, "   🔧 Suggested fix: {}", fix);
            }

            if issue.performance_impact > 0.0 {
                let _ = writeln!(
                    report,
                    "   📊 Performance impact: {:.0}%",
                    issue.performance_impact * 100.0
                );
            }

            if let Some(location) = &issue.location {
                let _ = writeln!(
                    report,
                    "   📍 Location: {}..{}",
                    location.offset,
                    location.offset + location.length
                );
            }

            let _ = writeln!(report);
        }
    }

    fn write_pattern_matches(&self, report: &mut String, result: &OptimizationAnalysisResult) {
        if result.pattern_matches.is_empty() {
            return;
        }

        let _ = writeln!(report, "🎯 PATTERN OPTIMIZATIONS");
        let _ = writeln!(report, "{}", "═".repeat(50));

        for pattern in &result.pattern_matches {
            let pattern_icon = self.get_pattern_type_icon(&pattern.pattern_type);
            let _ = writeln!(
                report,
                "{} {:?}: {}",
                pattern_icon, pattern.pattern_type, pattern.benefit
            );
            let _ = writeln!(report, "   {} → {}", pattern.original, pattern.suggested);

            if pattern.improvement_factor > 0.0 {
                let _ = writeln!(
                    report,
                    "   ⚡ Expected speedup: {:.0}%",
                    pattern.improvement_factor * 100.0
                );
            }

            if let Some(location) = &pattern.location {
                let _ = writeln!(
                    report,
                    "   📍 Location: {}..{}",
                    location.offset,
                    location.offset + location.length
                );
            }

            let _ = writeln!(report);
        }
    }

    fn write_function_stats(&self, report: &mut String, stats: &FunctionCallStats) {
        let _ = writeln!(report, "📈 FUNCTION CALL ANALYSIS");
        let _ = writeln!(report, "{}", "═".repeat(50));

        let _ = writeln!(report, "Total function calls: {}", stats.total_calls);
        let _ = writeln!(
            report,
            "Expensive calls: {} ({:.1}%)",
            stats.expensive_calls,
            if stats.total_calls > 0 {
                stats.expensive_calls as f32 / stats.total_calls as f32 * 100.0
            } else {
                0.0
            }
        );
        let _ = writeln!(
            report,
            "Cacheable calls: {} ({:.1}%)",
            stats.cacheable_calls,
            if stats.total_calls > 0 {
                stats.cacheable_calls as f32 / stats.total_calls as f32 * 100.0
            } else {
                0.0
            }
        );

        if !stats.frequent_functions.is_empty() {
            let _ = writeln!(report, "\n🔥 Most frequent functions:");
            for (func, count) in stats.frequent_functions.iter().take(5) {
                let _ = writeln!(report, "   • {} ({})", func, count);
            }
        }

        if !stats.replaceable_functions.is_empty() {
            let _ = writeln!(report, "\n⚠️  Functions that could be replaced:");
            for func in &stats.replaceable_functions {
                let _ = writeln!(report, "   • {}", func);
            }
        }

        let _ = writeln!(report);
    }

    fn write_depth_analysis(&self, report: &mut String, depth: &DepthAnalysis) {
        let _ = writeln!(report, "📊 DEPTH ANALYSIS");
        let _ = writeln!(report, "{}", "═".repeat(50));

        let _ = writeln!(
            report,
            "Max property access depth: {}",
            depth.max_property_depth
        );
        let _ = writeln!(
            report,
            "Max expression nesting depth: {}",
            depth.max_expression_depth
        );

        if depth.deep_expressions > 0 {
            let _ = writeln!(
                report,
                "⚠️  Deep expressions found: {}",
                depth.deep_expressions
            );
        }

        if !depth.depth_reduction_opportunities.is_empty() {
            let _ = writeln!(
                report,
                "🎯 Depth reduction opportunities: {}",
                depth.depth_reduction_opportunities.len()
            );
        }

        let _ = writeln!(report);
    }

    fn write_summary(&self, report: &mut String, result: &OptimizationAnalysisResult) {
        let _ = writeln!(report, "📋 SUMMARY & RECOMMENDATIONS");
        let _ = writeln!(report, "{}", "═".repeat(50));

        // Provide actionable recommendations based on analysis
        if result.performance_score >= 0.9 {
            let _ = writeln!(report, "🏆 Excellent! Your expression is highly optimized.");
        } else if result.performance_score >= 0.7 {
            let _ = writeln!(
                report,
                "✅ Good performance. Consider the suggestions above for further optimization."
            );
        } else if result.performance_score >= 0.5 {
            let _ = writeln!(
                report,
                "⚠️  Moderate performance. Multiple optimization opportunities exist."
            );
        } else {
            let _ = writeln!(
                report,
                "🔥 Performance concerns identified. Priority should be given to optimization."
            );
        }

        // Priority recommendations
        let high_impact_suggestions = result
            .suggestions
            .iter()
            .filter(|s| s.estimated_improvement >= 0.3)
            .count();

        if high_impact_suggestions > 0 {
            let _ = writeln!(report, "\n🎯 Priority Actions:");
            let _ = writeln!(
                report,
                "   • Focus on {} high-impact suggestions first",
                high_impact_suggestions
            );
        }

        let critical_issues = result
            .complexity_issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
            .count();

        if critical_issues > 0 {
            let _ = writeln!(
                report,
                "   • Address {} critical complexity issues",
                critical_issues
            );
        }

        if result.function_call_stats.expensive_calls > 0 {
            let _ = writeln!(
                report,
                "   • Consider alternatives to {} expensive function calls",
                result.function_call_stats.expensive_calls
            );
        }

        if result.depth_analysis.max_expression_depth > 7 {
            let _ = writeln!(
                report,
                "   • Break down deeply nested expressions for better readability"
            );
        }

        let _ = writeln!(
            report,
            "\n💡 For more details, see the individual sections above."
        );
    }

    // Helper methods for formatting and icons
    fn get_performance_icon(&self, score: f32) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match score {
            s if s >= 0.9 => "🟢",
            s if s >= 0.7 => "🟡",
            s if s >= 0.5 => "🟠",
            _ => "🔴",
        }
    }

    fn get_performance_description(&self, score: f32) -> &'static str {
        match score {
            s if s >= 0.9 => "Excellent",
            s if s >= 0.8 => "Very Good",
            s if s >= 0.7 => "Good",
            s if s >= 0.6 => "Fair",
            s if s >= 0.5 => "Needs Improvement",
            s if s >= 0.3 => "Poor",
            _ => "Critical",
        }
    }

    fn get_impact_icon(&self, improvement: f32) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match improvement {
            i if i >= 0.3 => "🔴",
            i if i >= 0.1 => "🟡",
            _ => "🟢",
        }
    }

    fn get_impact_level(&self, improvement: f32) -> &'static str {
        match improvement {
            i if i >= 0.4 => "CRITICAL IMPACT",
            i if i >= 0.3 => "HIGH IMPACT",
            i if i >= 0.1 => "MEDIUM IMPACT",
            i if i > 0.0 => "LOW IMPACT",
            _ => "QUALITY",
        }
    }

    fn get_optimization_kind_icon(&self, kind: &OptimizationKind) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match kind {
            OptimizationKind::ExpensiveOperation => "⚡",
            OptimizationKind::CollectionOptimization => "📊",
            OptimizationKind::CachableExpression => "💾",
            OptimizationKind::RedundantCondition => "🔄",
            OptimizationKind::FunctionSimplification => "🔧",
            OptimizationKind::DeepNesting => "📏",
            OptimizationKind::UnreachableCode => "❌",
            OptimizationKind::TypeCoercion => "🔀",
            OptimizationKind::PropertyCorrection => "🏷️",
        }
    }

    fn get_severity_icon(&self, severity: &IssueSeverity) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match severity {
            IssueSeverity::Critical => "🔥",
            IssueSeverity::High => "🔴",
            IssueSeverity::Medium => "🟠",
            IssueSeverity::Low => "🟡",
        }
    }

    fn get_complexity_issue_icon(&self, issue_type: &ComplexityIssueType) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match issue_type {
            ComplexityIssueType::ExpensiveOperation => "⚡",
            ComplexityIssueType::InefficientFilter => "🔍",
            ComplexityIssueType::DeepNesting => "📏",
            ComplexityIssueType::UnnecessaryIteration => "🔄",
            ComplexityIssueType::RepeatedSubexpression => "📋",
            ComplexityIssueType::SimplifiableFunction => "🔧",
            ComplexityIssueType::PropertyAccessOptimization => "🏷️",
            ComplexityIssueType::RedundantCondition => "❓",
            ComplexityIssueType::UnreachableCode => "❌",
            ComplexityIssueType::MissingIndex => "📍",
        }
    }

    fn get_pattern_type_icon(&self, pattern_type: &PatternType) -> &'static str {
        if !self.config.colored_output {
            return "•";
        }
        match pattern_type {
            PatternType::FilterCombination => "🔗",
            PatternType::IndexAccess => "📍",
            PatternType::EarlyExit => "⏭️",
            PatternType::CacheableExpression => "💾",
            PatternType::ExpensiveFunctionReplacement => "⚡",
            PatternType::SimplifyLogic => "🧩",
            PatternType::ExtractVariable => "📦",
            PatternType::ReduceComplexity => "🎯",
            PatternType::CombineOperations => "🔗",
            PatternType::NullSafety => "🛡️",
            PatternType::TypeSafety => "🔒",
            PatternType::EmptyCheck => "📋",
            PatternType::ReferenceCheck => "🔗",
        }
    }

    fn extract_source_snippet(&self, source: &str, location: &SourceLocation) -> Option<String> {
        let start = location.offset.min(source.len());
        let end = (location.offset + location.length).min(source.len());
        if start <= end {
            Some(source[start..end].to_string())
        } else {
            None
        }
    }
}

impl Default for OptimizationReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::optimization_detector::{
        DepthAnalysis, FunctionCallStats, OptimizationAnalysisResult,
    };

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert!(config.include_pattern_details);
        assert!(config.include_function_stats);
        assert!(config.include_depth_analysis);
        assert_eq!(config.max_suggestions, None);
        assert!(!config.high_impact_only);
    }

    #[test]
    fn test_optimization_reporter_creation() {
        let reporter = OptimizationReporter::new();
        assert!(reporter.config.colored_output);

        let config = ReportConfig {
            colored_output: false,
            ..ReportConfig::default()
        };
        let reporter = OptimizationReporter::with_config(config);
        assert!(!reporter.config.colored_output);
    }

    #[test]
    fn test_performance_icon_selection() {
        let reporter = OptimizationReporter::new();

        assert_eq!(reporter.get_performance_icon(0.95), "🟢");
        assert_eq!(reporter.get_performance_icon(0.75), "🟡");
        assert_eq!(reporter.get_performance_icon(0.55), "🟠");
        assert_eq!(reporter.get_performance_icon(0.25), "🔴");
    }

    #[test]
    fn test_impact_level_categorization() {
        let reporter = OptimizationReporter::new();

        assert_eq!(reporter.get_impact_level(0.5), "CRITICAL IMPACT");
        assert_eq!(reporter.get_impact_level(0.35), "HIGH IMPACT");
        assert_eq!(reporter.get_impact_level(0.15), "MEDIUM IMPACT");
        assert_eq!(reporter.get_impact_level(0.05), "LOW IMPACT");
        assert_eq!(reporter.get_impact_level(0.0), "QUALITY");
    }

    #[test]
    fn test_generate_summary() {
        let reporter = OptimizationReporter::new();
        let result = OptimizationAnalysisResult {
            suggestions: vec![],
            performance_score: 0.85,
            complexity_issues: vec![],
            pattern_matches: vec![],
            function_call_stats: FunctionCallStats {
                total_calls: 5,
                expensive_calls: 0,
                cacheable_calls: 2,
                frequent_functions: vec![],
                replaceable_functions: vec![],
            },
            depth_analysis: DepthAnalysis {
                max_property_depth: 3,
                max_expression_depth: 4,
                deep_expressions: 0,
                depth_reduction_opportunities: vec![],
            },
        };

        let summary = reporter.generate_summary(&result);
        assert!(summary.contains("Performance Score: 0.8/1.0"));
        assert!(summary.contains("🟡")); // Should show yellow icon for 0.85 score
    }

    #[test]
    fn test_json_report_generation() {
        let reporter = OptimizationReporter::new();
        let result = OptimizationAnalysisResult {
            suggestions: vec![],
            performance_score: 0.9,
            complexity_issues: vec![],
            pattern_matches: vec![],
            function_call_stats: FunctionCallStats {
                total_calls: 3,
                expensive_calls: 1,
                cacheable_calls: 1,
                frequent_functions: vec![("count".to_string(), 2)],
                replaceable_functions: vec![],
            },
            depth_analysis: DepthAnalysis {
                max_property_depth: 2,
                max_expression_depth: 3,
                deep_expressions: 0,
                depth_reduction_opportunities: vec![],
            },
        };

        let json_report = reporter.generate_json_report(&result);
        assert_eq!(json_report["performance_score"], 0.9);
        assert_eq!(json_report["function_stats"]["total_calls"], 3);
        assert_eq!(json_report["depth_analysis"]["max_property_depth"], 2);
    }
}
