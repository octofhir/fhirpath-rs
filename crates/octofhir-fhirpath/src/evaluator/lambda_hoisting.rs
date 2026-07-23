//! Loop-invariant subexpression hoisting for lambda functions.
//!
//! `where()`, `select()` and friends evaluate their argument once per input item.
//! Parts of that argument often do not depend on the item at all — they navigate
//! from `%resource` or `%context`, which stay fixed for the whole evaluation. The
//! `dom-3` invariant is the pathological case:
//!
//! ```text
//! contained.where('#' + id in (%resource.descendants().reference | ...4 terms...))
//! ```
//!
//! Each of the four `%resource.descendants()...` terms walks the entire resource,
//! and the whole thing runs once per contained resource — quadratic in resource
//! size. Evaluating those terms once up front makes it linear.
//!
//! [`invariant_subexpressions`] finds the maximal such subtrees; the lambda
//! evaluator pre-evaluates them and publishes the results in a
//! [`crate::evaluator::context::HoistScope`] that child contexts inherit.

use std::collections::HashMap;

use crate::ast::ExpressionNode;
use crate::core::Result;
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext};

/// Below this many iterations the saved work cannot outweigh the analysis and the
/// extra context clone.
const MIN_ITEMS_TO_HOIST: usize = 2;

/// Pre-evaluate the loop-invariant parts of `lambda_arg` and return a context that
/// publishes the results to the iteration that follows.
///
/// Returns a plain clone of `context` when there is nothing worth hoisting.
///
/// A subexpression that fails to evaluate is silently left out rather than
/// propagated: the loop body may never have reached it (`false and %resource.x`),
/// and if it does, evaluating it in place reports the error at its proper time.
pub async fn hoist_into(
    context: &EvaluationContext,
    lambda_arg: &ExpressionNode,
    item_count: usize,
    evaluator: &AsyncNodeEvaluator<'_>,
) -> Result<EvaluationContext> {
    if item_count < MIN_ITEMS_TO_HOIST {
        return Ok(context.clone());
    }

    let candidates = invariant_subexpressions(lambda_arg);
    if candidates.is_empty() {
        return Ok(context.clone());
    }

    let mut entries = HashMap::with_capacity(candidates.len());
    for candidate in candidates {
        if let Ok(result) = evaluator.evaluate(candidate, context).await {
            entries.insert(candidate as *const ExpressionNode as usize, result.value);
        }
    }

    Ok(context.clone().with_hoist_scope(entries))
}

/// Variables whose value is fixed for an entire evaluation.
///
/// `%resource`/`%context` are set once when the root context is built and shared
/// unchanged by every child context. Every other variable is excluded: lambda
/// bindings change per item, and user variables can be rebound by `defineVariable`
/// in a nested scope.
fn is_invariant_variable(name: &str) -> bool {
    matches!(
        name,
        "resource" | "%resource" | "context" | "%context" | "rootResource" | "%rootResource"
    )
}

/// Functions that must always run in place: their result depends on wall-clock
/// time, or they have effects (tracing, variable binding) that a single hoisted
/// evaluation would collapse into one occurrence.
fn is_unhoistable_call(name: &str) -> bool {
    matches!(
        name,
        "now" | "today" | "timeOfDay" | "trace" | "defineVariable"
    )
}

/// Methods whose arguments are type specifiers rather than expressions. Their
/// argument parses as an identifier (`as(canonical)`), which must not be mistaken
/// for a navigation off the current focus.
fn takes_type_argument(method: &str) -> bool {
    matches!(method, "as" | "is" | "ofType")
}

/// Methods that rebind the focus for their argument. Inside these, `$this` and
/// bare identifiers refer to the inner item, so they say nothing about whether the
/// enclosing expression depends on the *outer* focus.
fn rebinds_focus(method: &str) -> bool {
    matches!(
        method,
        "where"
            | "select"
            | "all"
            | "exists"
            | "repeat"
            | "aggregate"
            | "sort"
            | "iif"
            | "allTrue"
            | "anyTrue"
            | "allFalse"
            | "anyFalse"
    )
}

