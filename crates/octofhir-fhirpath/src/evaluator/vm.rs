//! Stack-machine execution for context-dependent expressions.
//!
//! Built-in/custom async functions keep their public callback interface. Child
//! expressions are scheduled as independent stack tasks, not recursively polled
//! evaluator futures. The scheduler also supports join/select and cancellation.
use super::function_registry::{
    FunctionEvaluatorWrapper, NullPropagationStrategy, PureFunctionEvaluator,
};
use super::{
    AsyncNodeEvaluator, EvaluationContext, EvaluationResult, Evaluator, OperationEvaluator,
};
use crate::ast::{BinaryOperator, ExpressionNode};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use futures::task::{ArcWake, AtomicWaker, waker_ref};
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use tokio::sync::oneshot;

#[cfg(test)]
#[path = "vm_tests.rs"]
mod tests;

type Id = usize;
type EvalFuture<'a> = Pin<Box<dyn Future<Output = Result<EvaluationResult>> + Send + 'a>>;
type Reply = oneshot::Sender<Result<EvaluationResult>>;
type PreparedCall = (Arc<dyn PureFunctionEvaluator>, Vec<Collection>);

pub(super) struct Program {
    ast: Arc<ExpressionNode>,
    nodes: Box<[Node]>,
    addresses: HashMap<usize, Id>,
}

struct Node {
    address: usize,
    op: Op,
    callback_free: bool,
}

enum Op {
    Value(Result<Collection>),
    Variable(Arc<str>),
    Path {
        receiver: Option<Id>,
        name: Arc<str>,
    },
    Forward(Id),
    Binary {
        left: Id,
        right: Id,
        kind: BinaryOperator,
        isolate: bool,
        operator: Option<Arc<dyn OperationEvaluator>>,
    },
    Unary {
        operand: Id,
        operator: Option<Arc<dyn OperationEvaluator>>,
    },
    Index {
        object: Id,
        index: Id,
    },
    Collection(Box<[Id]>),
    Call {
        receiver: Option<Id>,
        name: Arc<str>,
        path: Box<[usize]>,
        function: Option<FunctionEvaluatorWrapper>,
        prepared: Option<PreparedCall>,
    },
    Type {
        operand: Id,
        name: Arc<str>,
        function: Option<FunctionEvaluatorWrapper>,
        check: bool,
    },
    ReferenceDescendants(Id),
    Unsupported(String),
}

/// Child ordering is used only to find borrowed function arguments in the owned
/// tree. No self-referential pointers or unsafe dereferences are needed.
fn children(node: &ExpressionNode) -> SmallVec<[&ExpressionNode; 4]> {
    use ExpressionNode as E;
    match node {
        E::BinaryOperation(n) => smallvec::smallvec![n.left.as_ref(), n.right.as_ref()],
        E::UnaryOperation(n) => smallvec::smallvec![n.operand.as_ref()],
        E::IndexAccess(n) => smallvec::smallvec![n.object.as_ref(), n.index.as_ref()],
        E::PropertyAccess(n) => smallvec::smallvec![n.object.as_ref()],
        E::FunctionCall(n) => n.arguments.iter().collect(),
        E::MethodCall(n) => std::iter::once(n.object.as_ref())
            .chain(n.arguments.iter())
            .collect(),
        E::Collection(n) => n.elements.iter().collect(),
        E::Parenthesized(n) => smallvec::smallvec![n.as_ref()],
        E::TypeCheck(n) => smallvec::smallvec![n.expression.as_ref()],
        E::TypeCast(n) => smallvec::smallvec![n.expression.as_ref()],
        E::Union(n) => smallvec::smallvec![n.left.as_ref(), n.right.as_ref()],
        _ => SmallVec::new(),
    }
}

