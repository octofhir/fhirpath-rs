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

    /// Intern a string, returning a shared Arc<str>
    pub fn intern<S: AsRef<str>>(&self, s: S) -> Arc<str> {
        let s_ref = s.as_ref();

        // Fast path: check if already interned
        if let Some(interned) = self.cache.get(s_ref) {
            return Arc::clone(&interned);
        }

        // Slow path: intern the string
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
    let common_strings = [
        // Common property names
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
        // Common function names
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
        // Common operators and keywords
        "and",
        "or",
        "not",
        "is",
        "as",
        "in",
        "contains",
        // Common literals
        "true",
        "false",
        "null",
        "",
        // Common resource types
        "Patient",
        "Observation",
        "Encounter",
        "Condition",
        "Procedure",
        "DiagnosticReport",
        "Medication",
        "Bundle",
        "Organization",
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
}
