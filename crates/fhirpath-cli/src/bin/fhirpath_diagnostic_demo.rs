//! Simple FHIRPath CLI with Ariadne diagnostic integration demo
//!
//! This demonstrates the integration of beautiful Ariadne diagnostics
//! with FHIRPath parsing and evaluation.

use clap::{Arg, ArgAction, Command};
use octofhir_fhirpath::{
    diagnostics::{AriadneDiagnostic, DiagnosticEngine, DiagnosticFormatter, DiagnosticSeverity},
    core::error_code::*,
    parse_ast, parse_with_analysis,
};
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutputFormat {
    Raw,
    Pretty,
    Json,
}

impl OutputFormat {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(Self::Raw),
            "pretty" => Ok(Self::Pretty), 
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

fn main() {
    let matches = Command::new("fhirpath-diagnostic-demo")
        .version("0.1.0")
        .about("FHIRPath CLI with Ariadne diagnostic integration demo")
        .arg(
            Arg::new("expression")
                .help("FHIRPath expression to parse")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output-format")
                .long("output-format")
                .short('o')
                .help("Output format: raw, pretty, json")
                .value_name("FORMAT")
                .default_value("pretty"),
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .help("Disable colored output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress informational messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("show-types")
                .long("show-types")
                .help("Show demonstration of different diagnostic types")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("demo-system")
                .long("demo-system")
                .help("Show diagnostic system capabilities")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let expression = matches.get_one::<String>("expression").unwrap();
    let output_format_str = matches.get_one::<String>("output-format").unwrap();
    let no_color = matches.get_flag("no-color");
    let quiet = matches.get_flag("quiet");
    let show_types = matches.get_flag("show-types");
    let demo_system = matches.get_flag("demo-system");

    // Set NO_COLOR if requested
    if no_color {
        std::env::set_var("FHIRPATH_NO_COLOR", "1");
    }

    let output_format = match OutputFormat::from_str(output_format_str) {
        Ok(format) => format,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    // Handle special demo modes
    if demo_system {
        demo_diagnostic_system(&output_format, quiet);
        return;
    }

    if show_types {
        demo_diagnostic_types(&output_format, quiet);
        return;
    }

    // Regular parsing with diagnostics
    parse_with_diagnostics(expression, &output_format, quiet);
}

fn parse_with_diagnostics(expression: &str, output_format: &OutputFormat, quiet: bool) {
    let mut engine = DiagnosticEngine::new();
    let source_id = engine.add_source("expression.fhirpath".to_string(), expression.to_string());

    let mut stderr = io::stderr();

    // Show what we're doing
    if *output_format != OutputFormat::Json && !quiet {
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
                    if !quiet {
                        writeln!(stderr, "✅ Expression parsed successfully!").ok();
                        writeln!(stderr, "🌳 AST Structure:").ok();
                    }
                    println!("{:#?}", ast);
                }
                OutputFormat::Raw => {
                    if !quiet {
                        writeln!(stderr, "✅ Parse successful").ok();
                    }
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
                OutputFormat::Raw => {
                    let raw_output = DiagnosticFormatter::format_raw(&diagnostic);
                    write!(stderr, "{}", raw_output).ok();
                }
            }

            // Also try the analysis parser to show error recovery
            if *output_format != OutputFormat::Json && !quiet {
                writeln!(stderr).ok();
                writeln!(stderr, "🔧 Trying analysis parser for error recovery...").ok();
                
                match parse_with_analysis(expression) {
                    Ok(result) => {
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
                    Err(e) => {
                        writeln!(stderr, "❌ Analysis parser also failed: {}", e).ok();
                    }
                }
            }
        }
    }
}

fn demo_diagnostic_types(output_format: &OutputFormat, quiet: bool) {
    let mut engine = DiagnosticEngine::new();
    let source_id = engine.add_source("demo.fhirpath".to_string(), "Patient.name.invalid".to_string());

    let mut stderr = io::stderr();
    
    if *output_format != OutputFormat::Json && !quiet {
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

fn demo_diagnostic_system(output_format: &OutputFormat, quiet: bool) {
    if *output_format == OutputFormat::Json {
        let demo_info = serde_json::json!({
            "demo": "diagnostic_system_overview",
            "description": "FHIRPath CLI Diagnostic System Demo", 
            "capabilities": [
                "Beautiful Rust compiler-style error reports",
                "Precise source location highlighting",
                "Helpful error messages with documentation links",
                "Multiple output formats (pretty, raw, JSON)",
                "Error recovery with analysis parser",
                "Environment variable support (NO_COLOR, FHIRPATH_NO_COLOR)"
            ],
            "examples": [
                "fhirpath-diagnostic-demo 'Patient.name.family'",
                "fhirpath-diagnostic-demo 'Patient.invalid' --output-format pretty",
                "fhirpath-diagnostic-demo 'bad syntax(' --output-format json",
                "fhirpath-diagnostic-demo 'Patient.name' --show-types"
            ]
        });
        println!("{}", serde_json::to_string_pretty(&demo_info).unwrap());
        return;
    }

    if !quiet {
        println!("🚀 FHIRPath CLI Diagnostic System Demo");
        println!("=====================================");
        println!();
        
        println!("This demo shows the integration of Ariadne diagnostics with the FHIRPath CLI.");
        println!("The diagnostic system provides:");
        println!("  ✨ Beautiful Rust compiler-style error reports");
        println!("  🎯 Precise source location highlighting");
        println!("  📚 Helpful error messages with documentation links");
        println!("  🎨 Multiple output formats (pretty, raw, JSON)");
        println!("  🔧 Error recovery with analysis parser");
        println!("  🌈 Environment variable support (NO_COLOR, FHIRPATH_NO_COLOR)");
        println!();
        
        println!("Try these examples:");
        println!("  fhirpath-diagnostic-demo 'Patient.name.family'");
        println!("  fhirpath-diagnostic-demo 'Patient.invalid' --output-format pretty");
        println!("  fhirpath-diagnostic-demo 'bad syntax(' --output-format json");
        println!("  fhirpath-diagnostic-demo 'Patient.name' --show-types");
        println!();
    }
    
    println!("Example 1: Valid expression");
    parse_with_diagnostics("Patient.name.family", output_format, quiet);
    println!();
    
    println!("Example 2: Invalid expression");
    parse_with_diagnostics("Patient.invalid.syntax(", output_format, quiet);
    println!();
    
    println!("Example 3: Multiple diagnostic types");
    demo_diagnostic_types(output_format, quiet);
}