fn child_at(node: &ExpressionNode, index: usize) -> &ExpressionNode {
    use ExpressionNode as E;
    match node {
        E::BinaryOperation(n) => {
            if index == 0 {
                &n.left
            } else {
                &n.right
            }
        }
        E::UnaryOperation(n) => &n.operand,
        E::IndexAccess(n) => {
            if index == 0 {
                &n.object
            } else {
                &n.index
            }
        }
        E::PropertyAccess(n) => &n.object,
        E::FunctionCall(n) => &n.arguments[index],
        E::MethodCall(n) => {
            if index == 0 {
                &n.object
            } else {
                &n.arguments[index - 1]
            }
        }
        E::Collection(n) => &n.elements[index],
        E::Parenthesized(n) => n,
        E::TypeCheck(n) => &n.expression,
        E::TypeCast(n) => &n.expression,
        E::Union(n) => {
            if index == 0 {
                &n.left
            } else {
                &n.right
            }
        }
        _ => unreachable!("Compiler generated an invalid AST path"),
    }
}

fn address(node: &ExpressionNode) -> usize {
    node as *const ExpressionNode as usize
}
fn error(message: impl Into<String>) -> FhirPathError {
    FhirPathError::evaluation_error(crate::core::FP0054, message)
}

impl Program {
    pub(super) fn node_id(&self, node: &ExpressionNode) -> Id {
        self.addresses[&address(node)]
    }

