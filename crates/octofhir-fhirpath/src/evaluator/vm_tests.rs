use super::*;
use crate::ast::expression::{BinaryOperationNode, FunctionCallNode};
use crate::evaluator::function_registry::{
    FunctionMetadata, FunctionRegistry, LazyFunctionEvaluator,
};
use octofhir_fhir_model::EmptyModelProvider;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

fn evaluator(registry: FunctionRegistry) -> Evaluator {
    Evaluator::new(
        Arc::new(super::super::create_standard_operator_registry()),
        Arc::new(registry),
        Arc::new(EmptyModelProvider),
        None,
    )
}

fn parse(expression: &str) -> ExpressionNode {
    crate::parser::parse_ast(expression).unwrap()
}

#[tokio::test]
async fn vm_matches_reference_scopes_laziness_ordering_and_errors() {
    let evaluator = evaluator(crate::create_function_registry());
    for expression in [
        "(1 | 2 | 3).where($this > 1).select($this * 2)",
        "(1 | 2 | 3).aggregate($total + $this, 0)",
        "(1 | 2).select(iif($this = 1, 10, 20))",
        "(1 | 2).where(false and (1 / 0 > 0))",
        "(1 | 2).where(true or doesNotExist())",
        "(1 | 2).select($this.defineVariable('x', $this + 1).select(%x))",
        "1.defineVariable('x', 3) | 2.defineVariable('x', 4)",
        "(1 | 2)[1]",
        "(1 | 2).where($this is Integer)",
        "'abc'.replaceMatches('[a-z]', 'x')",
        "(1 | 2).where($this > 0).single()",
        "'abc'.matches('[')",
        "iif(false, doesNotExist(), 42)",
    ] {
        let ast = Arc::new(parse(expression));
        let context = crate::testing::test_context();
        let expected = evaluator.evaluate_reference(&ast, &context.nest()).await;
        let actual = run(
            Program::compile(ast, &evaluator),
            &evaluator,
            &context.nest(),
        )
        .await;
        match (expected, actual) {
            (Ok(expected), Ok(actual)) => assert_eq!(expected.value, actual.value, "{expression}"),
            (Err(expected), Err(actual)) => {
                assert_eq!(expected.to_string(), actual.to_string(), "{expression}")
            }
            results => panic!("VM/reference mismatch for {expression}: {results:?}"),
        }
    }
}

#[test]
fn inline_frames_do_not_clone_the_shared_program() {
    let evaluator = evaluator(crate::create_function_registry());
    let program = Program::compile(Arc::new(parse("(1 + 2) * 3")), &evaluator);
    let refs = Arc::strong_count(&program);
    let context = crate::testing::test_context();
    let bridge = Bridge::default();
    let mut stack = Stack::borrowed(&program, 0, context);
    let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    let mut budget = 2;
    assert!(
        stack
            .poll(&mut cx, &evaluator, &bridge, &mut budget)
            .is_pending()
    );
    assert!(!stack.state.frames.is_empty());
    assert_eq!(Arc::strong_count(&program), refs);
    drop(stack);
    assert_eq!(Arc::strong_count(&program), refs);
}

#[test]
fn borrowed_prepared_regex_preserves_validation_and_changed_arguments() {
    let registry = crate::create_function_registry();
    let string = |value: &str| Collection::single(FhirPathValue::string(value));
    for name in ["matches", "matchesFull"] {
        let Some(FunctionEvaluatorWrapper::Pure(function)) = registry.get_function_wrapper(name)
        else {
            panic!("Missing pure regex function");
        };
        let prepared = function.prepare(&[string("^a$")]).unwrap();
        let result = prepared
            .evaluate_sync_borrowed(string("b"), &[string("^b$")])
            .unwrap();
        assert_eq!(
            result.value,
            Collection::single(FhirPathValue::boolean(true))
        );
        assert!(prepared.evaluate_sync_borrowed(string("b"), &[]).is_err());
        assert!(
            prepared
                .evaluate_sync_borrowed(string("b"), &[string("[")])
                .is_err()
        );
        assert!(
            prepared
                .evaluate_sync_borrowed(
                    string("b"),
                    &[Collection::single(FhirPathValue::integer(1))],
                )
                .is_err()
        );
    }
    let Some(FunctionEvaluatorWrapper::Pure(function)) =
        registry.get_function_wrapper("replaceMatches")
    else {
        panic!("Missing replacement function");
    };
    let prepared = function.prepare(&[string("^a$"), string("x")]).unwrap();
    assert_eq!(
        prepared
            .evaluate_sync_borrowed(string("b"), &[string("^b$"), string("y")])
            .unwrap()
            .value,
        string("y"),
    );
    assert_eq!(
        prepared
            .evaluate_sync_borrowed(string("b"), &[string(""), string("y")])
            .unwrap()
            .value,
        string("b"),
    );
    assert!(
        prepared
            .evaluate_sync_borrowed(string("b"), &[string("["), string("y")])
            .is_err()
    );
}

