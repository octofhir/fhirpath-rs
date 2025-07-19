use fhirpath_core::errors::FhirPathError;
use fhirpath_core::evaluator::{
    evaluate_expression_with_visitor, AstVisitor, EvaluationContext, NoopVisitor,
};
use fhirpath_core::model::FhirPathValue;
use fhirpath_core::parser::AstNode;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

// A test visitor that counts the number of nodes visited
struct CountingVisitor {
    before_count: Rc<RefCell<usize>>,
    after_count: Rc<RefCell<usize>>,
    node_types: Rc<RefCell<Vec<String>>>,
}

impl CountingVisitor {
    fn new() -> Self {
        Self {
            before_count: Rc::new(RefCell::new(0)),
            after_count: Rc::new(RefCell::new(0)),
            node_types: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn before_count(&self) -> usize {
        *self.before_count.borrow()
    }

    fn after_count(&self) -> usize {
        *self.after_count.borrow()
    }

    fn node_types(&self) -> Vec<String> {
        self.node_types.borrow().clone()
    }
}

impl AstVisitor for CountingVisitor {
    fn before_evaluate(&self, node: &AstNode, _context: &EvaluationContext) {
        *self.before_count.borrow_mut() += 1;

        // Record the node type
        let node_type = match node {
            AstNode::Identifier(_) => "Identifier",
            AstNode::StringLiteral(_) => "StringLiteral",
            AstNode::NumberLiteral { .. } => "NumberLiteral",
            AstNode::BooleanLiteral(_) => "BooleanLiteral",
            AstNode::DateTimeLiteral(_) => "DateTimeLiteral",
            AstNode::QuantityLiteral { .. } => "QuantityLiteral",
            AstNode::Path(_, _) => "Path",
            AstNode::BinaryOp { .. } => "BinaryOp",
            AstNode::UnaryOp { .. } => "UnaryOp",
            AstNode::FunctionCall { .. } => "FunctionCall",
            AstNode::Indexer { .. } => "Indexer",
            AstNode::Variable(_) => "Variable",
        };

        self.node_types.borrow_mut().push(node_type.to_string());
    }

    fn after_evaluate(
        &self,
        _node: &AstNode,
        _context: &EvaluationContext,
        _result: &Result<FhirPathValue, FhirPathError>,
    ) {
        *self.after_count.borrow_mut() += 1;
    }
}

#[test]
fn test_visitor_counts_nodes() {
    let visitor = CountingVisitor::new();
    let resource = json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    // Simple expression
    let result = evaluate_expression_with_visitor("Patient.name.given", resource.clone(), &visitor);
    assert!(result.is_ok());

    // Check that the visitor was called for each node
    assert!(visitor.before_count() > 0);
    assert_eq!(visitor.before_count(), visitor.after_count());

    // Check that we visited the expected node types
    let node_types = visitor.node_types();
    assert!(node_types.contains(&"Identifier".to_string()));
    assert!(node_types.contains(&"Path".to_string()));
}

#[test]
fn test_visitor_with_complex_expression() {
    let visitor = CountingVisitor::new();
    let resource = json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ],
        "gender": "male"
    });

    // Complex expression with multiple operators
    let result = evaluate_expression_with_visitor(
        "Patient.name[0].given[0] = 'John' and Patient.gender = 'male'",
        resource.clone(),
        &visitor,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), FhirPathValue::Boolean(true));

    // Check that the visitor was called for each node
    assert!(visitor.before_count() > 0);
    assert_eq!(visitor.before_count(), visitor.after_count());

    // Check that we visited the expected node types
    let node_types = visitor.node_types();
    assert!(node_types.contains(&"Identifier".to_string()));
    assert!(node_types.contains(&"Path".to_string()));
    assert!(node_types.contains(&"BinaryOp".to_string()));
    assert!(node_types.contains(&"StringLiteral".to_string()));
    assert!(node_types.contains(&"Indexer".to_string()));
}

#[test]
fn test_noop_visitor() {
    let visitor = NoopVisitor::new();
    let resource = json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ]
    });

    // The NoopVisitor should not affect the result
    let result1 =
        evaluate_expression_with_visitor("Patient.name.given", resource.clone(), &visitor);
    let result2 = evaluate_expression_with_visitor(
        "Patient.name.given",
        resource.clone(),
        &CountingVisitor::new(),
    );

    // Check that both results are Ok or both are Err
    assert!(result1.is_ok() == result2.is_ok());

    // If both are Ok, check that the unwrapped values are equal
    if result1.is_ok() {
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