    pub(super) fn compile(ast: Arc<ExpressionNode>, evaluator: &Evaluator) -> Arc<Self> {
        // Iterative traversal: compilation itself does not add a Rust stack
        // frame for every expression. Parent links are compact and linear.
        let mut syntax = Vec::new();
        let mut parents: Vec<Option<(Id, usize)>> = Vec::new();
        let mut addresses = HashMap::new();
        let mut pending = vec![(ast.as_ref(), None)];
        while let Some((node, parent)) = pending.pop() {
            let id = syntax.len();
            syntax.push(node);
            parents.push(parent);
            addresses.insert(address(node), id);
            for (index, child) in children(node).into_iter().enumerate().rev() {
                pending.push((child, Some((id, index))));
            }
        }
        let id = |node: &ExpressionNode| addresses[&address(node)];
        let binary = |left: &ExpressionNode, right: &ExpressionNode, kind, isolate| Op::Binary {
            left: id(left),
            right: id(right),
            kind,
            isolate,
            operator: if kind == BinaryOperator::Union {
                Some(Arc::new(
                    super::operations::union_operator::UnionOperatorEvaluator::new(),
                ))
            } else {
                evaluator
                    .operator_registry
                    .get_binary_operator(&kind)
                    .cloned()
            },
        };
        let mut nodes = Vec::with_capacity(syntax.len());
        for (node_id, node) in syntax.iter().enumerate() {
            use ExpressionNode as E;
            let op = match node {
                E::Literal(n) => {
                    Op::Value(evaluator.evaluate_literal(&n.value).map(Collection::single))
                }
                E::Identifier(n) => Op::Path {
                    receiver: None,
                    name: Arc::from(n.name.as_str()),
                },
                E::Variable(n) => Op::Variable(Arc::from(n.name.as_str())),
                E::Parenthesized(n) => Op::Forward(id(n)),
                E::BinaryOperation(n) => binary(&n.left, &n.right, n.operator, false),
                E::Union(n) => binary(&n.left, &n.right, BinaryOperator::Union, true),
                E::UnaryOperation(n) => Op::Unary {
                    operand: id(&n.operand),
                    operator: evaluator
                        .operator_registry
                        .get_unary_operator(&n.operator)
                        .cloned(),
                },
                E::IndexAccess(n) => Op::Index {
                    object: id(&n.object),
                    index: id(&n.index),
                },
                E::PropertyAccess(n) => Op::Path {
                    receiver: Some(id(&n.object)),
                    name: Arc::from(n.property.as_str()),
                },
                E::Collection(n) => Op::Collection(n.elements.iter().map(&id).collect()),
                E::TypeCheck(n) => Op::Type {
                    operand: id(&n.expression),
                    name: Arc::from(n.target_type.as_str()),
                    function: evaluator
                        .function_registry
                        .get_function_wrapper("is")
                        .cloned(),
                    check: true,
                },
                E::TypeCast(n) => Op::Type {
                    operand: id(&n.expression),
                    name: Arc::from(n.target_type.as_str()),
                    function: evaluator
                        .function_registry
                        .get_function_wrapper("as")
                        .cloned(),
                    check: false,
                },
                E::FunctionCall(_) | E::MethodCall(_) => {
                    let (name, arguments, receiver) = match node {
                        E::FunctionCall(n) => (n.name.as_str(), &n.arguments, None),
                        E::MethodCall(n) => (n.method.as_str(), &n.arguments, Some(id(&n.object))),
                        _ => unreachable!(),
                    };
                    let specialized = match node {
                        E::MethodCall(n)
                            if n.method == "where"
                                && n.arguments.len() == 1
                                && Evaluator::is_reference_type_check(&n.arguments[0]) =>
                        {
                            Evaluator::descendants_receiver(&n.object).map(&id)
                        }
                        _ => None,
                    };
                    if let Some(receiver) = specialized {
                        Op::ReferenceDescendants(receiver)
                    } else {
                        let function = evaluator
                            .function_registry
                            .get_function_wrapper(name)
                            .cloned();
                        let prepared = function
                            .as_ref()
                            .and_then(|f| super::plan::prepare_call(f, arguments, evaluator));
                        let mut path = Vec::new();
                        let mut current = node_id;
                        while let Some((parent, index)) = parents[current] {
                            path.push(index);
                            current = parent;
                        }
                        path.reverse();
                        Op::Call {
                            receiver,
                            name: Arc::from(name),
                            path: path.into_boxed_slice(),
                            function,
                            prepared,
                        }
                    }
                }
                _ => Op::Unsupported(format!("Expression type not yet implemented: {node:?}")),
            };
            nodes.push(Node {
                address: address(node),
                op,
                callback_free: false,
            });
        }
        // Children follow parents in preorder. Only functions with prepared
        // value arguments are guaranteed not to re-enter the AST callback.
        for index in (0..nodes.len()).rev() {
            let safe = |id: Id| nodes[id].callback_free;
            let callback_free = match &nodes[index].op {
                Op::Value(_) | Op::Variable(_) | Op::Unsupported(_) => true,
                Op::Path { receiver, .. } => receiver.is_none_or(safe),
                Op::Forward(id) | Op::ReferenceDescendants(id) => safe(*id),
                Op::Binary { left, right, .. } => safe(*left) && safe(*right),
                Op::Unary { operand, .. } => safe(*operand),
                Op::Index { object, index } => safe(*object) && safe(*index),
                Op::Collection(elements) => elements.iter().all(|id| safe(*id)),
                Op::Call {
                    receiver, prepared, ..
                } => prepared.is_some() && receiver.is_none_or(safe),
                Op::Type { .. } => false,
            };
            nodes[index].callback_free = callback_free;
        }
        Arc::new(Self {
            ast,
            nodes: nodes.into_boxed_slice(),
            addresses,
        })
    }

    fn arguments(&self, path: &[usize]) -> &[ExpressionNode] {
        let mut node = self.ast.as_ref();
        for &index in path {
            node = child_at(node, index);
        }
        match node {
            ExpressionNode::FunctionCall(n) => &n.arguments,
            ExpressionNode::MethodCall(n) => &n.arguments,
            _ => unreachable!("Compiler generated a non-call argument location"),
        }
    }
}

#[derive(Clone)]
struct NodeRef {
    program: Arc<Program>,
    id: Id,
}

struct Request {
    node: NodeRef,
    context: EvaluationContext,
    reply: Reply,
}

#[derive(Default)]
pub(super) struct Bridge {
    requests: Mutex<Vec<Request>>,
    ready: Arc<ReadyQueue>,
}