#[tokio::test]
async fn reused_task_slots_switch_programs_and_release_dynamic_trees() {
    let mut registry = crate::create_function_registry();
    register(&mut registry, "vmDynamic", Behavior::Dynamic);
    let evaluator = evaluator(registry);
    let program = Program::compile(
        Arc::new(parse("(1 | 2 | 3).select(vmDynamic() + $this)")),
        &evaluator,
    );
    let result = run(program, &evaluator, &crate::testing::test_context())
        .await
        .unwrap();
    assert_eq!(
        result.value,
        Collection::from_values(vec![
            FhirPathValue::integer(43),
            FhirPathValue::integer(44),
            FhirPathValue::integer(45),
        ])
    );
    let dynamic = Program::compile(Arc::new(parse("40 + 2")), &evaluator);
    let weak = Arc::downgrade(&dynamic);
    let mut stack = Stack::new(
        NodeRef {
            program: dynamic,
            id: 0,
        },
        crate::testing::test_context(),
    );
    stack.clear();
    assert!(weak.upgrade().is_none());
}

#[tokio::test]
async fn deep_binary_ast_uses_heap_frames_not_recursive_polling() {
    let evaluator = evaluator(crate::create_function_registry());
    let one = parse("1");
    let mut ast = one.clone();
    for _ in 0..5000 {
        ast = ExpressionNode::BinaryOperation(BinaryOperationNode {
            left: Box::new(ast),
            right: Box::new(one.clone()),
            operator: BinaryOperator::Add,
            location: None,
        });
    }
    let ast = Arc::new(ast);
    let plan = super::super::plan::CompiledPlan::new(ast.clone(), &evaluator, Arc::new(()));
    assert_eq!(
        plan.evaluate(&evaluator, &crate::testing::test_context())
            .await
            .unwrap()
            .value,
        Collection::single(FhirPathValue::integer(5001)),
    );
    drop(plan);
    let result = run(
        Program::compile(ast.clone(), &evaluator),
        &evaluator,
        &crate::testing::test_context(),
    )
    .await
    .unwrap();
    assert_eq!(
        result.value,
        Collection::single(FhirPathValue::integer(5001))
    );
    let program = Program::compile(ast.clone(), &evaluator);
    assert!(program.nodes[0].callback_free);
    let bridge = Bridge::default();
    let callback = Callback::new(&bridge, program);
    let inline_result = callback
        .evaluate(&evaluator, &ast, &crate::testing::test_context())
        .await
        .unwrap();
    assert_eq!(inline_result.value, result.value);
    assert!(bridge.requests.lock().is_empty());
    drop(callback);
    // The public Box-based AST still has a recursive destructor. Deliberately
    // drop this synthetic tree iteratively; this test targets execution.
    let mut ast = Arc::try_unwrap(ast).unwrap();
    while let ExpressionNode::BinaryOperation(binary) = ast {
        ast = *binary.left;
    }
}

#[tokio::test]
async fn deeply_nested_lazy_calls_make_progress_across_yields() {
    let evaluator = evaluator(crate::create_function_registry());
    let mut ast = parse("42");
    for _ in 0..1024 {
        ast = ExpressionNode::FunctionCall(FunctionCallNode {
            name: "iif".into(),
            arguments: vec![parse("true"), ast, parse("0")],
            location: None,
        });
    }
    let ast = Arc::new(ast);
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            Program::compile(ast.clone(), &evaluator),
            &evaluator,
            &crate::testing::test_context(),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result.value, Collection::single(FhirPathValue::integer(42)));
    let mut ast = Arc::try_unwrap(ast).unwrap();
    while let ExpressionNode::FunctionCall(mut call) = ast {
        ast = call.arguments.remove(1);
    }
}

enum Behavior {
    Join,
    Barrier(Arc<tokio::sync::Barrier>),
    Race(Arc<AtomicUsize>),
    Pending(Arc<AtomicUsize>),
    Yield,
    Dynamic,
}

#[test]
fn callback_free_classification_respects_lazy_boundaries() {
    let evaluator = evaluator(crate::create_function_registry());
    for (expression, expected) in [
        ("id.matches('^item-[0-9]+$')", true),
        ("active and (score + 1 > 10)", true),
        ("{}.exists()", true),
        ("iif(active, 1, 2)", false),
        ("item.where(active)", false),
        ("$this is Integer", false),
        ("'abc'.matches(%pattern)", false),
    ] {
        let program = Program::compile(Arc::new(parse(expression)), &evaluator);
        assert_eq!(program.nodes[0].callback_free, expected, "{expression}");
    }
}

