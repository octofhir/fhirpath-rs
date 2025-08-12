// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! String interning for FHIRPath values
//!
//! This module provides string interning to reduce memory allocation overhead
//! for commonly used strings like property names, function names, and literals.

use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe string interner using Arc for shared ownership
pub struct StringInterner {
    /// Map from string content to interned Arc
    cache: DashMap<String, Arc<str>>,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Intern a string, returning a shared `Arc<str>`
    /// Uses similar pattern to tokenizer STRING_INTERNER for consistency
    pub fn intern<S: AsRef<str>>(&self, s: S) -> Arc<str> {
        let s_ref = s.as_ref();

        // Fast path: check if already interned
        if let Some(interned) = self.cache.get(s_ref) {
            return Arc::clone(&interned);
        }

        // Slow path: intern the string with optimizations similar to tokenizer
        // Only intern strings that are likely to be reused (reasonable length, simple content)
        if self.should_intern(s_ref) {
            let owned = s_ref.to_string();
            let interned: Arc<str> = Arc::from(owned.as_str());

            // Insert and return, handling potential race conditions
            match self.cache.entry(owned) {
                dashmap::mapref::entry::Entry::Occupied(entry) => Arc::clone(entry.get()),
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    let result = Arc::clone(&interned);
                    entry.insert(interned);
                    result
                }
            }
        } else {
            // Don't intern very long or unusual strings to avoid memory bloat
            Arc::from(s_ref)
        }
    }

    /// Determine if a string should be interned (similar logic to tokenizer)
    #[inline]
    fn should_intern(&self, s: &str) -> bool {
        // Intern strings up to reasonable length that are likely identifiers/common values
        s.len() <= 64
            && (s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
                || self.is_common_fhir_pattern(s))
    }

    /// Check if string matches common FHIR patterns
    #[inline]
    fn is_common_fhir_pattern(&self, s: &str) -> bool {
        // URLs, URIs, and common FHIR values
        (s.starts_with("http://") && (s.contains("hl7.org") || s.contains("fhir")))
            || (s.starts_with("https://") && (s.contains("hl7.org") || s.contains("fhir")))
            || s.starts_with("urn:")
            || s.starts_with("fhir:")
            || s.contains("fhir.org")
            || s.contains("hl7.org")
    }

    /// Get statistics about the interner
    pub fn stats(&self) -> InternerStats {
        InternerStats {
            entries: self.cache.len(),
        }
    }

    /// Clear all interned strings (useful for testing)
    pub fn clear(&self) {
        self.cache.clear();
    }
}

/// Statistics about string interning performance
#[derive(Debug, Clone)]
pub struct InternerStats {
    /// Number of unique strings interned
    pub entries: usize,
}

/// Global string interner instance
static GLOBAL_INTERNER: once_cell::sync::Lazy<StringInterner> = once_cell::sync::Lazy::new(|| {
    let interner = StringInterner::new();

    // Pre-intern common FHIRPath strings for performance
    // Coordinate with tokenizer STRING_INTERNER patterns
    let common_strings = [
        // Common property names (aligned with tokenizer patterns)
        "id",
        "resourceType",
        "name",
        "value",
        "code",
        "system",
        "display",
        "given",
        "family",
        "use",
        "text",
        "status",
        "type",
        "url",
        "reference",
        "identifier",
        "extension",
        "coding",
        "telecom",
        "address",
        "contact",
        "active",
        "version",
        "date",
        "description",
        "subject",
        "encounter",
        "performer",
        "issued",
        "category",
        "component",
        "interpretation",
        // Common function names (matching tokenizer common_identifiers)
        "where",
        "select",
        "first",
        "last",
        "count",
        "length",
        "empty",
        "exists",
        "all",
        "any",
        "distinct",
        "flatten",
        "skip",
        "take",
        "single",
        "ofType",
        "hasValue",
        "trace",
        "toString",
        "toInteger",
        "toDecimal",
        "substring",
        "startsWith",
        "endsWith",
        "matches",
        "replaceMatches",
        "join",
        "combine",
        // Common operators and keywords
        "and",
        "or",
        "not",
        "is",
        "as",
        "in",
        "contains",
        "memberOf",
        "subsetOf",
        "supersetOf",
        "implies",
        "xor",
        "div",
        "mod",
        // Common literals
        "true",
        "false",
        "null",
        "",
        // Common resource types (expanded for better coverage)
        "Patient",
        "Observation",
        "Encounter",
        "Condition",
        "Procedure",
        "DiagnosticReport",
        "Medication",
        "Bundle",
        "Organization",
        "Practitioner",
        "Location",
        "Device",
        "Specimen",
        "ValueSet",
        "CodeSystem",
        "StructureDefinition",
        "OperationOutcome",
        "Parameters",
        "Binary",
        "DocumentReference",
        "Composition",
        // Common FHIR URLs and systems
        "http://hl7.org/fhir",
        "http://terminology.hl7.org",
        "http://snomed.info/sct",
        "http://loinc.org",
        "http://unitsofmeasure.org",
        "urn:iso:std:iso:3166",
        "http://www.nlm.nih.gov/research/umls/rxnorm",
        "http://fdasis.nlm.nih.gov",
        // Common status values
        "active",
        "inactive",
        "pending",
        "completed",
        "cancelled",
        "on-hold",
        "stopped",
        "unknown",
        "final",
        "preliminary",
        "registered",
        "partial",
    ];

    for s in &common_strings {
        interner.intern(*s);
    }

    interner
});

