//! Reuse compiled patterns across calls, including calls inside FHIRPath lambdas.
use std::sync::{Arc, LazyLock};

use moka::sync::Cache;
use regex::Regex;

type CompiledRegex = Result<Arc<Regex>, Arc<regex::Error>>;

// Bound retained patterns, including invalid patterns. Long user-supplied patterns
// remain supported but are not retained. Regex's existing compilation limits apply.
const MAX_PATTERNS: u64 = 64;
const MAX_CACHED_PATTERN_BYTES: usize = 4096;
static PATTERNS: LazyLock<Cache<String, CompiledRegex>> =
    LazyLock::new(|| Cache::new(MAX_PATTERNS));

pub(super) fn compile(pattern: &str) -> CompiledRegex {
    let compile = || Regex::new(pattern).map(Arc::new).map_err(Arc::new);
    if pattern.len() > MAX_CACHED_PATTERN_BYTES {
        return compile();
    }
    // Concurrent misses for the same pattern share one compilation. Hits borrow
    // the key and clone only an Arc, not a String or the regex's search scratch.
    PATTERNS.get_with_by_ref(pattern, compile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_compiled_patterns_and_errors() {
        let first = compile("cache-test-[0-9]+").unwrap();
        let second = compile("cache-test-[0-9]+").unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert!(first.is_match("cache-test-42"));
        let first = compile("[cache-test").unwrap_err();
        let second = compile("[cache-test").unwrap_err();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(
            first.to_string(),
            Regex::new(std::hint::black_box("[cache-test"))
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn long_patterns_are_supported_without_retention() {
        let pattern = "a".repeat(MAX_CACHED_PATTERN_BYTES + 1);
        let first = compile(&pattern).unwrap();
        let second = compile(&pattern).unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(first.is_match(&pattern));
    }

    #[tokio::test]
    async fn cached_regex_functions_preserve_results() {
        let engine = crate::testing::test_engine().await;
        let context = crate::testing::test_context();
        for _ in 0..2 {
            for (expression, expected) in [
                ("'abc123xyz'.matches('[0-9]+')", serde_json::json!(true)),
                (
                    "'abc123xyz'.matchesFull('[0-9]+')",
                    serde_json::json!(false),
                ),
                ("'123'.matchesFull('[0-9]+')", serde_json::json!(true)),
                ("'abc'.matches('')", serde_json::json!(true)),
                ("'abc'.matches({})", serde_json::Value::Null),
                (
                    "'abc123'.replaceMatches('([0-9]+)', '[$1]')",
                    serde_json::json!("abc[123]"),
                ),
                ("'abc'.replaceMatches('', 'x')", serde_json::json!("abc")),
            ] {
                let result = engine.evaluate(expression, &context).await.unwrap();
                assert_eq!(result.value.to_json_value(), expected, "{expression}");
            }
            assert!(
                engine
                    .evaluate("'abc'.matches('[')", &context)
                    .await
                    .is_err()
            );
        }
    }
}
