//! Compile-time type guards (FHIRPath §1.11.1): when the focus can only be types for which
//! an operation is not defined, the expression is a semantic error.
//!
//! The static types of `$this` start from the declared context types (`Period | Timing`)
//! and narrow along the boolean structure of the expression: the right side of `implies`
//! and `and` sees the left side true, the right side of `or` sees it false, `iif` branches
//! see the condition true or false, and `not()` flips. `$this is T` (or `$this.is(T)`)
//! true keeps the types derived from `T`, false removes them. A `toString()` on `$this`
//! whose possible types are all complex (none has a string form) is reported.

use octofhir_fhir_model::ModelProvider;

use crate::ast::{BinaryOperator, ExpressionNode};
use crate::core::SourceLocation;
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

/// Diagnostics for operations a guarded `$this` cannot reach a valid type for.
pub fn check_type_guards(
    expression: &ExpressionNode,
    context_types: &[String],
    model_provider: &dyn ModelProvider,
) -> Vec<Diagnostic> {
    let checker = Checker { model_provider };
    let mut diagnostics = Vec::new();
    checker.visit(expression, Some(context_types.to_vec()), &mut diagnostics);
    diagnostics
}

struct Checker<'a> {
    model_provider: &'a dyn ModelProvider,
}

/// `$this` (also written without the `$`).
fn is_this(node: &ExpressionNode) -> bool {
    match node {
        ExpressionNode::Variable(v) => v.name == "this" || v.name == "$this",
        ExpressionNode::Parenthesized(inner) => is_this(inner),
        _ => false,
    }
}

/// The type a type specifier names, without a namespace (`FHIR.Period` is `Period`).
fn type_specifier(node: &ExpressionNode) -> Option<String> {
    let name = match node {
        ExpressionNode::Identifier(id) => id.name.clone(),
        ExpressionNode::TypeInfo(info) => info.name.clone(),
        ExpressionNode::PropertyAccess(access) => access.property.clone(),
        ExpressionNode::Path(path) => path.path.clone(),
        ExpressionNode::Literal(lit) => match &lit.value {
            crate::ast::LiteralValue::String(s) => s.clone(),
            _ => return None,
        },
        _ => return None,
    };
    Some(name.rsplit('.').next().unwrap_or(&name).to_string())
}

fn union(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    for t in right {
        if !left.contains(&t) {
            left.push(t);
        }
    }
    left
}

