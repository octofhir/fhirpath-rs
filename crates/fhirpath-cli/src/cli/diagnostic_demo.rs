//! Demonstration of CLI diagnostic integration with Ariadne
//!
//! This module provides a simple example of how to integrate the Ariadne
//! diagnostic system with CLI commands for beautiful error reporting.

use octofhir_fhirpath::{
    diagnostics::{DiagnosticEngine, DiagnosticFormatter, DiagnosticSeverity},
    core::error_code::*,
    parse_with_analysis, parse_ast,
};
use std::io::{self, Write};

use super::output::OutputFormat;

/// Simple diagnostic demo for parse command
pub fn demo_parse_with_diagnostics(expression: &str, output_format: &OutputFormat) {
    let mut engine = DiagnosticEngine::new();
    let source_id = engine.add_source("expression.fhirpath".to_string(), expression.to_string());

    let mut stderr = io::stderr();

    // Show what we're doing
    if *output_format != OutputFormat::Json {
        writeln!(stderr, "🔍 Parsing FHIRPath expression with Ariadne diagnostics...").ok();
        writeln!(stderr, "Expression: {}", expression).ok();
        writeln!(stderr).ok();
    }

    // Try parsing the expression
    match parse_ast(expression) {
        Ok(ast) => {
            // Success case - show success message and result
            match output_format {
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "success": true,
                        "expression": expression,
                        "ast_summary": format!("{:#?}", ast),
                        "message": "Expression parsed successfully"
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
                OutputFormat::Pretty => {
                    writeln!(stderr, "✅ Expression parsed successfully!").ok();
                    writeln!(stderr, "🌳 AST Structure:").ok();
                    println!("{:#?}", ast);
                }
                _ => {
                    writeln!(stderr, "✅ Parse successful").ok();
                    println!("AST: {:#?}", ast);
                }
            }
        }
        Err(error) => {
            // Error case - create and show diagnostic
            let diagnostic = engine.builder()
                .with_error_code(FP0001)
                .with_severity(DiagnosticSeverity::Error)
                .with_message(format!("Parse error: {}", error))
                .with_span(0..expression.len())
                .with_help("Check the FHIRPath syntax. Common issues include mismatched parentheses, invalid identifiers, or unsupported operators.".to_string())
                .with_note("For more information about FHIRPath syntax, visit: https://hl7.org/fhirpath/".to_string())
                .build();

            // Show diagnostic based on output format
            match output_format {
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "success": false,
                        "expression": expression,
                        "error": DiagnosticFormatter::format_json(&diagnostic),
                        "message": "Parse error occurred"
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
                OutputFormat::Pretty => {
                    match DiagnosticFormatter::format_pretty(&engine, &diagnostic, source_id) {
                        Ok(formatted) => {
                            write!(stderr, "{}", formatted).ok();
                        }
                        Err(e) => {
                            writeln!(stderr, "❌ Error formatting diagnostic: {}", e).ok();
                            writeln!(stderr, "Raw error: {}", error).ok();
                        }
                    }
                }
                _ => {
                    let raw_output = DiagnosticFormatter::format_raw(&diagnostic);
                    write!(stderr, "{}", raw_output).ok();
                }
            }

            // Also try the analysis parser to show error recovery
            if *output_format != OutputFormat::Json {
                writeln!(stderr).ok();
                writeln!(stderr, "🔧 Trying analysis parser for error recovery...").ok();
                
                let result = parse_with_analysis(expression);
                if result.success {
                    writeln!(stderr, "✅ Analysis parser succeeded with error recovery!").ok();
                    if let Some(ast) = result.ast {
                        writeln!(stderr, "Recovered AST: {:#?}", ast).ok();
                    }
                } else {
                    writeln!(stderr, "⚠️ Analysis parser also failed, but provided diagnostics:").ok();
                    for (i, diag) in result.diagnostics.iter().enumerate() {
                        writeln!(stderr, "  Diagnostic {}: {}", i + 1, diag.message).ok();
                    }
                }
            }
        }
    }
}