impl Drop for Bridge {
    fn drop(&mut self) {
        // A custom provider may retain a waker after cancellation.
        self.ready.outer.take();
    }
}

#[derive(Default)]
struct ReadyQueue {
    queue: Mutex<VecDeque<usize>>,
    outer: AtomicWaker,
}

struct TaskWake {
    index: usize,
    ready: Arc<ReadyQueue>,
    queued: AtomicBool,
}

impl TaskWake {
    fn new(index: usize, ready: Arc<ReadyQueue>) -> Arc<Self> {
        Arc::new(Self {
            index,
            ready,
            queued: AtomicBool::new(false),
        })
    }

    fn schedule(&self) {
        if !self.queued.swap(true, Ordering::AcqRel) {
            self.ready.queue.lock().push_back(self.index);
            self.ready.outer.wake();
        }
    }
}

impl ArcWake for TaskWake {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

pub(super) struct Callback<'a> {
    bridge: &'a Bridge,
    program: Arc<Program>,
}
impl<'a> Callback<'a> {
    pub(super) fn new(bridge: &'a Bridge, program: Arc<Program>) -> Self {
        Self { bridge, program }
    }

    pub(super) async fn evaluate(
        &self,
        evaluator: &Evaluator,
        mut node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Leaf callbacks cannot recursively request another expression. Execute
        // these without allocating a channel/task for common where(active).
        loop {
            if context.hoist_scope().is_some()
                && let Some(value) = context.hoisted_value(address(node))
            {
                return Ok(EvaluationResult { value });
            }
            match node {
                ExpressionNode::Parenthesized(inner) => node = inner,
                ExpressionNode::Literal(n) => {
                    return evaluator
                        .evaluate_literal(&n.value)
                        .map(|value| EvaluationResult {
                            value: Collection::single(value),
                        });
                }
                ExpressionNode::Variable(n) => {
                    return evaluator.evaluate_variable_sync(&n.name, context);
                }
                ExpressionNode::Identifier(n) => {
                    return evaluator.evaluate_path(&n.name, context).await;
                }
                _ => break,
            }
        }
        if let Some(&id) = self.program.addresses.get(&address(node))
            && self.program.nodes[id].callback_free
        {
            // This stack cannot enqueue recursive callbacks. Poll it locally:
            // no child task, channel, or queue locks per predicate evaluation.
            let mut stack = Stack::borrowed(&self.program, id, context.clone());
            return poll_fn(|cx| {
                let mut budget = 4096;
                let result = stack.poll(cx, evaluator, self.bridge, &mut budget);
                if result.is_pending() && budget == 0 {
                    cx.waker().wake_by_ref();
                }
                result
            })
            .await;
        }
        let node = match self.program.addresses.get(&address(node)) {
            Some(&id) => NodeRef {
                program: self.program.clone(),
                id,
            },
            // Custom functions may synthesize an AST. Own that tree for the
            // duration of its task; callbacks into it use its own address map.
            None => NodeRef {
                program: Program::compile(Arc::new(node.clone()), evaluator),
                id: 0,
            },
        };
        let (reply, result) = oneshot::channel();
        self.bridge.requests.lock().push(Request {
            node,
            context: context.clone(),
            reply,
        });
        self.bridge.ready.outer.wake();
        result
            .await
            .map_err(|_| error("Child evaluation cancelled"))?
    }
}

enum Frame<'a> {
    Eval(Id, EvaluationContext),
    BinaryLeft(Id, EvaluationContext),
    BinaryRight(Id, EvaluationContext, Collection),
    Unary(Id, EvaluationContext),
    IndexLeft(Id, EvaluationContext),
    IndexRight(Collection),
    Property(Id, EvaluationContext),
    Call(Id, EvaluationContext),
    Type(Id),
    ReferenceDescendants,
    Collection {
        node: Id,
        context: EvaluationContext,
        next: usize,
        values: Vec<FhirPathValue>,
    },
    Future(EvalFuture<'a>),
}