impl Checker<'_> {
    /// Whether a value of this type has a string form: primitives, System types and
    /// Quantity.
    fn has_string_form(&self, type_name: &str) -> bool {
        type_name.starts_with(|c: char| c.is_ascii_lowercase())
            || type_name.starts_with("System.")
            || type_name == "Quantity"
            || self
                .model_provider
                .is_type_derived_from(type_name, "Quantity")
    }

    /// The possible types of `$this` once `condition` is known to be `truth`.
    fn narrow(&self, condition: &ExpressionNode, truth: bool, types: &[String]) -> Vec<String> {
        let guard = |target: &str| -> Vec<String> {
            types
                .iter()
                .filter(|t| {
                    let derived =
                        *t == target || self.model_provider.is_type_derived_from(t, target);
                    derived == truth
                })
                .cloned()
                .collect()
        };
        match condition {
            ExpressionNode::Parenthesized(inner) => self.narrow(inner, truth, types),
            ExpressionNode::TypeCheck(check) if is_this(&check.expression) => guard(
                check
                    .target_type
                    .rsplit('.')
                    .next()
                    .unwrap_or(&check.target_type),
            ),
            ExpressionNode::BinaryOperation(op) => match op.operator {
                BinaryOperator::Is if is_this(&op.left) => match type_specifier(&op.right) {
                    Some(target) => guard(&target),
                    None => types.to_vec(),
                },
                BinaryOperator::And if truth => {
                    let left = self.narrow(&op.left, true, types);
                    self.narrow(&op.right, true, &left)
                }
                BinaryOperator::And => union(
                    self.narrow(&op.left, false, types),
                    self.narrow(&op.right, false, types),
                ),
                BinaryOperator::Or if truth => union(
                    self.narrow(&op.left, true, types),
                    self.narrow(&op.right, true, types),
                ),
                BinaryOperator::Or => {
                    let left = self.narrow(&op.left, false, types);
                    self.narrow(&op.right, false, &left)
                }
                _ => types.to_vec(),
            },
            ExpressionNode::MethodCall(call)
                if call.method == "not" && call.arguments.is_empty() =>
            {
                self.narrow(&call.object, !truth, types)
            }
            ExpressionNode::MethodCall(call)
                if call.method == "is" && is_this(&call.object) && call.arguments.len() == 1 =>
            {
                match type_specifier(&call.arguments[0]) {
                    Some(target) => guard(&target),
                    None => types.to_vec(),
                }
            }
            ExpressionNode::FunctionCall(call)
                if call.name == "is" && call.arguments.len() == 1 =>
            {
                match type_specifier(&call.arguments[0]) {
                    Some(target) => guard(&target),
                    None => types.to_vec(),
                }
            }
            _ => types.to_vec(),
        }
    }

    fn report(
        &self,
        types: &Option<Vec<String>>,
        location: Option<SourceLocation>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(types) = types else { return };
        if types.is_empty() || types.iter().any(|t| self.has_string_form(t)) {
            return;
        }
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "TYPE_GUARD".to_string(),
                namespace: Some("fhirpath".to_string()),
            },
            message: format!(
                "toString() is not defined for $this of type {}",
                types.join(" | ")
            ),
            location,
            related: vec![],
        });
    }

    /// Visit `node` with the possible types of `$this` (`None` where `$this` is another
    /// focus, inside an iteration).
    fn visit(
        &self,
        node: &ExpressionNode,
        types: Option<Vec<String>>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node {
            ExpressionNode::BinaryOperation(op) => {
                let narrowed =
                    |truth: bool| types.as_ref().map(|t| self.narrow(&op.left, truth, t));
                let right_types = match op.operator {
                    BinaryOperator::Implies | BinaryOperator::And => narrowed(true),
                    BinaryOperator::Or => narrowed(false),
                    _ => types.clone(),
                };
                self.visit(&op.left, types, diagnostics);
                self.visit(&op.right, right_types, diagnostics);
            }
            ExpressionNode::MethodCall(call) => {
                if call.method == "toString" && call.arguments.is_empty() && is_this(&call.object) {
                    self.report(&types, call.location.clone(), diagnostics);
                }
                self.visit(&call.object, types, diagnostics);
                // Arguments iterate over the method's input: `$this` is another value there.
                for argument in &call.arguments {
                    self.visit(argument, None, diagnostics);
                }
            }
            ExpressionNode::FunctionCall(call) => {
                if call.name == "toString" && call.arguments.is_empty() {
                    self.report(&types, call.location.clone(), diagnostics);
                } else if call.name == "iif" && (2..=3).contains(&call.arguments.len()) {
                    let condition = &call.arguments[0];
                    let narrowed =
                        |truth: bool| types.as_ref().map(|t| self.narrow(condition, truth, t));
                    self.visit(condition, types.clone(), diagnostics);
                    self.visit(&call.arguments[1], narrowed(true), diagnostics);
                    if let Some(otherwise) = call.arguments.get(2) {
                        self.visit(otherwise, narrowed(false), diagnostics);
                    }
                    return;
                }
                for argument in &call.arguments {
                    self.visit(argument, types.clone(), diagnostics);
                }
            }
            ExpressionNode::Parenthesized(inner) => self.visit(inner, types, diagnostics),
            ExpressionNode::UnaryOperation(op) => self.visit(&op.operand, types, diagnostics),
            ExpressionNode::TypeCheck(check) => self.visit(&check.expression, types, diagnostics),
            ExpressionNode::TypeCast(cast) => self.visit(&cast.expression, types, diagnostics),
            ExpressionNode::PropertyAccess(access) => {
                self.visit(&access.object, types, diagnostics)
            }
            ExpressionNode::IndexAccess(access) => {
                self.visit(&access.object, types.clone(), diagnostics);
                self.visit(&access.index, types, diagnostics);
            }
            ExpressionNode::Union(union) => {
                self.visit(&union.left, types.clone(), diagnostics);
                self.visit(&union.right, types, diagnostics);
            }
            ExpressionNode::Collection(collection) => {
                for element in &collection.elements {
                    self.visit(element, types.clone(), diagnostics);
                }
            }
            ExpressionNode::Filter(filter) => {
                self.visit(&filter.base, types, diagnostics);
                self.visit(&filter.condition, None, diagnostics);
            }
            ExpressionNode::Path(path) => self.visit(&path.base, types, diagnostics),
            ExpressionNode::Lambda(lambda) => self.visit(&lambda.body, None, diagnostics),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(expression: &str, types: &[&str]) -> usize {
        let ast = crate::parser::parse_ast(expression).expect("parses");
        // Without a model, a type is derived only from itself.
        let provider = octofhir_fhir_model::EmptyModelProvider;
        let types: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        check_type_guards(&ast, &types, &provider).len()
    }

    #[test]
    fn guards_narrow_the_focus_type() {
        let s = "$this.toString() = 'x'";
        assert_eq!(errors(s, &["Period"]), 1);
        assert_eq!(errors(s, &["Period", "dateTime"]), 0);
        assert_eq!(
            errors(&format!("$this is dateTime implies {s}"), &["Period"]),
            0
        );
        assert_eq!(
            errors(
                &format!("$this is Period implies {s}"),
                &["Period", "Timing"]
            ),
            1
        );
        assert_eq!(errors(&format!("$this is Period or {s}"), &["Period"]), 0);
        assert_eq!(
            errors(&format!("$this is Period or {s}"), &["Period", "Timing"]),
            1
        );
        assert_eq!(
            errors(
                &format!("$this is Period or $this is Timing or {s}"),
                &["Period", "Timing"]
            ),
            0
        );
        assert_eq!(
            errors(&format!("$this is dateTime xor {s}"), &["Period"]),
            1
        );
        assert_eq!(
            errors(&format!("($this is Period).not() implies {s}"), &["Period"]),
            0
        );
        assert_eq!(
            errors(&format!("iif($this is Period, true, {s})"), &["Period"]),
            0
        );
        assert_eq!(
            errors(&format!("iif($this is Period, {s}, true)"), &["Period"]),
            1
        );
        assert_eq!(
            errors(&format!("$this.start is dateTime implies {s}"), &["Period"]),
            1
        );
        assert_eq!(
            errors("($this as dateTime).toString() = 'x'", &["Period"]),
            0
        );
    }
}
