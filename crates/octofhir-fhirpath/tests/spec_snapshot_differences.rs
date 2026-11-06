//! FHIRPath 3.x contracts that differ from older official snapshot expectations.
use octofhir_fhirpath::testing::{test_context, test_engine};

#[tokio::test]
async fn boundaries_preserve_date_type_and_distinct_uses_true_equality_only() {
    let engine = test_engine().await;
    let context = test_context();
    for expression in [
        "@2014.lowBoundary(6) is System.Date",
        "@2014.highBoundary(6) is System.Date",
        "@2020.combine(@2020-01-01).isDistinct()",
        "@2020.combine(@2020-01-01).distinct().count() = 2",
        "1 year > 2 months",
        "(1 year <= 2 days) = false",
        "(1 year < 2 months) = false",
    ] {
        let result = engine.evaluate(expression, &context).await.unwrap();
        assert_eq!(
            result.value.first().and_then(|v| v.as_boolean()),
            Some(true),
            "{expression}"
        );
    }
}

#[tokio::test]
async fn singleton_evaluation_unary_operators_and_time_precision() {
    let engine = test_engine().await;
    let context = test_context();
    for expression in [
        "iif(1, 'true', 'false') = 'true'",
        "iif(0, 'true', 'false') = 'true'",
        "iif('hi', 'true', 'false') = 'true'",
        "iif({}, 'true', 'false') = 'false'",
        "+(1 + 1) = 2",
        "(@T10 = @T11:00) = false",
        "(@T10:30 = @T10:31:00) = false",
        "(@T10 = @T10:00).empty()",
        "(@T10:00 = @T10:00:00).empty()",
        "@T10:00:00 = @T10:00:00.000",
    ] {
        let result = engine.evaluate(expression, &context).await.unwrap();
        assert_eq!(
            result.value.first().and_then(|v| v.as_boolean()),
            Some(true),
            "{expression}"
        );
    }
}

#[tokio::test]
async fn several_items_are_an_error_for_single_item_operations() {
    let engine = test_engine().await;
    let context = test_context();
    for expression in [
        "iif(1 | 2, 'a', 'b')",
        "-(7.combine(3))",
        "+(7.combine(3))",
        "(1 | 2).toBoolean()",
        "(1 | 2).toInteger()",
        "(1 | 2).toDecimal()",
        "(1 | 2).toString()",
        "(1 | 2).convertsToBoolean()",
        "(1 | 2).convertsToInteger()",
        "(1 | 2).convertsToDecimal()",
        "(1 | 2).convertsToString()",
        "(1 | 2).convertsToDate()",
        "(1 | 2).convertsToDateTime()",
        "(1 | 2).convertsToTime()",
    ] {
        assert!(
            engine.evaluate(expression, &context).await.is_err(),
            "{expression}"
        );
    }
}