/// Whether evaluating `node` can observe the current focus — the lambda item being
/// iterated — and therefore cannot be hoisted out of the loop.
///
/// `focus_bound` means some enclosing construct *within `node`* has already rebound
/// the focus, so focus-relative references below that point resolve against it
/// rather than against the caller's focus.
fn depends_on_focus(node: &ExpressionNode, focus_bound: bool) -> bool {
    match node {
        ExpressionNode::Literal(_) | ExpressionNode::TypeInfo(_) => false,

        // A bare identifier navigates off the current focus.
        ExpressionNode::Identifier(_) => !focus_bound,

        ExpressionNode::Variable(var) => {
            if is_invariant_variable(&var.name) {
                false
            } else {
                // Lambda bindings ($this/$index/$total) are satisfied by an inner
                // rebinding; anything else may be scoped and is assumed variant.
                !focus_bound || !is_lambda_binding(&var.name)
            }
        }

        ExpressionNode::Parenthesized(inner) => depends_on_focus(inner, focus_bound),

        ExpressionNode::PropertyAccess(access) => depends_on_focus(&access.object, focus_bound),

        ExpressionNode::IndexAccess(access) => {
            depends_on_focus(&access.object, focus_bound)
                || depends_on_focus(&access.index, focus_bound)
        }

        ExpressionNode::BinaryOperation(op) => {
            depends_on_focus(&op.left, focus_bound) || depends_on_focus(&op.right, focus_bound)
        }

        ExpressionNode::UnaryOperation(op) => depends_on_focus(&op.operand, focus_bound),

        ExpressionNode::Union(union) => {
            depends_on_focus(&union.left, focus_bound)
                || depends_on_focus(&union.right, focus_bound)
        }

        ExpressionNode::Collection(collection) => collection
            .elements
            .iter()
            .any(|element| depends_on_focus(element, focus_bound)),

        ExpressionNode::TypeCast(cast) => depends_on_focus(&cast.expression, focus_bound),
        ExpressionNode::TypeCheck(check) => depends_on_focus(&check.expression, focus_bound),

        ExpressionNode::Filter(filter) => {
            depends_on_focus(&filter.base, focus_bound)
                || depends_on_focus(&filter.condition, true)
        }

        // A bare call applies to the current focus.
        ExpressionNode::FunctionCall(call) => {
            is_unhoistable_call(&call.name)
                || !focus_bound
                || call
                    .arguments
                    .iter()
                    .any(|arg| depends_on_focus(arg, focus_bound))
        }

        ExpressionNode::MethodCall(call) => {
            if is_unhoistable_call(&call.method) {
                return true;
            }
            if depends_on_focus(&call.object, focus_bound) {
                return true;
            }
            if takes_type_argument(&call.method) {
                return false;
            }
            let args_focus_bound = focus_bound || rebinds_focus(&call.method);
            call.arguments
                .iter()
                .any(|arg| depends_on_focus(arg, args_focus_bound))
        }

        // Lambdas bind their own parameter; the body is evaluated against it.
        ExpressionNode::Lambda(lambda) => depends_on_focus(&lambda.body, true),

        // Path navigation semantics vary; treat conservatively.
        ExpressionNode::Path(_) => true,
    }
}

fn is_lambda_binding(name: &str) -> bool {
    matches!(
        name,
        "$this" | "this" | "$index" | "index" | "$total" | "total"
    )
}

/// Whether hoisting `node` can plausibly repay the bookkeeping.
///
/// Re-evaluating a literal or a plain variable reference is already as cheap as a
/// cache lookup, so only navigation that actually walks data is worth caching.
fn is_worth_hoisting(node: &ExpressionNode) -> bool {
    match node {
        ExpressionNode::MethodCall(_)
        | ExpressionNode::PropertyAccess(_)
        | ExpressionNode::IndexAccess(_)
        | ExpressionNode::Filter(_)
        | ExpressionNode::Union(_)
        | ExpressionNode::FunctionCall(_) => true,
        ExpressionNode::Parenthesized(inner) => is_worth_hoisting(inner),
        _ => false,
    }
}

