//! The debug trace of an evaluation, in the events of the FHIR Mapping Language IG format.

use octofhir_fhirpath::{
    Collection, DebugLimits, DebugPhase, DebugStatus, DebugTrace, EvaluationContext, FhirPathValue,
    evaluator::create_engine_with_mock_provider,
};
use serde_json::json;

async fn trace(expression: &str, limits: DebugLimits) -> DebugTrace {
    let engine = create_engine_with_mock_provider().await.unwrap();
    let patient = json!({"resourceType": "Patient", "name": [
        {"family": "Doe", "given": ["Ann", "Bea"]},
        {"family": "Roe"},
    ]});
    let context = EvaluationContext::new(
        Collection::from(vec![FhirPathValue::resource(patient)]),
        engine.get_model_provider(),
        None,
        None,
        None,
    );
    engine.evaluate_debug(expression, &context, limits).await.1
}

fn result_of_outermost(trace: &DebugTrace) -> Vec<serde_json::Value> {
    let last = trace.events.last().unwrap();
    assert_eq!(last.phase, DebugPhase::Exit);
    assert_eq!(last.depth, 0);
    last.variables
        .iter()
        .filter_map(|variable| variable.value.clone())
        .collect()
}

#[tokio::test]
async fn nodes_enter_and_exit_at_their_depth() {
    let trace = trace("name.family", DebugLimits::default()).await;
    assert_eq!(trace.status, DebugStatus::Complete);
    assert_eq!(trace.events[0].phase, DebugPhase::Enter);
    assert_eq!(trace.events[0].depth, 0);
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|e| e.phase == DebugPhase::Enter)
            .count(),
        trace
            .events
            .iter()
            .filter(|e| e.phase == DebugPhase::Exit)
            .count(),
    );
    assert!(trace.events.iter().any(|event| event.depth > 0));
    assert_eq!(result_of_outermost(&trace), [json!("Doe"), json!("Roe")]);
}

#[tokio::test]
async fn a_lambda_is_traced_for_each_item() {
    let trace = trace("name.where(family = 'Roe').family", DebugLimits::default()).await;
    assert_eq!(result_of_outermost(&trace), [json!("Roe")]);
    let comparisons = trace
        .events
        .iter()
        .filter(|event| {
            event.phase == DebugPhase::Enter
                && event
                    .range
                    .is_some_and(|range| (range.start_column, range.end_column) == (11, 25))
        })
        .filter(|event| event.variables.iter().any(|v| v.name == "$index"))
        .count();
    assert_eq!(comparisons, 2);
}

#[tokio::test]
async fn positions_are_utf16_columns_and_rows_from_zero() {
    let trace = trace("'😀é'.length() +\n  1", DebugLimits::default()).await;
    let ranges: Vec<_> = trace
        .events
        .iter()
        .filter_map(|event| event.range)
        .collect();
    let literal = ranges
        .iter()
        .find(|range| range.start_row == 1)
        .expect("the literal on the second row");
    assert_eq!((literal.start_column, literal.end_column), (2, 3));
    // The emoji takes two code units, the quotes and é one each.
    assert!(
        ranges
            .iter()
            .any(|range| range.start_column == 0 && range.end_column == 5)
    );
}

#[tokio::test]
async fn positions_are_in_the_expression_as_written() {
    // The parser reads this without the comment, and with one space and a decoded entity.
    let trace = trace(
        "name  /* all */ .family\n  .count() &gt; 1",
        DebugLimits::default(),
    )
    .await;
    let ranges: Vec<_> = trace
        .events
        .iter()
        .filter_map(|event| event.range)
        .collect();
    assert!(
        ranges.iter().any(|range| {
            (
                range.start_row,
                range.start_column,
                range.end_row,
                range.end_column,
            ) == (1, 3, 1, 8)
        }),
        "{ranges:?}"
    );
    assert!(
        ranges
            .iter()
            .any(|range| (range.start_row, range.start_column) == (1, 16)),
        "{ranges:?}"
    );
}

#[tokio::test]
async fn trace_adds_a_step_with_its_message() {
    let trace = trace("name.family.trace('names')", DebugLimits::default()).await;
    let steps: Vec<_> = trace
        .events
        .iter()
        .filter(|event| event.phase == DebugPhase::Step)
        .collect();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].messages[0].name, "names");
    assert_eq!(steps[0].messages[0].text, "'Doe'");
}

#[tokio::test]
async fn an_empty_result_has_a_variable_without_a_value() {
    let trace = trace("name.nothing", DebugLimits::default()).await;
    let last = trace.events.last().unwrap();
    assert_eq!(last.variables.len(), 1);
    assert_eq!(last.variables[0].name, "result");
    assert_eq!(last.variables[0].value, None);
}

#[tokio::test]
async fn a_failure_stops_the_trace_with_scopes_left_open() {
    let trace = trace(
        "name.family.first() + (1 / {}).toInteger() + 'a'.foo()",
        DebugLimits::default(),
    )
    .await;
    assert_eq!(trace.status, DebugStatus::Stopped);
    assert!(trace.note.is_some());
    assert!(
        trace
            .events
            .iter()
            .filter(|e| e.phase == DebugPhase::Enter)
            .count()
            > trace
                .events
                .iter()
                .filter(|e| e.phase == DebugPhase::Exit)
                .count()
    );
}

#[tokio::test]
async fn the_limits_cut_the_events_and_not_the_evaluation() {
    let limits = DebugLimits {
        max_events: 3,
        ..DebugLimits::default()
    };
    let trace = trace("name.family", limits).await;
    assert_eq!(trace.events.len(), 3);
    assert_eq!(trace.status, DebugStatus::Truncated);
}

#[tokio::test]
async fn the_parameters_follow_the_format() {
    let trace = trace("name.family.first()", DebugLimits::default()).await;
    let parameters = trace.parameters();
    assert_eq!(
        parameters[0],
        json!({"name": "traceStatus", "valueCode": "complete"})
    );
    assert_eq!(parameters.len(), trace.events.len() + 1);
    let last = parameters.last().unwrap();
    assert_eq!(last["name"], "trace");
    let part = |name: &str| {
        last["part"]
            .as_array()
            .unwrap()
            .iter()
            .find(|part| part["name"] == name)
            .cloned()
            .unwrap()
    };
    assert_eq!(part("phase")["valueCode"], "exit");
    assert_eq!(part("depth")["valueInteger"], 0);
    assert_eq!(part("startRow")["valueInteger"], 0);
    let variable = part("variable");
    assert_eq!(variable["part"][0]["valueString"], "result");
    assert_eq!(variable["part"][2]["valueString"], "Doe");
}