/// Intern a string using the global interner
pub fn intern_string<S: AsRef<str>>(s: S) -> Arc<str> {
    GLOBAL_INTERNER.intern(s)
}

/// Get statistics from the global interner
pub fn global_interner_stats() -> InternerStats {
    GLOBAL_INTERNER.stats()
}

/// Get global interner statistics in tokenizer-compatible format
/// Returns (entries, estimated_capacity) similar to tokenizer interner_stats()
pub fn global_interner_stats_compat() -> (usize, usize) {
    let stats = GLOBAL_INTERNER.stats();
    (stats.entries, stats.entries * 2) // Estimate capacity like DashMap
}

/// Check if a string is already interned globally (useful for debugging)
pub fn is_interned<S: AsRef<str>>(s: S) -> bool {
    GLOBAL_INTERNER.cache.contains_key(s.as_ref())
}

/// Clear the global interner (useful for testing coordination)
pub fn clear_global_interner() {
    GLOBAL_INTERNER.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let interner = StringInterner::new();

        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");

        // Same string should return same Arc
        assert_eq!(s1.as_ptr(), s2.as_ptr());

        // Different strings should have different Arc
        assert_ne!(s1.as_ptr(), s3.as_ptr());

        assert_eq!(s1.as_ref(), "hello");
        assert_eq!(s3.as_ref(), "world");
    }

    #[test]
    fn test_global_interner() {
        clear_global_interner(); // Start clean to avoid interference with other tests

        let s1 = intern_string("test");
        let s2 = intern_string("test");

        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(s1.as_ref(), "test");
    }

    #[test]
    fn test_interner_stats() {
        let interner = StringInterner::new();
        assert_eq!(interner.stats().entries, 0);

        interner.intern("a");
        interner.intern("b");
        interner.intern("a"); // Should not increase count

        assert_eq!(interner.stats().entries, 2);
    }

    #[test]
    fn test_should_intern_patterns() {
        let interner = StringInterner::new();

        // Should intern simple identifiers
        assert!(interner.should_intern("resourceType"));
        assert!(interner.should_intern("patient_name"));
        assert!(interner.should_intern("value.code"));

        // Should intern FHIR URLs
        assert!(interner.should_intern("http://hl7.org/fhir/Patient"));
        assert!(interner.should_intern("https://terminology.hl7.org/CodeSystem/test"));

        // Should NOT intern very long strings
        let long_string = "a".repeat(100);
        assert!(!interner.should_intern(&long_string));

        // Should NOT intern strings with unusual characters
        assert!(!interner.should_intern("test@#$%"));
    }

    #[test]
    fn test_coordination_functions() {
        clear_global_interner(); // Reset for test

        let _s1 = intern_string("test_coord");
        assert!(is_interned("test_coord"));
        assert!(!is_interned("not_interned"));

        let stats = global_interner_stats_compat();
        assert!(stats.0 > 0); // Should have at least the entry we just added
        assert!(stats.1 > 0); // Should have estimated capacity
    }

    #[test]
    fn test_fhir_pattern_recognition() {
        let interner = StringInterner::new();

        // FHIR URLs should be recognized
        assert!(interner.is_common_fhir_pattern("http://hl7.org/fhir/Patient"));
        assert!(interner.is_common_fhir_pattern("https://terminology.hl7.org/test"));
        assert!(interner.is_common_fhir_pattern("urn:oid:1.2.3.4"));

        // Non-FHIR patterns should not match
        assert!(!interner.is_common_fhir_pattern("regular_identifier"));
        assert!(!interner.is_common_fhir_pattern("http://example.com"));
    }
}