/// Collect the maximal focus-independent subexpressions of `expr`.
///
/// Maximal: once a subtree qualifies, its children are not reported separately —
/// evaluating the parent once already covers them.
pub fn invariant_subexpressions(expr: &ExpressionNode) -> Vec<&ExpressionNode> {
    let mut found = Vec::new();
    collect(expr, &mut found);
    found
}

fn collect<'a>(node: &'a ExpressionNode, out: &mut Vec<&'a ExpressionNode>) {
    if is_worth_hoisting(node) && !depends_on_focus(node, false) {
        out.push(node);
        return;
    }

    match node {
        ExpressionNode::Parenthesized(inner) => collect(inner, out),
        ExpressionNode::PropertyAccess(access) => collect(&access.object, out),
        ExpressionNode::IndexAccess(access) => {
            collect(&access.object, out);
            collect(&access.index, out);
        }
        ExpressionNode::BinaryOperation(op) => {
            collect(&op.left, out);
            collect(&op.right, out);
        }
        ExpressionNode::UnaryOperation(op) => collect(&op.operand, out),
        ExpressionNode::Union(union) => {
            collect(&union.left, out);
            collect(&union.right, out);
        }
        ExpressionNode::Collection(collection) => {
            for element in &collection.elements {
                collect(element, out);
            }
        }
        ExpressionNode::TypeCast(cast) => collect(&cast.expression, out),
        ExpressionNode::TypeCheck(check) => collect(&check.expression, out),
        ExpressionNode::Filter(filter) => {
            collect(&filter.base, out);
            collect(&filter.condition, out);
        }
        ExpressionNode::FunctionCall(call) => {
            for arg in &call.arguments {
                collect(arg, out);
            }
        }
        ExpressionNode::MethodCall(call) => {
            collect(&call.object, out);
            if !takes_type_argument(&call.method) {
                for arg in &call.arguments {
                    collect(arg, out);
                }
            }
        }
        ExpressionNode::Lambda(lambda) => collect(&lambda.body, out),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expression;

    fn hoisted(expr: &str) -> Vec<String> {
        let parsed = parse_expression(expr).expect("expression should parse");
        invariant_subexpressions(&parsed)
            .into_iter()
            .map(|node| format!("{node:?}"))
            .collect()
    }

    #[test]
    fn hoists_resource_rooted_navigation() {
        assert_eq!(hoisted("%resource.descendants().reference").len(), 1);
    }

    #[test]
    fn hoists_a_union_of_invariant_terms_as_one_subtree() {
        // The union node is itself invariant, so it is reported whole rather than
        // as one entry per term.
        let found = hoisted(
            "'#' + id in (%resource.descendants().reference | %resource.descendants().as(canonical))",
        );
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("Union"));
    }

    #[test]
    fn skips_focus_dependent_navigation() {
        assert!(hoisted("descendants().where(reference = '#')").is_empty());
        assert!(hoisted("id").is_empty());
        assert!(hoisted("$this.reference").is_empty());
    }

    #[test]
    fn never_hoists_time_dependent_or_effectful_calls() {
        // The invariant receiver may still be hoisted — what must not be lifted out
        // of the loop is the effectful call itself, which has to run per item.
        for expr in [
            "%resource.descendants().trace('x')",
            "%resource.name.where(use = now())",
        ] {
            for found in hoisted(expr) {
                assert!(!found.contains("trace"), "hoisted trace from {expr}");
                assert!(!found.contains("\"now\""), "hoisted now() from {expr}");
            }
        }
    }

    #[test]
    fn skips_other_variables_which_may_be_rebound() {
        assert!(hoisted("%myvar.descendants()").is_empty());
    }

    #[test]
    fn hoists_from_inside_a_nested_lambda() {
        // The inner where() rebinds the focus, so the %resource term is still
        // invariant with respect to the outer loop.
        let found = hoisted("%resource.descendants().where(reference = '#')");
        assert_eq!(found.len(), 1);
    }
}
