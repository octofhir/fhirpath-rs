//! Causes of known differences against the unmodified upstream snapshots.
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};

/// An explained difference, not a replacement for the test's actual result.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailureClassification {
    /// Implementation, test, specification, integration or infrastructure.
    pub cause: String,
    /// What differs and why.
    pub note: String,
    /// Supporting specification or upstream issue.
    pub reference: String,
}

static CLASSIFICATIONS: LazyLock<HashMap<String, FailureClassification>> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/official/failure-causes.json"
    )))
    .expect("valid checked-in conformance classifications")
});

/// Look up a cause by the original suite and case name.
pub fn classify_failure(suite: &str, name: &str) -> Option<&'static FailureClassification> {
    CLASSIFICATIONS.get(&format!("{suite}/{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_classification_has_a_cause_explanation_and_reference() {
        assert!(!CLASSIFICATIONS.is_empty());
        for (name, entry) in CLASSIFICATIONS.iter() {
            assert!(name.contains('/'));
            assert!(
                [
                    "implementation",
                    "test",
                    "specification",
                    "integration",
                    "infrastructure"
                ]
                .contains(&entry.cause.as_str())
            );
            assert!(!entry.note.is_empty());
            assert!(entry.reference.starts_with("https://"));
        }
    }
    #[test]
    fn unknown_failures_are_not_silently_declared_expected() {
        assert!(classify_failure("new-suite", "new-regression").is_none());
    }
}