enum PathName<'a> {
    Borrowed(&'a str),
    Owned(Arc<str>),
}

impl AsRef<str> for PathName<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Borrowed(name) => name,
            Self::Owned(name) => name,
        }
    }
}

fn path_name<'a>(node: Id, program: &Program, borrowed: Option<&'a Program>) -> PathName<'a> {
    if let Some(program) = borrowed {
        let Op::Path { name, .. } = &program.nodes[node].op else {
            unreachable!()
        };
        PathName::Borrowed(name)
    } else {
        let Op::Path { name, .. } = &program.nodes[node].op else {
            unreachable!()
        };
        PathName::Owned(name.clone())
    }
}

struct Task<'a> {
    stack: Stack<'a>,
    reply: Option<Reply>,
    active: bool,
    wake: Arc<TaskWake>,
}

impl<'a> Task<'a> {
    fn new(
        node: NodeRef,
        context: EvaluationContext,
        reply: Option<Reply>,
        wake: Arc<TaskWake>,
    ) -> Self {
        Self {
            stack: Stack::new(node, context),
            reply,
            active: true,
            wake,
        }
    }
}

struct Stack<'a> {
    // A scheduled task owns its program once; an inline callback borrows it.
    // Individual instructions carry IDs, never contended program refcounts.
    program: Option<Cow<'a, Arc<Program>>>,
    state: StackState<'a>,
}

impl<'a> Stack<'a> {
    fn new(node: NodeRef, context: EvaluationContext) -> Self {
        Self {
            program: Some(Cow::Owned(node.program)),
            state: StackState::new(node.id, context),
        }
    }

    fn borrowed(program: &'a Arc<Program>, id: Id, context: EvaluationContext) -> Self {
        Self {
            program: Some(Cow::Borrowed(program)),
            state: StackState::new(id, context),
        }
    }

    fn reset(&mut self, node: NodeRef, context: EvaluationContext) {
        debug_assert!(self.state.frames.is_empty());
        self.program = Some(Cow::Owned(node.program));
        self.state.frames.push(Frame::Eval(node.id, context));
    }

    fn clear(&mut self) {
        self.state.frames.clear();
        self.state.value = None;
        self.program = None;
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        evaluator: &'a Evaluator,
        bridge: &'a Bridge,
        budget: &mut usize,
    ) -> Poll<Result<EvaluationResult>> {
        let program = self.program.as_ref().expect("Inactive VM stack");
        let borrowed: Option<&'a Program> = match program {
            Cow::Borrowed(program) => Some((*program).as_ref()),
            Cow::Owned(_) => None,
        };
        self.state
            .poll(cx, evaluator, bridge, program.as_ref(), borrowed, budget)
    }
}

struct StackState<'a> {
    frames: SmallVec<[Frame<'a>; 2]>,
    value: Option<Collection>,
}

impl<'a> StackState<'a> {
    fn new(node: Id, context: EvaluationContext) -> Self {
        Self {
            frames: smallvec::smallvec![Frame::Eval(node, context)],
            value: None,
        }
    }

    fn take(&mut self) -> Collection {
        self.value.take().expect("Missing VM operand")
    }

    fn call(
        &mut self,
        node: Id,
        program: &Arc<Program>,
        context: EvaluationContext,
        input: Collection,
        evaluator: &'a Evaluator,
        bridge: &'a Bridge,
    ) -> Result<()> {
        let Op::Call {
            name,
            function,
            prepared,
            ..
        } = &program.nodes[node].op
        else {
            unreachable!()
        };
        if let Some((function, args)) = prepared {
            self.value = Some(
                if input.is_empty()
                    && matches!(
                        function.metadata().null_propagation,
                        NullPropagationStrategy::Focus
                    )
                {
                    Collection::empty()
                } else {
                    function.evaluate_sync_borrowed(input, args)?.value
                },
            );
        } else if function.is_none() {
            return Err(error(format!("Unknown function: {name}")));
        } else {
            let program = program.clone();
            self.frames.push(Frame::Future(Box::pin(async move {
                let Op::Call { path, function, .. } = &program.nodes[node].op else {
                    unreachable!()
                };
                evaluator
                    .evaluate_function_wrapper(
                        function.as_ref().unwrap(),
                        program.arguments(path),
                        &context,
                        input,
                        AsyncNodeEvaluator::for_vm(evaluator, bridge, program.clone()),
                    )
                    .await
            })));
        }
        Ok(())
    }

