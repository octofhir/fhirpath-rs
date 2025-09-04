//! Advanced batch formatter for multiple diagnostics
//!
//! This module provides comprehensive formatting for batches of diagnostics
//! with different output formats and beautiful visual presentation.

use super::{DiagnosticBatch, DiagnosticEngine, DiagnosticFormatter, DiagnosticSeverity, AriadneDiagnostic};
use crate::diagnostics::collector::DiagnosticStatistics;
use serde_json::{json, Value};

/// Advanced formatter for batches of diagnostics
pub struct BatchFormatter;

impl BatchFormatter {
    /// Format diagnostic batch for pretty output with comprehensive report
    pub fn format_comprehensive_report(
        engine: &DiagnosticEngine,
        batch: &DiagnosticBatch,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();
        
        // Header with emoji and statistics
        output.push_str(&Self::format_header(&batch.statistics));
        output.push('\n');

        if batch.diagnostics.is_empty() {
            output.push_str("✨ No issues found! Your FHIRPath expression is clean.\n");
            return Ok(output);
        }

        // Group diagnostics by severity for better organization
        let errors: Vec<_> = batch.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).collect();
        let warnings: Vec<_> = batch.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).collect();
        let suggestions: Vec<_> = batch.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Hint).collect(); // Mapped from suggestions
        let notes: Vec<_> = batch.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Info).collect(); // Mapped from notes

        // Display errors first (most critical)
        if !errors.is_empty() {
            output.push_str(&format!("❌ {} Error{}\n", errors.len(), if errors.len() == 1 { "" } else { "s" }));
            output.push_str(&"═".repeat(50));
            output.push('\n');
            for (i, diagnostic) in errors.iter().enumerate() {
                output.push_str(&format!("Error #{}\n", i + 1));
                output.push_str(&Self::format_single_diagnostic(engine, diagnostic, batch.source_id)?);
                output.push('\n');
            }
        }

        // Display warnings next
        if !warnings.is_empty() {
            output.push_str(&format!("⚠️  {} Warning{}\n", warnings.len(), if warnings.len() == 1 { "" } else { "s" }));
            output.push_str(&"═".repeat(50));
            output.push('\n');
            for (i, diagnostic) in warnings.iter().enumerate() {
                output.push_str(&format!("Warning #{}\n", i + 1));
                output.push_str(&Self::format_single_diagnostic(engine, diagnostic, batch.source_id)?);
                output.push('\n');
            }
        }

        // Display suggestions last
        if !suggestions.is_empty() {
            output.push_str(&format!("💡 {} Suggestion{}\n", suggestions.len(), if suggestions.len() == 1 { "" } else { "s" }));
            output.push_str(&"═".repeat(50));
            output.push('\n');
            for (i, diagnostic) in suggestions.iter().enumerate() {
                output.push_str(&format!("Suggestion #{}\n", i + 1));
                output.push_str(&Self::format_single_diagnostic(engine, diagnostic, batch.source_id)?);
                output.push('\n');
            }
        }

        // Display notes if any
        if !notes.is_empty() {
            output.push_str(&format!("ℹ️  {} Note{}\n", notes.len(), if notes.len() == 1 { "" } else { "s" }));
            output.push_str(&"═".repeat(50));
            output.push('\n');
            for (i, diagnostic) in notes.iter().enumerate() {
                output.push_str(&format!("Note #{}\n", i + 1));
                output.push_str(&Self::format_single_diagnostic(engine, diagnostic, batch.source_id)?);
                output.push('\n');
            }
        }

        // Summary footer
        output.push_str(&Self::format_summary(&batch.statistics));
        
        Ok(output)
    }

    /// Format diagnostic batch for table output
    pub fn format_table_report(
        batch: &DiagnosticBatch,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use tabled::{Table, Tabled, settings::Style};

        #[derive(Tabled)]
        struct DiagnosticRow {
            #[tabled(rename = "Type")]
            severity: String,
            #[tabled(rename = "Code")]
            error_code: String,
            #[tabled(rename = "Span")]
            span: String,
            #[tabled(rename = "Message")]
            message: String,
        }

        let rows: Vec<DiagnosticRow> = batch.diagnostics.iter().map(|d| {
            DiagnosticRow {
                severity: match d.severity {
                    DiagnosticSeverity::Error => "❌ ERROR".to_string(),
                    DiagnosticSeverity::Warning => "⚠️  WARN".to_string(),
                    DiagnosticSeverity::Hint => "💡 SUGGEST".to_string(),
                    DiagnosticSeverity::Info => "ℹ️  NOTE".to_string(),
                },
                error_code: d.error_code.code_str().to_string(),
                span: format!("{}..{}", d.span.start, d.span.end),
                message: d.message.clone(),
            }
        }).collect();

        let mut output = String::new();
        output.push_str(&Self::format_header(&batch.statistics));
        output.push('\n');

        if !rows.is_empty() {
            let table = Table::new(&rows)
                .with(Style::modern())
                .to_string();
            output.push_str(&table);
            output.push('\n');
        }

        output.push_str(&Self::format_summary(&batch.statistics));
        Ok(output)
    }

    /// Format diagnostic batch for JSON output
    pub fn format_json_report(batch: &DiagnosticBatch) -> Value {
        json!({
            "source": {
                "id": batch.source_id,
                "name": batch.source_name
            },
            "statistics": {
                "total": batch.statistics.total_count,
                "errors": batch.statistics.error_count,
                "warnings": batch.statistics.warning_count,
                "suggestions": batch.statistics.suggestion_count,
                "notes": batch.statistics.note_count
            },
            "diagnostics": batch.diagnostics.iter().map(|d| {
                DiagnosticFormatter::format_json(d)
            }).collect::<Vec<_>>(),
            "has_errors": batch.statistics.error_count > 0,
            "has_warnings": batch.statistics.warning_count > 0,
            "success": batch.statistics.error_count == 0
        })
    }

    /// Format single diagnostic within batch context
    fn format_single_diagnostic(
        engine: &DiagnosticEngine,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        DiagnosticFormatter::format_pretty(engine, diagnostic, source_id)
    }

    /// Format report header with statistics
    fn format_header(stats: &DiagnosticStatistics) -> String {
        if stats.total_count == 0 {
            return "📋 FHIRPath Analysis Report - No Issues Found".to_string();
        }

        format!(
            "📋 FHIRPath Analysis Report\n{}\nFound {} issue{}: {} error{}, {} warning{}, {} suggestion{}",
            "=".repeat(50),
            stats.total_count,
            if stats.total_count == 1 { "" } else { "s" },
            stats.error_count,
            if stats.error_count == 1 { "" } else { "s" },
            stats.warning_count,
            if stats.warning_count == 1 { "" } else { "s" },
            stats.suggestion_count,
            if stats.suggestion_count == 1 { "" } else { "s" }
        )
    }

    /// Format report summary footer
    fn format_summary(stats: &DiagnosticStatistics) -> String {
        let mut summary = String::new();
        summary.push_str(&"=".repeat(50));
        summary.push('\n');

        if stats.error_count > 0 {
            summary.push_str("🔴 Analysis failed due to errors. Please fix the errors above.\n");
        } else if stats.warning_count > 0 {
            summary.push_str("🟡 Analysis completed with warnings. Consider addressing the warnings above.\n");
        } else if stats.suggestion_count > 0 {
            summary.push_str("🟢 Analysis completed successfully with optimization suggestions.\n");
        } else {
            summary.push_str("🟢 Analysis completed successfully with no issues!\n");
        }

        // Add helpful links
        summary.push_str("\n📚 For more information about error codes, visit:\n");
        summary.push_str("    https://octofhir.github.io/fhirpath-rs/errors/\n");

        summary
    }

    /// Format compact single-line summary
    pub fn format_compact_summary(stats: &DiagnosticStatistics) -> String {
        if stats.total_count == 0 {
            return "✨ No issues found".to_string();
        }

        format!(
            "📊 {} total: {}❌ {}⚠️  {}💡",
            stats.total_count,
            stats.error_count,
            stats.warning_count, 
            stats.suggestion_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_code::*;
    use crate::diagnostics::collector::MultiDiagnosticCollector;

    #[test]
    fn test_comprehensive_report_formatting() {
        let engine = DiagnosticEngine::new();
        let mut collector = MultiDiagnosticCollector::new();
        
        collector.error(FP0001, "Syntax error".to_string(), 0..5);
        collector.warning(FP0153, "Performance warning".to_string(), 10..15);
        collector.suggestion(FP0154, "Optimization suggestion".to_string(), 20..25);
        
        let batch = collector.build_batch(0, "test".to_string());
        let formatted = BatchFormatter::format_comprehensive_report(&engine, &batch).unwrap();
        
        assert!(formatted.contains("📋 FHIRPath Analysis Report"));
        assert!(formatted.contains("❌ 1 Error"));
        assert!(formatted.contains("⚠️  1 Warning"));
        assert!(formatted.contains("💡 1 Suggestion"));
        assert!(formatted.contains("octofhir.github.io"));
    }

    #[test]
    fn test_json_report_formatting() {
        let mut collector = MultiDiagnosticCollector::new();
        collector.error(FP0001, "Test error".to_string(), 0..5);
        collector.warning(FP0153, "Test warning".to_string(), 10..15);
        
        let batch = collector.build_batch(0, "test.fhirpath".to_string());
        let json = BatchFormatter::format_json_report(&batch);
        
        assert_eq!(json["source"]["name"], "test.fhirpath");
        assert_eq!(json["statistics"]["total"], 2);
        assert_eq!(json["statistics"]["errors"], 1);
        assert_eq!(json["statistics"]["warnings"], 1);
        assert_eq!(json["has_errors"], true);
        assert_eq!(json["success"], false);
    }

    #[test]
    fn test_compact_summary() {
        let stats = DiagnosticStatistics {
            error_count: 2,
            warning_count: 3,
            suggestion_count: 1,
            note_count: 1,
            total_count: 7,
        };
        
        let summary = BatchFormatter::format_compact_summary(&stats);
        assert!(summary.contains("📊 7 total"));
        assert!(summary.contains("2❌"));
        assert!(summary.contains("3⚠️"));
        assert!(summary.contains("1💡"));
    }

    #[test]
    fn test_empty_batch() {
        let engine = DiagnosticEngine::new();
        let collector = MultiDiagnosticCollector::new();
        let batch = collector.build_batch(0, "test.fhirpath".to_string());
        
        let formatted = BatchFormatter::format_comprehensive_report(&engine, &batch).unwrap();
        assert!(formatted.contains("✨ No issues found!"));
    }
}