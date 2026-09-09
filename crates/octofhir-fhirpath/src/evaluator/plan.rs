//! Cached instructions for CPU-only regions and a stack-machine program for
//! provider access and context-sensitive constructs. No speculative execution.
use super::function_registry::{
    FunctionEvaluatorWrapper, NullPropagationStrategy, PureFunctionEvaluator,
};
use super::operator_registry::OperationEvaluator;
use super::{EvaluationContext, EvaluationResult, Evaluator};
use crate::ast::{BinaryOperator, ExpressionNode};
use crate::core::{Collection, FhirPathError, Result};
use std::sync::Arc;

/// Retained executable handle. Independent of cache eviction; execute with the
/// engine that compiled it via `FhirPathEngine::evaluate_plan`.
pub struct CompiledPlan {
    pub(super) ast: Arc<ExpressionNode>,
    pub(super) owner: Arc<()>,
    instructions: Box<[Instruction]>,
}

enum Instruction {
    Value(Collection),
    Input,
    Path(Arc<str>),
    Variable(Arc<str>),
    Execute(Arc<super::vm::Program>, usize),
    ShortCircuit(BinaryOperator, usize),
    Binary(Arc<dyn OperationEvaluator>),
    Unary(Arc<dyn OperationEvaluator>),
    Call(Arc<dyn PureFunctionEvaluator>, Vec<Collection>),
}

impl CompiledPlan {
    pub(super) fn new(ast: Arc<ExpressionNode>, evaluator: &Evaluator, owner: Arc<()>) -> Self {
        let mut instructions = Vec::new();
        lower(&ast, evaluator, &mut instructions, 0, &ast, &mut None);
        Self {
            ast,
            owner,
            instructions: instructions.into_boxed_slice(),
        }
    }

    /// Original syntax tree, also usable by metadata-aware evaluation.
    pub fn ast(&self) -> &Arc<ExpressionNode> {
        &self.ast
    }

    pub(super) async fn evaluate(
        &self,
        evaluator: &Evaluator,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let mut stack: smallvec::SmallVec<[Collection; 4]> = smallvec::SmallVec::new();
        let mut pc = 0;
        while pc < self.instructions.len() {
            match &self.instructions[pc] {
                Instruction::Value(value) => stack.push(value.clone()),
                Instruction::Input => stack.push(context.input_collection().clone()),
                Instruction::Path(name) => {
                    // Provider navigation has a large async state too; pure
                    // plans should not carry it on every runtime poll.
                    stack.push(
                        Box::pin(evaluator.evaluate_path(name, context))
                            .await?
                            .value,
                    )
                }
                Instruction::Variable(name) => {
                    stack.push(evaluator.evaluate_variable_sync(name, context)?.value)
                }
                Instruction::Execute(program, entry) => stack.push(
                    // Keep the large general interpreter state out of every
                    // CPU-only plan's future. Allocate only at the VM boundary.
                    Box::pin(super::vm::run_at(
                        program.clone(),
                        *entry,
                        evaluator,
                        context,
                    ))
                    .await?
                    .value,
                ),
                Instruction::ShortCircuit(operator, target) => {
                    if let Some(value) = Evaluator::short_circuit(operator, stack.last().unwrap()) {
                        *stack.last_mut().unwrap() = Collection::single(value);
                        pc = *target;
                        continue;
                    }
                }
                Instruction::Binary(operator) => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(
                        operator
                            .evaluate_sync(Collection::empty(), context, left, right)?
                            .value,
                    );
                }
                Instruction::Unary(operator) => {
                    let operand = stack.pop().unwrap();
                    stack.push(
                        operator
                            .evaluate_sync(
                                Collection::empty(),
                                context,
                                operand,
                                Collection::empty(),
                            )?
                            .value,
                    );
                }
                Instruction::Call(function, args) => {
                    let input = stack.pop().unwrap();
                    if input.is_empty()
                        && matches!(
                            function.metadata().null_propagation,
                            NullPropagationStrategy::Focus
                        )
                    {
                        stack.push(Collection::empty());
                    } else {
                        stack.push(function.evaluate_sync_borrowed(input, args)?.value);
                    }
                }
            }
            pc += 1;
        }
        Ok(EvaluationResult {
            value: stack.pop().ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    "Empty evaluation plan",
                )
            })?,
        })
    }
}

