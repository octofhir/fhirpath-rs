//! Expression interning for memory optimization and deduplication
//!
//! This module provides an interning system for ExpressionNode that allows
//! sharing of identical expressions across the AST, reducing memory usage
//! and improving cache locality.

use super::expression::ExpressionNode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// A reference-counted, interned expression node
///
/// This allows multiple parts of the AST to share the same ExpressionNode
/// without duplication, improving memory efficiency and cache performance.
#[derive(Debug, Clone, PartialEq)]
pub struct InternedExpr {
    inner: Arc<ExpressionNode>,
}

impl InternedExpr {
    /// Create a new interned expression from an ExpressionNode
    pub fn new(expr: ExpressionNode) -> Self {
        Self {
            inner: Arc::new(expr),
        }
    }

    /// Get a reference to the underlying expression
    pub fn as_expr(&self) -> &ExpressionNode {
        &self.inner
    }

    /// Get the reference count for this interned expression
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Check if this is a unique reference (not shared)
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.inner) == 1
    }

    /// Try to get a mutable reference if this is the only reference
    pub fn get_mut(&mut self) -> Option<&mut ExpressionNode> {
        Arc::get_mut(&mut self.inner)
    }

    /// Clone the underlying expression if we need to modify it
    pub fn make_mut(&mut self) -> &mut ExpressionNode {
        Arc::make_mut(&mut self.inner)
    }

    /// Check if two interned expressions share the same memory
    /// This is useful for verifying interning effectiveness
    pub fn shares_memory_with(&self, other: &InternedExpr) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl std::ops::Deref for InternedExpr {
    type Target = ExpressionNode;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<ExpressionNode> for InternedExpr {
    fn from(expr: ExpressionNode) -> Self {
        Self::new(expr)
    }
}

impl From<InternedExpr> for ExpressionNode {
    fn from(interned: InternedExpr) -> Self {
        // Try to avoid cloning if this is the only reference
        match Arc::try_unwrap(interned.inner) {
            Ok(expr) => expr,
            Err(arc) => (*arc).clone(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for InternedExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for InternedExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let expr = ExpressionNode::deserialize(deserializer)?;
        Ok(InternedExpr::new(expr))
    }
}

/// Global expression interner for deduplication
///
/// This provides a global cache of expressions to enable sharing of
/// identical subtrees across different ASTs.
pub struct ExpressionInterner {
    cache: HashMap<ExpressionNode, Arc<ExpressionNode>>,
    stats: InternerStats,
}

#[derive(Debug, Default)]
/// Statistics for expression interning performance
pub struct InternerStats {
    /// Total number of intern requests
    pub total_requests: usize,
    /// Number of cache hits (expressions found in cache)
    pub cache_hits: usize,
    /// Current size of the interning cache
    pub cache_size: usize,
    /// Estimated bytes saved through interning
    pub memory_saved_bytes: usize,
}

impl ExpressionInterner {
    /// Create a new expression interner
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: InternerStats::default(),
        }
    }

    /// Intern an expression, returning a shared reference
    pub fn intern(&mut self, expr: ExpressionNode) -> InternedExpr {
        self.stats.total_requests += 1;

        if let Some(cached) = self.cache.get(&expr) {
            self.stats.cache_hits += 1;
            // Estimate memory saved (rough approximation)
            self.stats.memory_saved_bytes += std::mem::size_of::<ExpressionNode>();
            return InternedExpr {
                inner: Arc::clone(cached),
            };
        }

        let arc = Arc::new(expr.clone());
        self.cache.insert(expr, Arc::clone(&arc));
        self.stats.cache_size = self.cache.len();

        InternedExpr { inner: arc }
    }

    /// Get interner statistics
    pub fn stats(&self) -> &InternerStats {
        &self.stats
    }

    /// Clear the cache and reset statistics
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats = InternerStats::default();
    }

    /// Get cache hit ratio as a percentage
    pub fn hit_ratio(&self) -> f64 {
        if self.stats.total_requests == 0 {
            0.0
        } else {
            (self.stats.cache_hits as f64 / self.stats.total_requests as f64) * 100.0
        }
    }

    /// Remove unused entries from the cache (entries with only one reference)
    pub fn garbage_collect(&mut self) {
        let before_size = self.cache.len();
        self.cache.retain(|_, arc| Arc::strong_count(arc) > 1);
        let after_size = self.cache.len();
        self.stats.cache_size = after_size;

        if before_size > after_size {
            log::debug!(
                "GC: Removed {} unused entries from expression cache",
                before_size - after_size
            );
        }
    }
}

