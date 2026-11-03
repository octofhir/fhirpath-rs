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