    fn operation(
        &mut self,
        operator: Option<&Arc<dyn OperationEvaluator>>,
        context: EvaluationContext,
        left: Collection,
        right: Collection,
    ) -> Result<()> {
        let operator = operator.ok_or_else(|| error("Unsupported operator"))?;
        if operator.supports_sync() {
            self.value = Some(
                operator
                    .evaluate_sync(Collection::empty(), &context, left, right)?
                    .value,
            );
        } else {
            let operator = operator.clone();
            self.frames.push(Frame::Future(Box::pin(async move {
                operator
                    .evaluate(Collection::empty(), &context, left, right)
                    .await
            })));
        }
        Ok(())
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        evaluator: &'a Evaluator,
        bridge: &'a Bridge,
        program: &Arc<Program>,
        borrowed_program: Option<&'a Program>,
        budget: &mut usize,
    ) -> Poll<Result<EvaluationResult>> {
        loop {
            if *budget == 0 {
                return Poll::Pending;
            }
            *budget -= 1;
            let Some(frame) = self.frames.pop() else {
                return Poll::Ready(Ok(EvaluationResult { value: self.take() }));
            };
            let mut waiting = false;
            let step = (|| -> Result<()> {
                match frame {
                    Frame::Eval(node, context) => {
                        if context.hoist_scope().is_some()
                            && let Some(value) = context.hoisted_value(program.nodes[node].address)
                        {
                            self.value = Some(value);
                            return Ok(());
                        }
                        match &program.nodes[node].op {
                            Op::Value(value) => self.value = Some(value.clone()?),
                            Op::Variable(name) => {
                                self.value =
                                    Some(evaluator.evaluate_variable_sync(name, &context)?.value);
                            }
                            Op::Forward(id) => self.frames.push(Frame::Eval(*id, context)),
                            Op::Path {
                                receiver: Some(receiver),
                                ..
                            } => {
                                self.frames.push(Frame::Property(node, context.clone()));
                                self.frames.push(Frame::Eval(*receiver, context));
                            }
                            Op::Path { receiver: None, .. } => {
                                let name = path_name(node, program, borrowed_program);
                                self.frames.push(Frame::Future(Box::pin(async move {
                                    evaluator.evaluate_path(name.as_ref(), &context).await
                                })));
                            }
                            Op::Binary { left, isolate, .. } => {
                                let left_context = if *isolate {
                                    context.nest()
                                } else {
                                    context.clone()
                                };
                                self.frames.push(Frame::BinaryLeft(node, context));
                                self.frames.push(Frame::Eval(*left, left_context));
                            }
                            Op::Unary { operand, .. } => {
                                self.frames.push(Frame::Unary(node, context.clone()));
                                self.frames.push(Frame::Eval(*operand, context));
                            }
                            Op::Index { object, .. } => {
                                self.frames.push(Frame::IndexLeft(node, context.clone()));
                                self.frames.push(Frame::Eval(*object, context));
                            }
                            Op::Collection(elements) => {
                                if let Some(&first) = elements.first() {
                                    self.frames.push(Frame::Collection {
                                        node,
                                        context: context.clone(),
                                        next: 1,
                                        values: Vec::new(),
                                    });
                                    self.frames.push(Frame::Eval(first, context));
                                } else {
                                    self.value = Some(Collection::from_values_with_ordering(
                                        Vec::new(),
                                        true,
                                    ));
                                }
                            }
                            Op::Call {
                                receiver: Some(receiver),
                                ..
                            } => {
                                self.frames.push(Frame::Call(node, context.clone()));
                                self.frames.push(Frame::Eval(*receiver, context));
                            }
                            Op::Call { receiver: None, .. } => {
                                let input = context.input_collection().clone();
                                self.call(node, program, context, input, evaluator, bridge)?;
                            }
                            Op::Type { operand, .. } => {
                                self.frames.push(Frame::Type(node));
                                self.frames.push(Frame::Eval(*operand, context));
                            }
                            Op::ReferenceDescendants(receiver) => {
                                self.frames.push(Frame::ReferenceDescendants);
                                self.frames.push(Frame::Eval(*receiver, context));
                            }
                            Op::Unsupported(message) => return Err(error(message.clone())),
                        }
                    }
                    Frame::BinaryLeft(node, context) => {
                        let left = self.take();
                        let Op::Binary {
                            right,
                            kind,
                            isolate,
                            ..
                        } = &program.nodes[node].op
                        else {
                            unreachable!()
                        };
                        if let Some(value) = Evaluator::short_circuit(kind, &left) {
                            self.value = Some(Collection::single(value));
                        } else {
                            let right_context = if *isolate {
                                context.nest()
                            } else {
                                context.clone()
                            };
                            self.frames.push(Frame::BinaryRight(node, context, left));
                            self.frames.push(Frame::Eval(*right, right_context));
                        }
                    }
                    Frame::BinaryRight(node, context, left) => {
                        let right = self.take();
                        let Op::Binary { operator, .. } = &program.nodes[node].op else {
                            unreachable!()
                        };
                        self.operation(operator.as_ref(), context, left, right)?;
                    }
                    Frame::Unary(node, context) => {
                        let operand = self.take();
                        let Op::Unary { operator, .. } = &program.nodes[node].op else {
                            unreachable!()
                        };
                        self.operation(operator.as_ref(), context, operand, Collection::empty())?;
                    }
                    Frame::IndexLeft(node, context) => {
                        let object = self.take();
                        let Op::Index { index, .. } = &program.nodes[node].op else {
                            unreachable!()
                        };
                        self.frames.push(Frame::IndexRight(object));
                        self.frames.push(Frame::Eval(*index, context));
                    }
                    Frame::IndexRight(object) => {
                        let index = self.take();
                        self.value = Some(evaluator.evaluate_index_sync(object, index)?.value);
                    }
                    Frame::Property(node, context) => {
                        let context = context.create_child_context(self.take());
                        let name = path_name(node, program, borrowed_program);
                        self.frames.push(Frame::Future(Box::pin(async move {
                            evaluator.evaluate_path(name.as_ref(), &context).await
                        })));
                    }
                    Frame::Call(node, context) => {
                        let input = self.take();
                        self.call(node, program, context, input, evaluator, bridge)?;
                    }
                    Frame::Type(node) => {
                        let context = evaluator.type_context(self.take());
                        let program = program.clone();
                        self.frames.push(Frame::Future(Box::pin(async move {
                            let Op::Type {
                                name,
                                function,
                                check,
                                ..
                            } = &program.nodes[node].op
                            else {
                                unreachable!()
                            };
                            let function = function.as_ref().ok_or_else(|| {
                                error(format!(
                                    "Unknown function: {}",
                                    if *check { "is" } else { "as" }
                                ))
                            })?;
                            let args = [ExpressionNode::Identifier(
                                crate::ast::expression::IdentifierNode {
                                    name: name.to_string(),
                                    location: None,
                                },
                            )];
                            evaluator
                                .evaluate_function_wrapper(
                                    function,
                                    &args,
                                    &context,
                                    context.input_collection().clone(),
                                    AsyncNodeEvaluator::for_vm(evaluator, bridge, program.clone()),
                                )
                                .await
                        })));
                    }
                    Frame::ReferenceDescendants => {
                        let mut values = Vec::new();
                        for item in self.take().iter() {
                            if let FhirPathValue::Resource(root, _, _) = item {
                                evaluator.collect_reference_descendants(root, &mut values);
                            }
                        }
                        self.value = Some(Collection::from(values));
                    }
                    Frame::Collection {
                        node,
                        context,
                        next,
                        mut values,
                    } => {
                        values.extend(self.take());
                        let Op::Collection(elements) = &program.nodes[node].op else {
                            unreachable!()
                        };
                        if let Some(&element) = elements.get(next) {
                            self.frames.push(Frame::Collection {
                                node,
                                context: context.clone(),
                                next: next + 1,
                                values,
                            });
                            self.frames.push(Frame::Eval(element, context));
                        } else {
                            self.value = Some(Collection::from_values_with_ordering(values, true));
                        }
                    }
                    Frame::Future(mut future) => match future.as_mut().poll(cx) {
                        Poll::Ready(result) => self.value = Some(result?.value),
                        Poll::Pending => {
                            self.frames.push(Frame::Future(future));
                            waiting = true;
                        }
                    },
                }
                Ok(())
            })();
            if let Err(error) = step {
                return Poll::Ready(Err(error));
            }
            if waiting {
                return Poll::Pending;
            }
        }
    }
}

