//! The CLI/LSP analyzer must apply the same focus type guards as the dedicated API.

use std::sync::Arc;

use octofhir_fhir_model::{EmptyModelProvider, TypeInfo};
use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalysisResult, StaticAnalyzer};

async fn analyze(expression: &str, name: &str) -> StaticAnalysisResult {
    // Schema-backed providers retain the FHIR name even when the mapped type is Any.
    let context = AnalysisContext::new(TypeInfo {
        type_name: "Any".into(),
        singleton: Some(true),
        namespace: Some("FHIR".into()),
        name: Some(name.into()),
        is_empty: Some(false),
    });
    StaticAnalyzer::new(Arc::new(EmptyModelProvider))
        .analyze_expression(expression, context)
        .await
}

#[tokio::test]
async fn unsupported_focus_conversion_is_reported_with_its_location() {
    let expression = "$this is Period implies $this.toString() = 'x'";
    let result = analyze(expression, "Period").await;
    assert!(!result.success);
    assert_eq!(result.statistics.errors_found, 1);
    let diagnostic = &result.diagnostics[0];
    assert!(diagnostic.message.contains("toString() is not defined"));
    let start = expression.find("toString").unwrap();
    assert_eq!(diagnostic.span, start..start + "toString".len());
}

#[tokio::test]
async fn boolean_guards_do_not_report_unreachable_conversions() {
    for expression in [
        "$this is dateTime implies $this.toString() = 'x'",
        "$this is Period or $this.toString() = 'x'",
        "iif($this is Period, true, $this.toString() = 'x')",
    ] {
        let result = analyze(expression, "Period").await;
        assert!(result.success, "{expression}: {:?}", result.diagnostics);
    }
}

#[tokio::test]
async fn reachable_conversion_in_an_iif_branch_is_checked() {
    let result = analyze("iif($this is Period, $this.toString(), 'x')", "Period").await;
    assert!(!result.success);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("toString() is not defined") })
    );
}

#[tokio::test]
async fn primitive_and_unknown_contexts_are_not_rejected() {
    for name in ["dateTime", "Any", "Unknown"] {
        let result = analyze("$this.toString()", name).await;
        assert!(result.success, "{name}: {:?}", result.diagnostics);
    }
}

#[tokio::test]
async fn iteration_focus_is_not_mistaken_for_the_root_focus() {
    let context = AnalysisContext::new(TypeInfo {
        type_name: "Any".into(),
        singleton: Some(false),
        namespace: Some("FHIR".into()),
        name: Some("Period".into()),
        is_empty: Some(false),
    });
    let result = StaticAnalyzer::new(Arc::new(EmptyModelProvider))
        .analyze_expression("$this.select($this.toString())", context)
        .await;
    assert!(result.success, "{:?}", result.diagnostics);
}

#[tokio::test]
async fn semantic_entry_points_share_guards_without_duplicating_union_diagnostics() {
    use octofhir_fhirpath::parser::{parse_with_semantic_analysis, parse_with_type_guard_analysis};
    let model = Arc::new(EmptyModelProvider);
    let focus = TypeInfo {
        type_name: "Any".into(),
        name: Some("Period".into()),
        namespace: Some("FHIR".into()),
        singleton: Some(true),
        is_empty: Some(false),
    };
    let single = parse_with_semantic_analysis("$this.toString()", model.clone(), Some(focus)).await;
    assert!(!single.analysis.success);
    assert_eq!(
        single
            .analysis
            .diagnostics
            .iter()
            .filter(|d| d.code.code == "TYPE_GUARD")
            .count(),
        1
    );
    let union = ["Period".to_string(), "dateTime".to_string()];
    let valid = parse_with_type_guard_analysis("$this.toString()", model.clone(), &union).await;
    assert!(valid.analysis.success, "{:?}", valid.analysis.diagnostics);
    let invalid = parse_with_type_guard_analysis(
        "$this is Period implies $this.toString() = 'x'",
        model,
        &union,
    )
    .await;
    assert!(!invalid.analysis.success);
    assert_eq!(
        invalid
            .analysis
            .diagnostics
            .iter()
            .filter(|d| d.code.code == "TYPE_GUARD")
            .count(),
        1
    );
}
