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

/// The pattern FHIRPath evaluates, as inline flags: always single-line mode (`.` matches a
/// newline), plus the optional `flags` of `matches`/`matchesFull`/`replaceMatches`
/// (`i` case-insensitive, `m` multi-line `^`/`$`).
pub(super) fn effective_pattern(pattern: &str, flags: &str) -> crate::core::Result<String> {
    let mut inline = String::from("s");
    for flag in flags.chars() {
        match flag {
            'i' | 'm' if !inline.contains(flag) => inline.push(flag),
            'i' | 'm' => {}
            other => {
                return Err(crate::core::FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!("Invalid regular expression flag '{other}' (expected 'i' or 'm')"),
                ));
            }
        }
    }
    Ok(format!("(?{inline}){pattern}"))
}

/// The optional flags argument at `index`: absent or empty means no flags; anything but a
/// single string is an error.
pub(super) fn flags_arg(
    args: &[crate::core::Collection],
    index: usize,
    function: &str,
) -> crate::core::Result<String> {
    match args.get(index) {
        None => Ok(String::new()),
        Some(arg) if arg.is_empty() => Ok(String::new()),
        Some(arg) => match (arg.len(), arg.first().and_then(|v| v.as_string())) {
            (1, Some(flags)) => Ok(flags.to_string()),
            _ => Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                format!("{function} flags argument must be a single string"),
            )),
        },
    }
}

/// Compile `pattern` (already passed through [`effective_pattern`]) for a plan, with the
/// size limit that keeps retained regexes bounded.
pub(super) fn compile_for_plan(pattern: &str) -> Option<Arc<Regex>> {
    if pattern.len() > MAX_CACHED_PATTERN_BYTES {
        return None;
    }
    regex::RegexBuilder::new(pattern)
        .size_limit(256 * 1024)
        .build()
        .ok()
        .map(Arc::new)
}

/// The cached compilation of `pattern` (as [`effective_pattern`] gives it).
fn compile_checked(pattern: &str) -> crate::core::Result<Arc<Regex>> {
    compile(pattern).map_err(|err| {
        crate::core::FhirPathError::evaluation_error(
            crate::core::error_code::FP0058,
            format!("Invalid regular expression pattern '{pattern}': {err}"),
        )
    })
}

/// A regex compiled when a plan was prepared, with the arguments it was compiled from.
#[derive(Debug)]
pub(super) struct PreparedRegex {
    pattern: String,
    flags: String,
    regex: Arc<Regex>,
}

impl PreparedRegex {
    /// Compile `effective` (the pattern as [`effective_pattern`] gives it) for `pattern` and
    /// `flags`, as written in the expression.
    pub(super) fn new(pattern: &str, flags: &str, effective: &str) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            pattern: pattern.to_string(),
            flags: flags.to_string(),
            regex: compile_for_plan(effective)?,
        }))
    }
}

/// The regex for `pattern` with `flags`: the prepared one when it was compiled for the same
/// arguments (no allocation), otherwise a cached compilation of `effective()`.
pub(super) fn regex_for(
    prepared: Option<&Arc<PreparedRegex>>,
    pattern: &str,
    flags: &str,
    effective: impl FnOnce() -> crate::core::Result<String>,
) -> crate::core::Result<Arc<Regex>> {
    if let Some(prepared) = prepared
        && prepared.pattern == pattern
        && prepared.flags == flags
    {
        return Ok(prepared.regex.clone());
    }
    compile_checked(&effective()?)
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