#[tokio::test]
async fn specializing_exists_does_not_bypass_custom_registry_overrides() {
    let mut registry = crate::create_function_registry();
    register(&mut registry, "exists", Behavior::Yield);
    let evaluator = evaluator(registry);
    let plan = super::super::plan::CompiledPlan::new(
        Arc::new(parse("{}.exists()")),
        &evaluator,
        Arc::new(()),
    );
    let result = plan
        .evaluate(&evaluator, &crate::testing::test_context())
        .await
        .unwrap();
    assert_eq!(result.value, Collection::single(FhirPathValue::integer(42)));
}

struct TestFunction {
    metadata: FunctionMetadata,
    behavior: Behavior,
}
struct Active(Arc<AtomicUsize>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for TestFunction {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        self.evaluate_borrowed(input, context, &args, evaluator)
            .await
    }
    async fn evaluate_borrowed(
        &self,
        _input: Collection,
        context: &EvaluationContext,
        args: &[ExpressionNode],
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        match &self.behavior {
            Behavior::Join => {
                let (left, right) = tokio::join!(
                    evaluator.evaluate(&args[0], context),
                    evaluator.evaluate(&args[1], context)
                );
                let mut result = left?.value.into_vec();
                result.extend(right?.value);
                return Ok(EvaluationResult {
                    value: Collection::from(result),
                });
            }
            Behavior::Barrier(barrier) => {
                barrier.wait().await;
            }
            Behavior::Race(active) => {
                let result = tokio::select! {
                    biased;
                    result = evaluator.evaluate(&args[0], context) => result,
                    result = evaluator.evaluate(&args[1], context) => result,
                };
                tokio::task::yield_now().await;
                assert_eq!(
                    active.load(Ordering::SeqCst),
                    0,
                    "Cancelled child I/O remained active"
                );
                return result;
            }
            Behavior::Pending(active) => {
                active.fetch_add(1, Ordering::SeqCst);
                let _guard = Active(active.clone());
                std::future::pending::<()>().await;
            }
            Behavior::Yield => {
                tokio::task::yield_now().await;
            }
            Behavior::Dynamic => return evaluator.evaluate(&parse("40 + 2"), context).await,
        }
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::integer(42)),
        })
    }
    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

fn register(registry: &mut FunctionRegistry, name: &str, behavior: Behavior) {
    let mut metadata = registry
        .get_function_wrapper("iif")
        .unwrap()
        .metadata()
        .clone();
    metadata.name = name.into();
    metadata.deterministic = false;
    metadata.signature.min_params = 0;
    metadata.signature.max_params = None;
    registry.register_lazy_function(Arc::new(TestFunction { metadata, behavior }));
}

#[tokio::test]
async fn joined_callbacks_can_wait_for_each_other_without_deadlock() {
    let mut registry = crate::create_function_registry();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    register(&mut registry, "vmJoin", Behavior::Join);
    register(&mut registry, "vmLeft", Behavior::Barrier(barrier.clone()));
    register(&mut registry, "vmRight", Behavior::Barrier(barrier));
    let evaluator = evaluator(registry);
    let program = Program::compile(Arc::new(parse("vmJoin(vmLeft(), vmRight())")), &evaluator);
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        run(program, &evaluator, &crate::testing::test_context()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result.value.len(), 2);
}

#[tokio::test]
async fn losing_select_branch_cancels_io_that_never_wakes() {
    let mut registry = crate::create_function_registry();
    let active = Arc::new(AtomicUsize::new(0));
    register(&mut registry, "vmRace", Behavior::Race(active.clone()));
    register(
        &mut registry,
        "vmPending",
        Behavior::Pending(active.clone()),
    );
    register(&mut registry, "vmReady", Behavior::Yield);
    let evaluator = evaluator(registry);
    let program = Program::compile(
        Arc::new(parse("vmRace(vmPending(), vmReady())")),
        &evaluator,
    );
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        run(program, &evaluator, &crate::testing::test_context()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result.value, Collection::single(FhirPathValue::integer(42)));
    assert_eq!(active.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn dropping_root_cancels_provider_future_and_dynamic_asts_are_owned() {
    let mut registry = crate::create_function_registry();
    let active = Arc::new(AtomicUsize::new(0));
    register(
        &mut registry,
        "vmPending",
        Behavior::Pending(active.clone()),
    );
    register(&mut registry, "vmDynamic", Behavior::Dynamic);
    let evaluator = evaluator(registry);
    let context = crate::testing::test_context();
    let program = Program::compile(Arc::new(parse("vmPending()")), &evaluator);
    let mut future = Box::pin(run(program, &evaluator, &context));
    assert!(futures::poll!(future.as_mut()).is_pending());
    assert_eq!(active.load(Ordering::SeqCst), 1);
    drop(future);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    let result = run(
        Program::compile(Arc::new(parse("vmDynamic()")), &evaluator),
        &evaluator,
        &context,
    )
    .await
    .unwrap();
    assert_eq!(result.value, Collection::single(FhirPathValue::integer(42)));
}