fn lower(
    node: &ExpressionNode,
    evaluator: &Evaluator,
    out: &mut Vec<Instruction>,
    depth: usize,
    root: &Arc<ExpressionNode>,
    program: &mut Option<Arc<super::vm::Program>>,
) {
    // Bound the small fast-region compiler's recursion. The general compiler
    // below traverses and executes every remaining node with explicit stacks.
    if depth < 128 {
        match node {
            ExpressionNode::Identifier(node) => {
                out.push(Instruction::Path(Arc::from(node.name.as_str())));
                return;
            }
            ExpressionNode::Variable(node) => {
                out.push(Instruction::Variable(Arc::from(node.name.as_str())));
                return;
            }
            ExpressionNode::Collection(collection) if collection.elements.is_empty() => {
                out.push(Instruction::Value(Collection::empty()));
                return;
            }
            ExpressionNode::Literal(literal) => {
                if let Ok(value) = evaluator.evaluate_literal(&literal.value) {
                    out.push(Instruction::Value(Collection::single(value)));
                    return;
                }
            }
            ExpressionNode::Parenthesized(inner) => {
                lower(inner, evaluator, out, depth + 1, root, program);
                return;
            }
            ExpressionNode::BinaryOperation(binary)
                if !matches!(
                    binary.operator,
                    BinaryOperator::Is | BinaryOperator::As | BinaryOperator::Union
                ) =>
            {
                if let Some(operator) = evaluator
                    .operator_registry
                    .get_binary_operator(&binary.operator)
                    && operator.supports_sync()
                {
                    lower(&binary.left, evaluator, out, depth + 1, root, program);
                    let jump = out.len();
                    let short_circuit = matches!(
                        binary.operator,
                        BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Implies
                    );
                    if short_circuit {
                        out.push(Instruction::ShortCircuit(binary.operator, 0));
                    }
                    lower(&binary.right, evaluator, out, depth + 1, root, program);
                    out.push(Instruction::Binary(operator.clone()));
                    let target = out.len();
                    if short_circuit {
                        out[jump] = Instruction::ShortCircuit(binary.operator, target);
                    }
                    return;
                }
            }
            ExpressionNode::UnaryOperation(unary) => {
                if let Some(operator) = evaluator
                    .operator_registry
                    .get_unary_operator(&unary.operator)
                    && operator.supports_sync()
                {
                    lower(&unary.operand, evaluator, out, depth + 1, root, program);
                    out.push(Instruction::Unary(operator.clone()));
                    return;
                }
            }
            _ => {}
        }
        let call = match node {
            ExpressionNode::MethodCall(call) => Some((
                call.method.as_str(),
                &call.arguments,
                Some(call.object.as_ref()),
            )),
            ExpressionNode::FunctionCall(call) => Some((call.name.as_str(), &call.arguments, None)),
            _ => None,
        };
        if let Some((name, args, receiver)) = call
            && let Some(wrapper) = evaluator.function_registry.get_function_wrapper(name)
            && let Some((function, literals)) = prepare_call(wrapper, args, evaluator)
        {
            if let Some(receiver) = receiver {
                lower(receiver, evaluator, out, depth + 1, root, program);
            } else {
                out.push(Instruction::Input);
            }
            out.push(Instruction::Call(function, literals));
            return;
        }
    }
    // Share the original owned AST and one general program between regions.
    // Never recursively clone a subtree at a fast/general execution boundary.
    let program =
        program.get_or_insert_with(|| super::vm::Program::compile(root.clone(), evaluator));
    out.push(Instruction::Execute(program.clone(), program.node_id(node)));
}

pub(super) fn prepare_call(
    wrapper: &FunctionEvaluatorWrapper,
    args: &[ExpressionNode],
    evaluator: &Evaluator,
) -> Option<(Arc<dyn PureFunctionEvaluator>, Vec<Collection>)> {
    let function = match wrapper {
        FunctionEvaluatorWrapper::Pure(function) => function.clone(),
        FunctionEvaluatorWrapper::Lazy(function) if args.is_empty() => {
            function.prepare_no_args()?
        }
        _ => return None,
    };
    if !function.supports_sync() {
        return None;
    }
    let literals: Option<Vec<_>> = args
        .iter()
        .map(|arg| match arg {
            ExpressionNode::Literal(literal) => evaluator
                .evaluate_literal(&literal.value)
                .ok()
                .map(Collection::single),
            ExpressionNode::Collection(collection) if collection.elements.is_empty() => {
                Some(Collection::empty())
            }
            _ => None,
        })
        .collect();
    let literals = literals?;
    Some((function.prepare(&literals).unwrap_or(function), literals))
}

#[cfg(test)]
mod tests {
    use crate::testing::{test_context, test_engine};
    #[tokio::test]
    async fn concurrent_compiles_share_handles_and_reject_foreign_engine() {
        let engine = test_engine().await;
        let barrier = std::sync::Barrier::new(8);
        let plans = std::thread::scope(|scope| {
            let jobs: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        engine.compile_plan("12345 + 67890").unwrap()
                    })
                })
                .collect();
            jobs.into_iter()
                .map(|job| job.join().unwrap())
                .collect::<Vec<_>>()
        });
        for plan in &plans {
            assert!(std::sync::Arc::ptr_eq(plan, &plans[0]));
        }
        for i in 0..300 {
            engine.compile_plan(&format!("{i} + 1")).unwrap();
        }
        assert_eq!(
            engine
                .evaluate_plan(&plans[0], &test_context())
                .await
                .unwrap()
                .value
                .to_json_value(),
            serde_json::json!(80235)
        );
        let other = test_engine().await;
        assert!(
            other
                .evaluate_plan(&plans[0], &test_context())
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn plans_match_ast_and_preserve_short_circuiting() {
        let engine = test_engine().await;
        let context = test_context();
        for expression in [
            "1 + 2 * 3",
            "false and (1 / 0 > 2)",
            "true or {}.single()",
            "false implies (1 / 0 > 2)",
            "'abc'.substring(1)",
            "(1 | 2).count()",
            "(1 | 2).where($this > 1).count()",
            "1 'm' + 100 'cm'",
            "@2024-01-01 + 1 day",
        ] {
            let plan = engine.compile_plan(expression).unwrap();
            let expected = engine.evaluate_ast(plan.ast(), &context).await.unwrap();
            assert_eq!(
                engine.evaluate_plan(&plan, &context).await.unwrap().value,
                expected.value,
                "{expression}"
            );
        }
    }
}