pub(super) async fn run(
    program: Arc<Program>,
    evaluator: &Evaluator,
    context: &EvaluationContext,
) -> Result<EvaluationResult> {
    run_at(program, 0, evaluator, context).await
}

pub(super) async fn run_at(
    program: Arc<Program>,
    entry: Id,
    evaluator: &Evaluator,
    context: &EvaluationContext,
) -> Result<EvaluationResult> {
    let bridge = Bridge::default();
    let root_wake = TaskWake::new(0, bridge.ready.clone());
    let mut tasks = vec![Task::new(
        NodeRef { program, id: entry },
        context.clone(),
        None,
        root_wake.clone(),
    )];
    let mut free = Vec::new();
    root_wake.schedule();
    poll_fn(|cx| {
        bridge.ready.outer.register(cx.waker());
        // Bound interpreter dispatch between cooperative yields. Work internal
        // to one synchronous custom function is still the function's responsibility.
        let mut budget = 4096;
        loop {
            for request in bridge.requests.lock().drain(..) {
                if request.reply.is_closed() {
                    continue;
                }
                let index = if let Some(index) = free.pop() {
                    let task: &mut Task<'_> = &mut tasks[index];
                    task.stack.reset(request.node, request.context);
                    task.reply = Some(request.reply);
                    task.active = true;
                    index
                } else {
                    let index = tasks.len();
                    tasks.push(Task::new(
                        request.node,
                        request.context,
                        Some(request.reply),
                        TaskWake::new(index, bridge.ready.clone()),
                    ));
                    index
                };
                tasks[index].wake.schedule();
            }
            let Some(index) = bridge.ready.queue.lock().pop_front() else {
                return Poll::Pending;
            };
            let task = &mut tasks[index];
            task.wake.queued.store(false, Ordering::Release);
            if !task.active {
                continue;
            }
            let wake = task.wake.clone();
            let waker = waker_ref(&wake);
            let mut task_context = Context::from_waker(&waker);
            // Losing select branches are dropped even when their I/O never wakes.
            if task
                .reply
                .as_mut()
                .is_some_and(|reply| reply.poll_closed(&mut task_context).is_ready())
            {
                task.stack.clear();
                task.reply = None;
                task.active = false;
                free.push(index);
                continue;
            }
            if let Poll::Ready(result) =
                task.stack
                    .poll(&mut task_context, evaluator, &bridge, &mut budget)
            {
                if index == 0 {
                    return Poll::Ready(result);
                }
                if let Some(reply) = task.reply.take() {
                    let _ = reply.send(result);
                }
                task.stack.clear();
                task.active = false;
                free.push(index);
            }
            if budget == 0 {
                if task.active {
                    task.wake.schedule();
                }
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }
    })
    .await
}