impl Default for ExpressionInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Global expression interner instance
static GLOBAL_INTERNER: OnceLock<Mutex<ExpressionInterner>> = OnceLock::new();

/// Get access to the global expression interner
pub fn global_interner() -> &'static Mutex<ExpressionInterner> {
    GLOBAL_INTERNER.get_or_init(|| Mutex::new(ExpressionInterner::new()))
}

/// Convenience function to intern an expression using the global interner
pub fn intern_expr(expr: ExpressionNode) -> InternedExpr {
    let mut interner = global_interner().lock().unwrap();
    interner.intern(expr)
}

/// Get global interner statistics
pub fn global_interner_stats() -> InternerStats {
    let interner = global_interner().lock().unwrap();
    InternerStats {
        total_requests: interner.stats.total_requests,
        cache_hits: interner.stats.cache_hits,
        cache_size: interner.stats.cache_size,
        memory_saved_bytes: interner.stats.memory_saved_bytes,
    }
}

/// Clear the global interner cache
pub fn clear_global_interner() {
    let mut interner = global_interner().lock().unwrap();
    interner.clear();
}

/// Run garbage collection on the global interner
pub fn gc_global_interner() {
    let mut interner = global_interner().lock().unwrap();
    interner.garbage_collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::LiteralValue;

    #[test]
    fn test_interned_expr_basic() {
        let expr = ExpressionNode::literal(LiteralValue::Integer(42));
        let interned = InternedExpr::new(expr.clone());

        assert_eq!(interned.as_expr(), &expr);
        assert!(interned.is_unique());
    }

    #[test]
    fn test_expression_interner() {
        let mut interner = ExpressionInterner::new();

        let expr1 = ExpressionNode::literal(LiteralValue::Integer(42));
        let expr2 = ExpressionNode::literal(LiteralValue::Integer(42));

        let interned1 = interner.intern(expr1);
        let interned2 = interner.intern(expr2);

        // Should be the same Arc instance
        assert!(Arc::ptr_eq(&interned1.inner, &interned2.inner));
        assert_eq!(interner.stats().cache_hits, 1);
        assert_eq!(interner.hit_ratio(), 50.0); // 1 hit out of 2 requests
    }

    #[test]
    fn test_interner_garbage_collection() {
        let mut interner = ExpressionInterner::new();

        let expr = ExpressionNode::literal(LiteralValue::Integer(42));
        {
            let _interned = interner.intern(expr.clone());
            assert_eq!(interner.stats().cache_size, 1);
        }

        // After the interned expression is dropped, GC should remove it
        interner.garbage_collect();
        assert_eq!(interner.stats().cache_size, 0);
    }

    #[test]
    fn test_global_interner() {
        // Test with a unique value to avoid conflicts with other tests
        let unique_val = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let expr1 = ExpressionNode::literal(LiteralValue::Integer(unique_val));
        let expr2 = ExpressionNode::literal(LiteralValue::Integer(unique_val));

        let interned1 = intern_expr(expr1);
        let interned2 = intern_expr(expr2);

        // These should share memory since they're identical
        assert!(interned1.shares_memory_with(&interned2));

        // Create a different expression to verify they don't share memory
        let expr3 = ExpressionNode::literal(LiteralValue::Integer(unique_val + 1));
        let interned3 = intern_expr(expr3);

        assert!(!interned1.shares_memory_with(&interned3));
    }

    #[test]
    fn test_shares_memory_with() {
        clear_global_interner(); // Start clean

        let expr1 = ExpressionNode::literal(LiteralValue::Integer(42));
        let expr2 = ExpressionNode::literal(LiteralValue::Integer(42)); // Same
        let expr3 = ExpressionNode::literal(LiteralValue::Integer(99)); // Different

        let interned1 = intern_expr(expr1);
        let interned2 = intern_expr(expr2);
        let interned3 = intern_expr(expr3);

        // Same expressions should share memory
        assert!(interned1.shares_memory_with(&interned2));

        // Different expressions should not share memory
        assert!(!interned1.shares_memory_with(&interned3));
        assert!(!interned2.shares_memory_with(&interned3));
    }
}