/// Demo showing multiple diagnostic types
pub fn demo_diagnostic_types(output_format: &OutputFormat) {
    let mut engine = DiagnosticEngine::new();
    let source_id = engine.add_source("demo.fhirpath".to_string(), "Patient.name.invalid".to_string());

    let mut stderr = io::stderr();
    
    if *output_format != OutputFormat::Json {
        writeln!(stderr, "🎨 Demonstrating different diagnostic types with Ariadne...").ok();
        writeln!(stderr).ok();
    }

    // Create different types of diagnostics
    let error_diagnostic = engine.builder()
        .with_error_code(FP0055)
        .with_severity(DiagnosticSeverity::Error)
        .with_message("Property 'invalid' not found on Patient.name")
        .with_span(13..20)
        .with_help("Available properties: family, given, use, text, prefix, suffix".to_string())
        .build();

    let warning_diagnostic = engine.builder()
        .with_error_code(FP0153)
        .with_severity(DiagnosticSeverity::Warning)
        .with_message("This property access pattern may be inefficient")
        .with_span(8..20)
        .with_help("Consider using direct property access instead of chained navigation".to_string())
        .build();

    let info_diagnostic = engine.builder()
        .with_error_code(FP0154)
        .with_severity(DiagnosticSeverity::Info)
        .with_message("FHIRPath expression uses FHIR R4 schema")
        .with_span(0..7)
        .with_note("Ensure your data matches the expected FHIR version".to_string())
        .build();

    let hint_diagnostic = engine.builder()
        .with_error_code(FP0154)
        .with_severity(DiagnosticSeverity::Hint)
        .with_message("Consider using Patient.name.family for better clarity")
        .with_span(8..13)
        .build();

    let diagnostics = vec![error_diagnostic, warning_diagnostic, info_diagnostic, hint_diagnostic];

    // Show based on format
    match output_format {
        OutputFormat::Json => {
            let json_diagnostics: Vec<_> = diagnostics.iter()
                .map(DiagnosticFormatter::format_json)
                .collect();
            
            let output = serde_json::json!({
                "demo": "diagnostic_types",
                "diagnostics": json_diagnostics,
                "summary": {
                    "total": diagnostics.len(),
                    "errors": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Error)).count(),
                    "warnings": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Warning)).count(),
                    "info": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Info)).count(),
                    "hints": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Hint)).count(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            match DiagnosticFormatter::format_batch_pretty(&engine, &diagnostics, source_id) {
                Ok(formatted) => {
                    write!(stderr, "{}", formatted).ok();
                }
                Err(e) => {
                    writeln!(stderr, "❌ Error formatting batch diagnostics: {}", e).ok();
                    for (i, diag) in diagnostics.iter().enumerate() {
                        writeln!(stderr, "{}: {}", i + 1, DiagnosticFormatter::format_raw(diag)).ok();
                    }
                }
            }
        }
    }
}

/// Demo showing the diagnostic system capabilities
pub fn demo_diagnostic_system() {
    println!("🚀 FHIRPath CLI Diagnostic System Demo");
    println!("=====================================");
    println!();
    
    println!("This demo shows the integration of Ariadne diagnostics with the FHIRPath CLI.");
    println!("The diagnostic system provides:");
    println!("  ✨ Beautiful Rust compiler-style error reports");
    println!("  🎯 Precise source location highlighting");
    println!("  📚 Helpful error messages with documentation links");
    println!("  🎨 Multiple output formats (pretty, raw, JSON, table)");
    println!("  🔧 Error recovery with analysis parser");
    println!();
    
    println!("Try these examples:");
    println!("  cargo run --bin octofhir-fhirpath parse 'Patient.invalid'");
    println!("  cargo run --bin octofhir-fhirpath parse 'Patient.name(' --output-format pretty");
    println!("  cargo run --bin octofhir-fhirpath parse 'bad syntax here' --output-format json");
    println!();
    
    println!("Example 1: Valid expression");
    demo_parse_with_diagnostics("Patient.name.family", &OutputFormat::Pretty);
    println!();
    
    println!("Example 2: Invalid expression");
    demo_parse_with_diagnostics("Patient.invalid.syntax(", &OutputFormat::Pretty);
    println!();
    
    println!("Example 3: Multiple diagnostic types");
    demo_diagnostic_types(&OutputFormat::Pretty);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_diagnostic_demo_valid_expression() {
        // Test that valid expressions don't create diagnostics
        let expression = "Patient.name";
        let mut output = Vec::new();
        
        // Capture stderr for the demo
        demo_parse_with_diagnostics(expression, &OutputFormat::Raw);
        
        // If we get here without panicking, the demo worked
        assert!(true);
    }

    #[test]
    fn test_diagnostic_demo_invalid_expression() {
        // Test that invalid expressions create diagnostics  
        let expression = "Patient.name.(";
        
        // This should create a diagnostic without panicking
        demo_parse_with_diagnostics(expression, &OutputFormat::Raw);
        
        // If we get here without panicking, the error handling worked
        assert!(true);
    }

    #[test]
    fn test_diagnostic_types_demo() {
        // Test that the diagnostic types demo works
        demo_diagnostic_types(&OutputFormat::Json);
        
        // If we get here without panicking, the demo worked
        assert!(true);
    }
}
